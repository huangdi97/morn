use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ChannelKind {
    Telegram,
    WeChat,
    DingTalk,
    Feishu,
    Custom(String),
}

impl std::fmt::Display for ChannelKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelKind::Telegram => write!(f, "telegram"),
            ChannelKind::WeChat => write!(f, "wechat"),
            ChannelKind::DingTalk => write!(f, "dingtalk"),
            ChannelKind::Feishu => write!(f, "feishu"),
            ChannelKind::Custom(s) => write!(f, "custom_{}", s),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct ChannelUser {
    pub channel: ChannelKind,
    pub channel_user_id: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UnifiedIdentity {
    pub identity_id: String,
    pub display_name: String,
    pub bound_users: Vec<ChannelUser>,
    pub metadata: HashMap<String, String>,
}

pub struct IdentityBridge {
    channel_to_identity: HashMap<ChannelUser, String>,
    identity_store: HashMap<String, UnifiedIdentity>,
}

impl IdentityBridge {
    pub fn new() -> Self {
        IdentityBridge {
            channel_to_identity: HashMap::new(),
            identity_store: HashMap::new(),
        }
    }

    pub fn bind(
        &mut self,
        identity_id: &str,
        display_name: &str,
        channel: ChannelKind,
        channel_user_id: &str,
    ) -> Result<(), String> {
        let user = ChannelUser {
            channel,
            channel_user_id: channel_user_id.to_string(),
        };

        if self.channel_to_identity.contains_key(&user) {
            return Err(format!(
                "User {} on {} is already bound to another identity",
                user.channel_user_id, user.channel
            ));
        }

        let identity = self
            .identity_store
            .entry(identity_id.to_string())
            .or_insert_with(|| UnifiedIdentity {
                identity_id: identity_id.to_string(),
                display_name: display_name.to_string(),
                bound_users: Vec::new(),
                metadata: HashMap::new(),
            });

        identity.bound_users.push(user.clone());
        self.channel_to_identity
            .insert(user, identity_id.to_string());
        Ok(())
    }

    pub fn resolve(
        &self,
        channel: &ChannelKind,
        channel_user_id: &str,
    ) -> Option<&UnifiedIdentity> {
        let user = ChannelUser {
            channel: channel.clone(),
            channel_user_id: channel_user_id.to_string(),
        };
        let identity_id = self.channel_to_identity.get(&user)?;
        self.identity_store.get(identity_id)
    }

    pub fn unbind(&mut self, channel: &ChannelKind, channel_user_id: &str) -> Result<(), String> {
        let user = ChannelUser {
            channel: channel.clone(),
            channel_user_id: channel_user_id.to_string(),
        };

        let identity_id = self
            .channel_to_identity
            .remove(&user)
            .ok_or_else(|| format!("No binding found for {} on {}", channel_user_id, channel))?;

        if let Some(identity) = self.identity_store.get_mut(&identity_id) {
            identity.bound_users.retain(|u| u != &user);
            if identity.bound_users.is_empty() {
                self.identity_store.remove(&identity_id);
            }
        }

        Ok(())
    }

    pub fn get_identity(&self, identity_id: &str) -> Option<&UnifiedIdentity> {
        self.identity_store.get(identity_id)
    }

    pub fn list_identities(&self) -> Vec<&UnifiedIdentity> {
        self.identity_store.values().collect()
    }

    pub fn get_bound_channels(&self, identity_id: &str) -> Vec<&ChannelUser> {
        self.identity_store
            .get(identity_id)
            .map(|id| id.bound_users.iter().collect())
            .unwrap_or_default()
    }

    pub fn set_metadata(
        &mut self,
        identity_id: &str,
        key: &str,
        value: &str,
    ) -> Result<(), String> {
        let identity = self
            .identity_store
            .get_mut(identity_id)
            .ok_or_else(|| format!("Identity {} not found", identity_id))?;
        identity.metadata.insert(key.to_string(), value.to_string());
        Ok(())
    }

    pub fn resolve_all_channels(
        &self,
        channel_user_id: &str,
    ) -> Vec<(ChannelKind, Option<&UnifiedIdentity>)> {
        let channels = vec![
            ChannelKind::Telegram,
            ChannelKind::WeChat,
            ChannelKind::DingTalk,
            ChannelKind::Feishu,
        ];
        channels
            .into_iter()
            .map(|c| {
                let identity = self.resolve(&c, channel_user_id);
                (c, identity)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_bridge() -> IdentityBridge {
        let mut bridge = IdentityBridge::new();
        bridge
            .bind("user_1", "Alice", ChannelKind::Telegram, "tg_123")
            .unwrap();
        bridge
            .bind("user_1", "Alice", ChannelKind::WeChat, "wx_456")
            .unwrap();
        bridge
            .bind("user_2", "Bob", ChannelKind::DingTalk, "dt_789")
            .unwrap();
        bridge
    }

    #[test]
    fn test_bind_and_resolve() {
        let bridge = setup_bridge();
        let identity = bridge.resolve(&ChannelKind::Telegram, "tg_123").unwrap();
        assert_eq!(identity.identity_id, "user_1");
        assert_eq!(identity.display_name, "Alice");
    }

    #[test]
    fn test_resolve_cross_channel() {
        let bridge = setup_bridge();
        let via_wechat = bridge.resolve(&ChannelKind::WeChat, "wx_456").unwrap();
        let via_telegram = bridge.resolve(&ChannelKind::Telegram, "tg_123").unwrap();
        assert_eq!(via_wechat.identity_id, via_telegram.identity_id);
        assert_eq!(via_wechat.identity_id, "user_1");
    }

    #[test]
    fn test_resolve_nonexistent() {
        let bridge = setup_bridge();
        assert!(bridge.resolve(&ChannelKind::Feishu, "fs_000").is_none());
    }

    #[test]
    fn test_duplicate_bind_rejected() {
        let mut bridge = setup_bridge();
        let result = bridge.bind("user_2", "Alice", ChannelKind::Telegram, "tg_123");
        assert!(result.is_err());
    }

    #[test]
    fn test_unbind() {
        let mut bridge = setup_bridge();
        bridge.unbind(&ChannelKind::Telegram, "tg_123").unwrap();
        assert!(bridge.resolve(&ChannelKind::Telegram, "tg_123").is_none());
        let identity = bridge.get_identity("user_1").unwrap();
        assert_eq!(identity.bound_users.len(), 1);
    }

    #[test]
    fn test_unbind_last_removes_identity() {
        let mut bridge = IdentityBridge::new();
        bridge
            .bind("user_x", "X", ChannelKind::Telegram, "tg_x")
            .unwrap();
        bridge.unbind(&ChannelKind::Telegram, "tg_x").unwrap();
        assert!(bridge.get_identity("user_x").is_none());
    }

    #[test]
    fn test_list_identities() {
        let bridge = setup_bridge();
        let ids = bridge.list_identities();
        assert_eq!(ids.len(), 2);
    }

    #[test]
    fn test_get_bound_channels() {
        let bridge = setup_bridge();
        let channels = bridge.get_bound_channels("user_1");
        assert_eq!(channels.len(), 2);
    }

    #[test]
    fn test_set_metadata() {
        let mut bridge = setup_bridge();
        bridge.set_metadata("user_1", "role", "admin").unwrap();
        let identity = bridge.get_identity("user_1").unwrap();
        assert_eq!(identity.metadata.get("role").unwrap(), "admin");
    }

    #[test]
    fn test_resolve_all_channels() {
        let bridge = setup_bridge();
        let results = bridge.resolve_all_channels("tg_123");
        assert!(results
            .iter()
            .any(|(c, id)| *c == ChannelKind::Telegram && id.is_some()));
        assert!(results
            .iter()
            .any(|(c, id)| *c == ChannelKind::WeChat && id.is_none()));
        assert!(results
            .iter()
            .any(|(c, id)| *c == ChannelKind::DingTalk && id.is_none()));
        assert!(results
            .iter()
            .any(|(c, id)| *c == ChannelKind::Feishu && id.is_none()));
    }

    #[test]
    fn test_channel_kind_display() {
        assert_eq!(ChannelKind::Telegram.to_string(), "telegram");
        assert_eq!(ChannelKind::WeChat.to_string(), "wechat");
        assert_eq!(ChannelKind::DingTalk.to_string(), "dingtalk");
        assert_eq!(ChannelKind::Feishu.to_string(), "feishu");
        assert_eq!(
            ChannelKind::Custom("slack".to_string()).to_string(),
            "custom_slack"
        );
    }

    #[test]
    fn test_channel_user_serde() {
        let user = ChannelUser {
            channel: ChannelKind::WeChat,
            channel_user_id: "wx_001".to_string(),
        };
        let json = serde_json::to_string(&user).unwrap();
        let deserialized: ChannelUser = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.channel, ChannelKind::WeChat);
        assert_eq!(deserialized.channel_user_id, "wx_001");
    }
}
