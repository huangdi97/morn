//! publisher — Publishes studio capabilities into registries and marketplaces.
use crate::core::registry::Registry;
use crate::core::storage::Storage;

#[allow(dead_code)] /* 预留：Studio 发布流程聚合入口 */
pub struct StudioPublisher {
    registry: Option<Registry>,
    storage: Option<Storage>,
}

impl StudioPublisher {
    pub fn new(registry: Option<Registry>, storage: Option<Storage>) -> Self {
        StudioPublisher { registry, storage }
    }

    pub fn publish_agent(&self, agent_id: &str) -> Result<(), String> {
        if let Some(ref storage) = self.storage {
            if let Some(mut agent) = storage.get_agent(agent_id)? {
                agent.status = "active".to_string();
                storage.update_agent_status(agent_id, "active")?;
            }
        }
        Ok(())
    }

    pub fn unpublish_agent(&self, agent_id: &str) -> Result<(), String> {
        if let Some(ref storage) = self.storage {
            storage.update_agent_status(agent_id, "inactive")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage::{AgentRecord, Storage};

    fn storage_with_agent(status: &str) -> Storage {
        let storage = Storage::new_in_memory().unwrap();
        storage
            .insert_agent(&AgentRecord {
                id: "agent-1".into(),
                name: "Publishable".into(),
                component_type: "agent".into(),
                config_json: None,
                status: status.into(),
                trust_score: 70.0,
                created_at: chrono::Utc::now().to_rfc3339(),
                updated_at: None,
            })
            .unwrap();
        storage
    }

    #[test]
    fn publish_agent_marks_agent_active() {
        let storage = storage_with_agent("inactive");
        let publisher = StudioPublisher::new(None, Some(storage.clone()));

        publisher.publish_agent("agent-1").unwrap();

        assert_eq!(
            storage.get_agent("agent-1").unwrap().unwrap().status,
            "active"
        );
    }

    #[test]
    fn unpublish_agent_marks_agent_inactive() {
        let storage = storage_with_agent("active");
        let publisher = StudioPublisher::new(None, Some(storage.clone()));

        publisher.unpublish_agent("agent-1").unwrap();

        assert_eq!(
            storage.get_agent("agent-1").unwrap().unwrap().status,
            "inactive"
        );
    }

    #[test]
    fn publishing_without_storage_is_noop() {
        let publisher = StudioPublisher::new(None, None);

        assert!(publisher.publish_agent("missing").is_ok());
        assert!(publisher.unpublish_agent("missing").is_ok());
    }
}
