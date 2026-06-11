//! Security constitution — security levels, policy definitions, and the Merkle audit chain.

use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, PartialEq)]
pub enum SecurityLevel {
    L1HardBlocked,
    L2NeedApproval,
    L3NeedNotify,
    L4Free,
}

impl SecurityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityLevel::L1HardBlocked => "L1HardBlocked",
            SecurityLevel::L2NeedApproval => "L2NeedApproval",
            SecurityLevel::L3NeedNotify => "L3NeedNotify",
            SecurityLevel::L4Free => "L4Free",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub name: String,
    pub level: SecurityLevel,
    pub pattern: String,
    pub description: String,
}

// ---------------------------------------------------------------------------
// Merkle Audit Chain
// ---------------------------------------------------------------------------

fn merkle_hash(prev_hash: u64, data: &str, nonce: u64) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    prev_hash.hash(&mut hasher);
    data.hash(&mut hasher);
    nonce.hash(&mut hasher);
    hasher.finish()
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AuditEntry {
    pub index: u64,
    pub timestamp: String,
    pub agent_id: String,
    pub action_type: String,
    pub data_hash: u64,
    pub prev_hash: u64,
    pub hash: u64,
    pub nonce: u64,
}

impl AuditEntry {
    pub fn new(index: u64, prev_hash: u64, agent_id: &str, action_type: &str, data: &str) -> Self {
        let data_hash = {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            data.hash(&mut h);
            h.finish()
        };
        let nonce = 0;
        let hash = merkle_hash(prev_hash, data, nonce);
        AuditEntry {
            index,
            timestamp: chrono::Utc::now().to_rfc3339(),
            agent_id: agent_id.to_string(),
            action_type: action_type.to_string(),
            data_hash,
            prev_hash,
            hash,
            nonce,
        }
    }

    pub fn verify(&self, data: &str) -> bool {
        let expected_hash = merkle_hash(self.prev_hash, data, self.nonce);
        self.hash == expected_hash
    }
}

pub struct AuditLog {
    entries: Vec<AuditEntry>,
}

impl AuditLog {
    pub fn new() -> Self {
        AuditLog {
            entries: Vec::new(),
        }
    }

    pub fn append(&mut self, agent_id: &str, action_type: &str, data: &str) -> &AuditEntry {
        let prev_hash = self.entries.last().map(|e| e.hash).unwrap_or(0);
        let index = self.entries.len() as u64;
        let entry = AuditEntry::new(index, prev_hash, agent_id, action_type, data);
        self.entries.push(entry);
        let last_index = self.entries.len() - 1;
        &self.entries[last_index]
    }

    pub fn verify_chain(&self) -> bool {
        if self.entries.is_empty() {
            return true;
        }
        if self.entries[0].prev_hash != 0 {
            return false;
        }
        for i in 1..self.entries.len() {
            let prev = &self.entries[i - 1];
            let curr = &self.entries[i];
            if curr.prev_hash != prev.hash {
                return false;
            }
        }
        true
    }

    pub fn entries(&self) -> &[AuditEntry] {
        &self.entries
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn query_by_agent(&self, agent_id: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.agent_id == agent_id)
            .collect()
    }

    pub fn query_by_action(&self, action_type: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.action_type == action_type)
            .collect()
    }

    pub fn query_by_time_range(&self, start: &str, end: &str) -> Vec<&AuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.timestamp.as_str() >= start && e.timestamp.as_str() <= end)
            .collect()
    }

    pub fn verify_entry(&self, index: usize, data: &str) -> bool {
        self.entries
            .get(index)
            .map(|e| e.verify(data))
            .unwrap_or(false)
    }

    pub fn tamper(&mut self, index: usize, new_data: &str) -> Result<(), String> {
        let entry = self
            .entries
            .get_mut(index)
            .ok_or_else(|| format!("entry index {} out of range", index))?;
        let data_hash = {
            let mut h = std::collections::hash_map::DefaultHasher::new();
            new_data.hash(&mut h);
            h.finish()
        };
        entry.data_hash = data_hash;
        entry.hash = merkle_hash(entry.prev_hash, new_data, entry.nonce);
        Ok(())
    }
}

impl Default for AuditLog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_creation() {
        let entry = AuditEntry::new(0, 0, "agent-1", "chat", "hello");
        assert_eq!(entry.index, 0);
        assert_eq!(entry.agent_id, "agent-1");
        assert_eq!(entry.action_type, "chat");
        assert!(entry.hash != 0);
    }

    #[test]
    fn test_audit_entry_verify() {
        let entry = AuditEntry::new(0, 0, "agent-1", "chat", "hello");
        assert!(entry.verify("hello"));
        assert!(!entry.verify("wrong-data"));
    }

    #[test]
    fn test_audit_log_append_and_chain() {
        let mut log = AuditLog::new();
        assert!(log.is_empty());
        log.append("agent-a", "chat", "msg1");
        log.append("agent-b", "tool", "calc");
        log.append("agent-a", "chat", "msg2");
        assert_eq!(log.len(), 3);
        assert!(log.verify_chain());
    }

    #[test]
    fn test_audit_log_verify_chain_empty() {
        let log = AuditLog::new();
        assert!(log.verify_chain());
    }

    #[test]
    fn test_audit_log_tamper_detection() {
        let mut log = AuditLog::new();
        log.append("agent-a", "chat", "original");
        log.append("agent-b", "tool", "calc");
        assert!(log.verify_chain());
        log.tamper(0, "tampered").unwrap();
        assert!(!log.verify_chain());
    }

    #[test]
    fn test_query_by_agent() {
        let mut log = AuditLog::new();
        log.append("alice", "chat", "hi");
        log.append("bob", "tool", "search");
        log.append("alice", "chat", "bye");
        let alice_entries = log.query_by_agent("alice");
        assert_eq!(alice_entries.len(), 2);
        let bob_entries = log.query_by_agent("bob");
        assert_eq!(bob_entries.len(), 1);
    }

    #[test]
    fn test_query_by_action() {
        let mut log = AuditLog::new();
        log.append("agent-a", "chat", "hello");
        log.append("agent-a", "tool", "search");
        log.append("agent-b", "chat", "world");
        let chat_entries = log.query_by_action("chat");
        assert_eq!(chat_entries.len(), 2);
        let tool_entries = log.query_by_action("tool");
        assert_eq!(tool_entries.len(), 1);
    }

    #[test]
    fn test_verify_entry() {
        let mut log = AuditLog::new();
        log.append("agent-a", "chat", "hello world");
        assert!(log.verify_entry(0, "hello world"));
        assert!(!log.verify_entry(0, "wrong"));
        assert!(!log.verify_entry(99, "anything"));
    }

    #[test]
    fn test_merkle_chain_integrity() {
        let mut log = AuditLog::new();
        for i in 0..5 {
            log.append("agent-x", "action", &format!("data-{}", i));
        }
        assert!(log.verify_chain());
        log.tamper(2, "corrupted").unwrap();
        assert!(!log.verify_chain());
    }
}
