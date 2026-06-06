use crate::core::storage::Storage;
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
                redirect_uri: "http://localhost:3000/auth/github/callback".to_string(),
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
                redirect_uri: "http://localhost:3000/auth/google/callback".to_string(),
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
                redirect_uri: "http://localhost:3000/auth/slack/callback".to_string(),
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
                redirect_uri: "http://localhost:3000/auth/notion/callback".to_string(),
            },
        );
    }

    pub fn register_provider(&mut self, name: &str, config: OAuthConfig) -> Result<(), String> {
        self.providers.insert(name.to_string(), config);
        Ok(())
    }

    pub fn get_auth_url(&self, provider: &str) -> Result<String, String> {
        let config = self
            .providers
            .get(provider)
            .ok_or_else(|| format!("Unknown provider: {}", provider))?;
        let scopes = config.scopes.join(" ");
        Ok(format!(
            "{}?client_id={}&redirect_uri={}&scope={}&response_type=code",
            config.auth_url, config.client_id, config.redirect_uri, scopes
        ))
    }

    pub fn handle_callback(&self, provider: &str, code: &str) -> Result<OAuthToken, String> {
        let token = OAuthToken {
            access_token: format!("mock_{}_token_{}", provider, code),
            refresh_token: Some(format!("mock_refresh_{}", code)),
            expires_at: Some((chrono::Utc::now() + chrono::Duration::hours(1)).to_rfc3339()),
            scope: None,
            token_type: "Bearer".to_string(),
        };

        let token_id = uuid::Uuid::new_v4().to_string();
        self.storage.save_oauth_token(
            &token_id,
            provider,
            "default",
            &token.access_token,
            token.refresh_token.as_deref(),
            token.expires_at.as_deref(),
            token.scope.as_deref(),
        )?;

        Ok(token)
    }

    pub fn get_token(&self, provider: &str, user_id: &str) -> Result<OAuthToken, String> {
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
    }

    #[test]
    fn test_get_auth_url_unknown_provider() {
        let mgr = setup_manager();
        assert!(mgr.get_auth_url("unknown").is_err());
    }

    #[test]
    fn test_handle_callback() {
        let mgr = setup_manager();
        let token = mgr.handle_callback("github", "auth_code_123").unwrap();
        assert_eq!(token.token_type, "Bearer");
        assert!(token.access_token.contains("github"));
    }

    #[test]
    fn test_get_token() {
        let mgr = setup_manager();
        mgr.handle_callback("github", "code_1").unwrap();
        let token = mgr.get_token("github", "default").unwrap();
        assert_eq!(token.token_type, "Bearer");
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
