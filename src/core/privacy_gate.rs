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

    pub fn parse_from_str(s: &str) -> Self {
        match s {
            "critical" => DataSensitivity::Critical,
            "private" => DataSensitivity::Private,
            "internal" => DataSensitivity::Internal,
            _ => DataSensitivity::Public,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PrivacyRule {
    pub keywords: Vec<String>,
    pub sensitivity: DataSensitivity,
    pub action: String,
}

pub struct PrivacyGate {
    rules: Vec<PrivacyRule>,
}

impl Default for PrivacyGate {
    fn default() -> Self {
        Self::new()
    }
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
            DataSensitivity::parse_from_str("critical"),
            DataSensitivity::Critical
        );
        assert_eq!(
            DataSensitivity::parse_from_str("public"),
            DataSensitivity::Public
        );
    }
}
