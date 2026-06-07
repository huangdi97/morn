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
