//! sync_enhanced — Device pairing and lightweight encrypted sync payload helpers.
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevicePairing {
    pub device_id: String,
    pub public_key: String,
    pub paired_at: String,
    pub last_sync: Option<String>,
}

pub struct SyncEnhancedManager {
    paired_devices: HashMap<String, DevicePairing>,
    encryption_key: Option<String>,
}

impl SyncEnhancedManager {
    pub fn new() -> Self {
        Self {
            paired_devices: HashMap::new(),
            encryption_key: None,
        }
    }

    pub fn pair_device(&mut self, device_id: &str, public_key: &str) -> DevicePairing {
        let pairing = DevicePairing {
            device_id: device_id.to_string(),
            public_key: public_key.to_string(),
            paired_at: chrono::Utc::now().to_rfc3339(),
            last_sync: None,
        };
        self.paired_devices
            .insert(device_id.to_string(), pairing.clone());
        pairing
    }

    pub fn unpair_device(&mut self, device_id: &str) -> Option<DevicePairing> {
        self.paired_devices.remove(device_id)
    }

    pub fn list_paired(&self) -> Vec<DevicePairing> {
        let mut devices = self.paired_devices.values().cloned().collect::<Vec<_>>();
        devices.sort_by(|left, right| left.device_id.cmp(&right.device_id));
        devices
    }

    pub fn set_encryption_key(&mut self, key: &str) {
        self.encryption_key = Some(key.to_string());
    }

    pub fn encrypt_payload(&self, data: &[u8]) -> Vec<u8> {
        self.apply_xor(data)
    }

    pub fn decrypt_payload(&self, data: &[u8]) -> Vec<u8> {
        self.apply_xor(data)
    }

    fn apply_xor(&self, data: &[u8]) -> Vec<u8> {
        let Some(key) = self.encryption_key.as_ref() else {
            return data.to_vec();
        };
        let key_bytes = key.as_bytes();
        if key_bytes.is_empty() {
            return data.to_vec();
        }

        data.iter()
            .enumerate()
            .map(|(idx, byte)| byte ^ key_bytes[idx % key_bytes.len()])
            .collect()
    }
}

impl Default for SyncEnhancedManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pair_and_unpair_device() {
        let mut manager = SyncEnhancedManager::new();
        let pairing = manager.pair_device("device-b", "pubkey-b");
        manager.pair_device("device-a", "pubkey-a");

        assert_eq!(pairing.device_id, "device-b");
        assert_eq!(manager.list_paired()[0].device_id, "device-a");

        let removed = manager.unpair_device("device-b").unwrap();
        assert_eq!(removed.public_key, "pubkey-b");
        assert_eq!(manager.list_paired().len(), 1);
    }

    #[test]
    fn test_encrypt_and_decrypt_payload() {
        let mut manager = SyncEnhancedManager::new();
        manager.set_encryption_key("secret");
        let payload = b"{\"sync\":true}";

        let encrypted = manager.encrypt_payload(payload);
        assert_ne!(encrypted, payload);
        assert_eq!(manager.decrypt_payload(&encrypted), payload);
    }

    #[test]
    fn test_without_key_returns_original_payload() {
        let manager = SyncEnhancedManager::new();
        let payload = b"plain";

        assert_eq!(manager.encrypt_payload(payload), payload);
        assert_eq!(manager.decrypt_payload(payload), payload);
    }
}
