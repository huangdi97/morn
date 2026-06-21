//! oauth — Manages OAuth token persistence and provider authorization state.
use crate::core::error::MornError;
use crate::core::storage::{SaveOAuthTokenArgs, Storage};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub auth_url: String,
    pub token_url: String,
    pub scopes: Vec<String>,
    pub redirect_uri: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OAuthToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<String>,
    pub scope: Option<String>,
    pub token_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderInfo {
    pub name: String,
    pub has_client_id: bool,
    pub has_token: bool,
    pub auth_url: String,
}

pub struct OAuthManager {
    storage: Arc<Storage>,
    providers: HashMap<String, OAuthConfig>,
}

impl OAuthManager {
    pub fn new(storage: Arc<Storage>) -> Self {
        let mut mgr = OAuthManager {
            storage,
            providers: HashMap::new(),
        };
        mgr.register_defaults();
        let _ = mgr.load_provider_configs();
        mgr
    }

    fn register_defaults(&mut self) {
        self.providers.insert(
            "github".to_string(),
            OAuthConfig {
                client_id: String::new(),
                client_secret: String::new(),
                auth_url: "https://github.com/login/oauth/authorize".to_string(),
                token_url: "https://github.com/login/oauth/access_token".to_string(),
                scopes: vec!["read:user".to_string(), "repo".to_string()],
                redirect_uri: "http://localhost:1420/oauth/callback".to_string(),
            },
        );
        self.providers.insert(
            "google".to_string(),
            OAuthConfig {
                client_id: String::new(),
                client_secret: String::new(),
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
                token_url: "https://oauth2.googleapis.com/token".to_string(),
                scopes: vec![
                    "openid".to_string(),
                    "email".to_string(),
                    "profile".to_string(),
                ],
                redirect_uri: "http://localhost:1420/oauth/callback".to_string(),
            },
        );
        self.providers.insert(
            "slack".to_string(),
            OAuthConfig {
                client_id: String::new(),
                client_secret: String::new(),
                auth_url: "https://slack.com/oauth/v2/authorize".to_string(),
                token_url: "https://slack.com/api/oauth.v2.access".to_string(),
                scopes: vec!["channels:read".to_string(), "chat:write".to_string()],
                redirect_uri: "http://localhost:1420/oauth/callback".to_string(),
            },
        );
        self.providers.insert(
            "notion".to_string(),
            OAuthConfig {
                client_id: String::new(),
                client_secret: String::new(),
                auth_url: "https://api.notion.com/v1/oauth/authorize".to_string(),
                token_url: "https://api.notion.com/v1/oauth/token".to_string(),
                scopes: vec!["read:database".to_string(), "read:page".to_string()],
                redirect_uri: "http://localhost:1420/oauth/callback".to_string(),
            },
        );
    }

    pub fn register_provider(&mut self, name: &str, config: OAuthConfig) -> Result<(), MornError> {
        self.providers.insert(name.to_string(), config);
        Ok(())
    }

    pub fn get_auth_url(&self, provider: &str) -> Result<String, MornError> {
        let config = self
            .providers
            .get(provider)
            .ok_or_else(|| MornError::Internal(format!("Unknown provider: {}", provider)))?;
        let scopes = config.scopes.join(" ");
        Ok(format!(
            "{}?client_id={}&redirect_uri={}&scope={}&response_type=code&state={}",
            config.auth_url, config.client_id, config.redirect_uri, scopes, provider
        ))
    }

    pub fn handle_callback(&self, provider: &str, code: &str) -> Result<OAuthToken, MornError> {
        let config = self
            .providers
            .get(provider)
            .ok_or_else(|| MornError::Internal(format!("unknown provider: {}", provider)))?;

        let params = serde_json::json!({
            "client_id": config.client_id,
            "client_secret": config.client_secret,
            "code": code,
            "grant_type": "authorization_code",
            "redirect_uri": config.redirect_uri,
        });

        let client = reqwest::blocking::Client::new();
        let resp = client
            .post(&config.token_url)
            .header("Accept", "application/json")
            .json(&params)
            .send()
            .map_err(|e| MornError::Network(format!("OAuth token request failed: {}", e)))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp
                .text()
                .unwrap_or_else(|_| "cannot read body".to_string());
            return Err(MornError::Network(format!(
                "OAuth token request returned {}: {}",
                status, body
            )));
        }

