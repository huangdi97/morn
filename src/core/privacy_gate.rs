//! privacy_gate — Evaluates privacy rules before data is shared or processed.
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use base64::Engine as _;
use ring::aead::{self, Aad, LessSafeKey, Nonce, UnboundKey};
use ring::digest;
use ring::rand::{SecureRandom, SystemRandom};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DataSensitivity {
    Public,
    Internal,
    Private,
    Critical,
}

impl DataSensitivity {
    pub fn as_str(&self) -> &'static str {
        match self {
            DataSensitivity::Public => "public",
            DataSensitivity::Internal => "internal",
            DataSensitivity::Private => "private",
            DataSensitivity::Critical => "critical",
        }
    }

    pub fn from_str_value(s: &str) -> Self {
        match s {
            "critical" => DataSensitivity::Critical,
            "private" => DataSensitivity::Private,
            "internal" => DataSensitivity::Internal,
            _ => DataSensitivity::Public,
        }
    }

    #[allow(clippy::should_implement_trait)] /* 预留：兼容旧调用入口 */
    pub fn from_str(s: &str) -> Self {
        Self::from_str_value(s)
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrivacyRule {
    pub keywords: Vec<String>,
    pub sensitivity: DataSensitivity,
    pub action: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct E2EEConfig {
    pub enabled: bool,
    pub algorithm: String,
    pub key: String,
}

impl Default for E2EEConfig {
    fn default() -> Self {
        E2EEConfig {
            enabled: false,
            algorithm: "aes-256-gcm".to_string(),
            key: String::new(),
        }
    }
}

pub struct PrivacyGate {
    rules: Vec<PrivacyRule>,
}

impl PrivacyGate {
    pub fn new() -> Self {
        let mut gate = PrivacyGate { rules: Vec::new() };
        gate.add_rule(PrivacyRule {
            keywords: vec![
                "password".into(),
                "secret".into(),
                "api_key".into(),
                "token".into(),
                "auth".into(),
                "credential".into(),
            ],
            sensitivity: DataSensitivity::Critical,
            action: "block".into(),
        });
        gate.add_rule(PrivacyRule {
            keywords: vec![
                "email".into(),
                "phone".into(),
                "ssn".into(),
                "credit card".into(),
                "address".into(),
                "birth".into(),
            ],
            sensitivity: DataSensitivity::Private,
            action: "anonymize".into(),
        });
        gate
    }

    pub fn with_rules(rules: Vec<PrivacyRule>) -> Self {
        PrivacyGate { rules }
    }

    pub fn add_rule(&mut self, rule: PrivacyRule) {
        self.rules.push(rule);
    }

    pub fn assess(&self, data: &str, _context: &str) -> DataSensitivity {
        let lower = data.to_lowercase();
        for rule in &self.rules {
            for keyword in &rule.keywords {
                if lower.contains(&keyword.to_lowercase()) {
                    return rule.sensitivity.clone();
                }
            }
        }
        DataSensitivity::Public
    }

    pub fn anonymize(&self, data: &str, level: &str) -> String {
        match level {
            "high" => data
                .chars()
                .map(|c| if c.is_alphanumeric() { '*' } else { c })
                .collect(),
            "medium" => {
                let mut result: Vec<char> = data.chars().collect();
                for i in (result.len() / 3)..(2 * result.len() / 3) {
                    if i < result.len() {
                        result[i] = '*';
                    }
                }
                result.into_iter().collect()
            }
            _ => {
                let words: Vec<&str> = data.split_whitespace().collect();
                words
                    .iter()
                    .map(|w| {
                        if w.len() > 3 {
                            let first = &w[..1];
                            let last = &w[w.len() - 1..];
                            format!("{}{}{}", first, "*".repeat(w.len() - 2), last)
                        } else {
                            w.to_string()
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            }
        }
    }

    pub fn allow_cloud(&self, level: &DataSensitivity) -> bool {
        match level {
            DataSensitivity::Public | DataSensitivity::Internal => true,
            DataSensitivity::Private | DataSensitivity::Critical => false,
        }
    }

    pub fn check_data(&self, data: &str, context: &str) -> PrivacyCheckResult {
        let sensitivity = self.assess(data, context);
        match sensitivity {
            DataSensitivity::Critical => PrivacyCheckResult {
                allowed: false,
                sensitivity,
                action: "blocked".to_string(),
                message: Some("Data contains critical sensitive information".to_string()),
            },
            DataSensitivity::Private => {
                let anonymized = self.anonymize(data, "low");
                PrivacyCheckResult {
                    allowed: true,
                    sensitivity,
                    action: "anonymized".to_string(),
                    message: Some(format!("Anonymized data: {}", anonymized)),
                }
            }
            _ => PrivacyCheckResult {
                allowed: true,
                sensitivity,
                action: "allowed".to_string(),
                message: None,
            },
        }
    }

    pub fn encrypt_message(&self, message: &str, config: &E2EEConfig) -> Result<String, String> {
        if !config.enabled {
            return Ok(message.to_string());
        }
        validate_algorithm(&config.algorithm)?;

        let key = aead_key(config)?;
        let rng = SystemRandom::new();
        let mut nonce_bytes = [0u8; 12];
        rng.fill(&mut nonce_bytes)
            .map_err(|_| "failed to generate E2EE nonce".to_string())?;

        let mut payload = message.as_bytes().to_vec();
        key.seal_in_place_append_tag(
            Nonce::assume_unique_for_key(nonce_bytes),
            Aad::from(normalize_algorithm(&config.algorithm).as_bytes()),
            &mut payload,
        )
        .map_err(|_| "failed to encrypt E2EE message".to_string())?;

        let mut packed = nonce_bytes.to_vec();
        packed.extend(payload);
        Ok(format!(
            "e2ee:{}:{}",
            normalize_algorithm(&config.algorithm),
            BASE64_STANDARD.encode(packed)
        ))
    }

    pub fn decrypt_message(&self, message: &str, config: &E2EEConfig) -> Result<String, String> {
        if !config.enabled {
            return Ok(message.to_string());
        }
        validate_algorithm(&config.algorithm)?;

        let prefix = format!("e2ee:{}:", normalize_algorithm(&config.algorithm));
        let encoded = message.strip_prefix(&prefix).ok_or_else(|| {
            "message is not encrypted with the configured E2EE algorithm".to_string()
        })?;
        let mut packed = BASE64_STANDARD
            .decode(encoded)
            .map_err(|e| format!("invalid E2EE payload: {}", e))?;
        if packed.len() <= 12 {
            return Err("invalid E2EE payload length".to_string());
        }

        let mut nonce_bytes = [0u8; 12];
        nonce_bytes.copy_from_slice(&packed[..12]);
        let mut ciphertext = packed.split_off(12);
        let key = aead_key(config)?;
        let plaintext = key
            .open_in_place(
                Nonce::assume_unique_for_key(nonce_bytes),
                Aad::from(normalize_algorithm(&config.algorithm).as_bytes()),
                &mut ciphertext,
            )
            .map_err(|_| "failed to decrypt E2EE message".to_string())?;

        String::from_utf8(plaintext.to_vec())
            .map_err(|e| format!("E2EE plaintext is not valid UTF-8: {}", e))
    }
}

fn normalize_algorithm(algorithm: &str) -> &'static str {
    match algorithm.trim().to_ascii_lowercase().as_str() {
        "aes-256-gcm" | "aes_256_gcm" | "aes256-gcm" | "aes256gcm" => "aes-256-gcm",
        _ => "unsupported",
    }
}

fn validate_algorithm(algorithm: &str) -> Result<(), String> {
    if normalize_algorithm(algorithm) == "aes-256-gcm" {
        Ok(())
    } else {
        Err(format!("unsupported E2EE algorithm '{}'", algorithm))
    }
}

fn aead_key(config: &E2EEConfig) -> Result<LessSafeKey, String> {
    if config.key.is_empty() {
        return Err("E2EE key is empty".to_string());
    }

    let decoded = BASE64_STANDARD.decode(&config.key).ok();
    let key_bytes = match decoded {
        Some(bytes) if bytes.len() == 32 => bytes,
        _ => digest::digest(&digest::SHA256, config.key.as_bytes())
            .as_ref()
            .to_vec(),
    };
    let unbound = UnboundKey::new(&aead::AES_256_GCM, &key_bytes)
        .map_err(|_| "failed to initialize E2EE key".to_string())?;
    Ok(LessSafeKey::new(unbound))
}

impl Default for PrivacyGate {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrivacyCheckResult {
    pub allowed: bool,
    pub sensitivity: DataSensitivity,
    pub action: String,
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_gate() -> PrivacyGate {
        PrivacyGate::new()
    }

    #[test]
    fn test_assess_public() {
        let gate = setup_gate();
        let level = gate.assess("hello world this is public data", "chat");
        assert_eq!(level, DataSensitivity::Public);
    }

    #[test]
    fn test_assess_private_email() {
        let gate = setup_gate();
        let level = gate.assess("my email is user@example.com", "context");
        assert_eq!(level, DataSensitivity::Private);
    }

    #[test]
    fn test_assess_critical_password() {
        let gate = setup_gate();
        let level = gate.assess("my password is secret123", "login");
        assert_eq!(level, DataSensitivity::Critical);
    }

    #[test]
    fn test_anonymize_high() {
        let gate = setup_gate();
        let result = gate.anonymize("secret123", "high");
        assert_eq!(result, "*********");
    }

    #[test]
    fn test_anonymize_low() {
        let gate = setup_gate();
        let result = gate.anonymize("hello world", "low");
        assert_ne!(result, "hello world");
    }

    #[test]
    fn test_allow_cloud_public() {
        let gate = setup_gate();
        assert!(gate.allow_cloud(&DataSensitivity::Public));
        assert!(gate.allow_cloud(&DataSensitivity::Internal));
        assert!(!gate.allow_cloud(&DataSensitivity::Private));
        assert!(!gate.allow_cloud(&DataSensitivity::Critical));
    }

    #[test]
    fn test_check_data_critical() {
        let gate = setup_gate();
        let result = gate.check_data("my api_key is abc123", "chat");
        assert!(!result.allowed);
        assert_eq!(result.action, "blocked");
    }

    #[test]
    fn test_check_data_private() {
        let gate = setup_gate();
        let result = gate.check_data("my email is test@test.com", "chat");
        assert!(result.allowed);
        assert_eq!(result.action, "anonymized");
    }

    #[test]
    fn test_add_custom_rule() {
        let mut gate = setup_gate();
        gate.add_rule(PrivacyRule {
            keywords: vec!["project_x".to_string()],
            sensitivity: DataSensitivity::Critical,
            action: "block".to_string(),
        });
        let level = gate.assess("project_x is confidential", "work");
        assert_eq!(level, DataSensitivity::Critical);
    }

    #[test]
    fn test_sensitivity_roundtrip() {
        assert_eq!(
            DataSensitivity::from_str_value("critical"),
            DataSensitivity::Critical
        );
        assert_eq!(
            DataSensitivity::from_str_value("public"),
            DataSensitivity::Public
        );
    }

    #[test]
    fn test_e2ee_encrypt_decrypt_roundtrip() {
        let gate = setup_gate();
        let config = E2EEConfig {
            enabled: true,
            algorithm: "aes-256-gcm".to_string(),
            key: "test-channel-key".to_string(),
        };

        let encrypted = gate
            .encrypt_message("hello private channel", &config)
            .unwrap();
        assert_ne!(encrypted, "hello private channel");
        assert!(encrypted.starts_with("e2ee:aes-256-gcm:"));

        let decrypted = gate.decrypt_message(&encrypted, &config).unwrap();
        assert_eq!(decrypted, "hello private channel");
    }

    #[test]
    fn test_e2ee_disabled_passthrough() {
        let gate = setup_gate();
        let config = E2EEConfig::default();
        assert_eq!(gate.encrypt_message("plain", &config).unwrap(), "plain");
        assert_eq!(gate.decrypt_message("plain", &config).unwrap(), "plain");
    }
}
