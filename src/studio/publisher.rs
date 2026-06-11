//! publisher — Publishes studio capabilities into registries and marketplaces.
use crate::core::registry::Registry;
use crate::core::storage::Storage;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

#[allow(dead_code)] /* 预留：Studio 发布流程聚合入口 */
pub struct StudioPublisher {
    registry: Option<Registry>,
    storage: Option<Storage>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum PublishStage {
    Validated,
    Packaged,
    Uploaded,
    Notified,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PublishArtifact {
    pub component_id: String,
    pub component_type: String,
    pub version: String,
    pub manifest: serde_json::Value,
    pub checksum: String,
    pub bytes: usize,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PublishReceipt {
    pub component_id: String,
    pub package_id: String,
    pub upload_url: String,
    pub stages: Vec<PublishStage>,
}

impl StudioPublisher {
    pub fn new(registry: Option<Registry>, storage: Option<Storage>) -> Self {
        StudioPublisher { registry, storage }
    }

    pub fn publish_agent(&self, agent_id: &str) -> Result<(), String> {
        if self.storage.is_none() {
            return Ok(());
        }

        self.publish_component(agent_id).map(|_| ())
    }

    pub fn unpublish_agent(&self, agent_id: &str) -> Result<(), String> {
        if let Some(ref storage) = self.storage {
            if storage.get_agent(agent_id)?.is_none() {
                return Err(format!("agent '{}' not found", agent_id));
            }
            storage.update_agent_status(agent_id, "inactive")?;
        }
        Ok(())
    }

    pub fn publish_component(&self, component_id: &str) -> Result<PublishReceipt, String> {
        let mut stages = Vec::new();
        self.validate(component_id)?;
        stages.push(PublishStage::Validated);

        let artifact = self.package(component_id)?;
        stages.push(PublishStage::Packaged);

        let upload_url = self.upload(&artifact)?;
        stages.push(PublishStage::Uploaded);

        self.notify(&artifact, &upload_url)?;
        stages.push(PublishStage::Notified);

        if let Some(ref storage) = self.storage {
            storage.update_agent_status(component_id, "active")?;
        }

        Ok(PublishReceipt {
            component_id: component_id.to_string(),
            package_id: artifact.checksum,
            upload_url,
            stages,
        })
    }

    fn validate(&self, component_id: &str) -> Result<(), String> {
        if component_id.trim().is_empty() {
            return Err("component id cannot be empty".into());
        }
        if let Some(ref storage) = self.storage {
            let agent = storage
                .get_agent(component_id)?
                .ok_or_else(|| format!("component '{}' not found", component_id))?;
            if agent.name.trim().is_empty() {
                return Err(format!("component '{}' has empty name", component_id));
            }
            if agent.component_type.trim().is_empty() {
                return Err(format!("component '{}' has empty type", component_id));
            }
        }
        Ok(())
    }

    fn package(&self, component_id: &str) -> Result<PublishArtifact, String> {
        let (component_type, name, status, config_json) = if let Some(ref storage) = self.storage {
            let agent = storage
                .get_agent(component_id)?
                .ok_or_else(|| format!("component '{}' not found", component_id))?;
            (
                agent.component_type,
                agent.name,
                agent.status,
                agent.config_json.unwrap_or_else(|| "{}".into()),
            )
        } else {
            (
                "agent".to_string(),
                component_id.to_string(),
                "external".to_string(),
                "{}".to_string(),
            )
        };

        let registry_version = self
            .registry
            .as_ref()
            .and_then(|registry| registry.get_version(component_id))
            .unwrap_or("0.1.0");
        let manifest = serde_json::json!({
            "id": component_id,
            "name": name,
            "component_type": component_type,
            "version": registry_version,
            "status": status,
            "config": serde_json::from_str::<serde_json::Value>(&config_json)
                .unwrap_or_else(|_| serde_json::json!({"raw": config_json})),
        });
        let payload = serde_json::to_string(&manifest).map_err(|e| e.to_string())?;
        let mut hasher = DefaultHasher::new();
        payload.hash(&mut hasher);

        Ok(PublishArtifact {
            component_id: component_id.to_string(),
            component_type,
            version: registry_version.to_string(),
            manifest,
            checksum: format!("pkg-{:016x}", hasher.finish()),
            bytes: payload.len(),
        })
    }

    fn upload(&self, artifact: &PublishArtifact) -> Result<String, String> {
        if artifact.bytes == 0 {
            return Err(format!("artifact '{}' is empty", artifact.component_id));
        }
        Ok(format!(
            "morn://studio/packages/{}/{}",
            artifact.component_type, artifact.checksum
        ))
    }

    fn notify(&self, artifact: &PublishArtifact, upload_url: &str) -> Result<(), String> {
        if upload_url.trim().is_empty() {
            return Err(format!(
                "upload url for '{}' is empty",
                artifact.component_id
            ));
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

    #[test]
    fn publish_component_returns_full_stage_receipt() {
        let storage = storage_with_agent("draft");
        let publisher = StudioPublisher::new(None, Some(storage.clone()));

        let receipt = publisher.publish_component("agent-1").unwrap();

        assert_eq!(receipt.component_id, "agent-1");
        assert_eq!(
            receipt.stages,
            vec![
                PublishStage::Validated,
                PublishStage::Packaged,
                PublishStage::Uploaded,
                PublishStage::Notified,
            ]
        );
        assert!(receipt
            .upload_url
            .starts_with("morn://studio/packages/agent/"));
        assert_eq!(
            storage.get_agent("agent-1").unwrap().unwrap().status,
            "active"
        );
    }

    #[test]
    fn publish_component_rejects_missing_component() {
        let storage = Storage::new_in_memory().unwrap();
        let publisher = StudioPublisher::new(None, Some(storage));

        let err = publisher.publish_component("missing").unwrap_err();

        assert!(err.contains("missing"));
    }
}