        let token_data: Value = resp
            .json()
            .map_err(|e| MornError::Serialization(format!("OAuth response parse failed: {}", e)))?;

        if let Some(error) = token_data.get("error").and_then(|v| v.as_str()) {
            let desc = token_data
                .get("error_description")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            return Err(MornError::Network(format!(
                "OAuth provider returned error: {} - {}",
                error, desc
            )));
        }

        let access_token = token_data["access_token"]
            .as_str()
            .ok_or_else(|| MornError::Network("no access_token in response".into()))?;

        let token = OAuthToken {
            access_token: access_token.to_string(),
            refresh_token: token_data["refresh_token"].as_str().map(String::from),
            expires_at: token_data["expires_in"]
                .as_i64()
                .map(|secs| (chrono::Utc::now() + chrono::Duration::seconds(secs)).to_rfc3339()),
            scope: token_data["scope"].as_str().map(String::from),
            token_type: token_data["token_type"]
                .as_str()
                .unwrap_or("Bearer")
                .to_string(),
        };

        let token_id = uuid::Uuid::new_v4().to_string();
        self.storage.save_oauth_token_args(SaveOAuthTokenArgs {
            id: &token_id,
            provider,
            user_id: "default",
            access_token: &token.access_token,
            refresh_token: token.refresh_token.as_deref(),
            expires_at: token.expires_at.as_deref(),
            scope: token.scope.as_deref(),
        })?;

        Ok(token)
    }

    pub fn get_token(&self, provider: &str, user_id: &str) -> Result<OAuthToken, MornError> {
        let row = self
            .storage
            .get_oauth_token(provider, user_id)?
            .ok_or_else(|| format!("No token found for {} / {}", provider, user_id))?;
        Ok(OAuthToken {
            access_token: row.3,
            refresh_token: row.4,
            expires_at: row.5,
            scope: row.6,
            token_type: "Bearer".to_string(),
        })
    }

    pub fn list_providers(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub fn get_provider_info(&self, provider: &str) -> Result<ProviderInfo, MornError> {
        let config = self
            .providers
            .get(provider)
            .ok_or_else(|| MornError::Internal(format!("Unknown provider: {}", provider)))?;
        let has_token = self
            .storage
            .get_oauth_token(provider, "default")
            .ok()
            .flatten()
            .is_some();
        Ok(ProviderInfo {
            name: provider.to_string(),
            has_client_id: !config.client_id.is_empty(),
            has_token,
            auth_url: config.auth_url.clone(),
        })
    }

    pub fn list_provider_info(&self) -> Vec<ProviderInfo> {
        self.providers
            .keys()
            .filter_map(|name| self.get_provider_info(name).ok())
            .collect()
    }

    pub fn set_provider_credentials(
        &mut self,
        provider: &str,
        client_id: String,
        client_secret: String,
    ) -> Result<(), MornError> {
        let config = self
            .providers
            .get_mut(provider)
            .ok_or_else(|| MornError::Internal(format!("Unknown provider: {}", provider)))?;
        config.client_id = client_id;
        config.client_secret = client_secret;
        self.save_provider_configs()
    }

    pub fn get_provider_config(&self, provider: &str) -> Result<OAuthConfig, MornError> {
        self.providers
            .get(provider)
            .cloned()
            .ok_or_else(|| MornError::Internal(format!("Unknown provider: {}", provider)))
    }

    fn save_provider_configs(&self) -> Result<(), MornError> {
        let json = serde_json::to_string(&self.providers)
            .map_err(|e| MornError::Serialization(e.to_string()))?;
        self.storage.set_setting("oauth_providers", &json)
    }

    fn load_provider_configs(&mut self) -> Result<(), MornError> {
        if let Some(json) = self.storage.get_setting("oauth_providers")? {
            if let Ok(configs) = serde_json::from_str::<HashMap<String, OAuthConfig>>(&json) {
                for (name, config) in configs {
                    if let Some(existing) = self.providers.get(&name) {
                        let merged = OAuthConfig {
                            client_id: config.client_id,
                            client_secret: config.client_secret,
                            auth_url: existing.auth_url.clone(),
                            token_url: existing.token_url.clone(),
                            scopes: existing.scopes.clone(),
                            redirect_uri: existing.redirect_uri.clone(),
                        };
                        self.providers.insert(name, merged);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn is_token_expired(&self, token: &OAuthToken) -> bool {
        if let Some(ref expires_at) = token.expires_at {
            if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(expires_at) {
                return chrono::Utc::now() > expires;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_manager() -> OAuthManager {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        OAuthManager::new(storage)
    }

    #[test]
    fn test_list_providers() {
        let mgr = setup_manager();
        let providers = mgr.list_providers();
        assert!(providers.contains(&"github".to_string()));
        assert!(providers.contains(&"google".to_string()));
        assert!(providers.contains(&"slack".to_string()));
        assert!(providers.contains(&"notion".to_string()));
        assert_eq!(providers.len(), 4);
    }

    #[test]
    fn test_get_auth_url() {
        let mgr = setup_manager();
        let url = mgr.get_auth_url("github").unwrap();
        assert!(url.contains("github.com"));
        assert!(url.contains("client_id="));
        assert!(url.contains("redirect_uri="));
        assert!(url.contains("state=github"));
        assert!(url.contains("response_type=code"));
    }

    #[test]
    fn test_get_auth_url_unknown_provider() {
        let mgr = setup_manager();
        assert!(mgr.get_auth_url("unknown").is_err());
    }

    #[test]
    fn test_handle_callback_no_credentials() {
        let mgr = setup_manager();
        let result = mgr.handle_callback("github", "auth_code_123");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_token() {
        let mgr = setup_manager();
        let token_id = uuid::Uuid::new_v4().to_string();
        mgr.storage
            .save_oauth_token_args(SaveOAuthTokenArgs {
                id: &token_id,
                provider: "github",
                user_id: "default",
                access_token: "test_token",
                refresh_token: None,
                expires_at: None,
                scope: None,
            })
            .unwrap();
        let token = mgr.get_token("github", "default").unwrap();
        assert_eq!(token.token_type, "Bearer");
        assert_eq!(token.access_token, "test_token");
    }

    #[test]
    fn test_register_custom_provider() {
        let mut mgr = setup_manager();
        let config = OAuthConfig {
            client_id: "custom_id".to_string(),
            client_secret: "custom_secret".to_string(),
            auth_url: "https://custom.com/auth".to_string(),
            token_url: "https://custom.com/token".to_string(),
            scopes: vec!["read".to_string()],
            redirect_uri: "http://localhost/callback".to_string(),
        };
        mgr.register_provider("custom", config).unwrap();
        assert!(mgr.list_providers().contains(&"custom".to_string()));
    }

    #[test]
    fn test_token_expiry() {
        let mgr = setup_manager();
        let valid_token = OAuthToken {
            access_token: "valid".to_string(),
            refresh_token: None,
            expires_at: Some((chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339()),
            scope: None,
            token_type: "Bearer".to_string(),
        };
        assert!(!mgr.is_token_expired(&valid_token));

        let expired_token = OAuthToken {
            access_token: "expired".to_string(),
            refresh_token: None,
            expires_at: Some((chrono::Utc::now() - chrono::Duration::hours(1)).to_rfc3339()),
            scope: None,
            token_type: "Bearer".to_string(),
        };
        assert!(mgr.is_token_expired(&expired_token));
    }

    #[test]
    fn test_get_token_not_found() {
        let mgr = setup_manager();
        assert!(mgr.get_token("github", "nonexistent").is_err());
    }
}
