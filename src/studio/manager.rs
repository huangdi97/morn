//! manager — Manages studio projects and registered capabilities.
use crate::core::error::MornError;
use crate::core::registry::Registry;
use crate::core::storage::Storage;

#[derive(Debug, Clone, serde::Serialize)]
#[allow(dead_code)] /* 预留：Studio 列表摘要 API 输出 */
pub struct ComponentSummary {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub status: String,
    pub trust_score: f64,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ComponentDetail {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub config_json: Option<String>,
    pub status: String,
    pub trust_score: f64,
}

#[derive(Debug, Clone)]
pub struct CreateComponentDef {
    pub name: String,
    pub component_type: String,
    pub config_json: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateComponentDef {
    pub name: Option<String>,
    pub config_json: Option<String>,
    pub status: Option<String>,
}

#[allow(dead_code)] /* 预留：Studio 组件管理聚合入口 */
pub struct StudioManager {
    registry: Option<Registry>,
    storage: Option<Storage>,
    assembler: Option<crate::core::assembler::AgentAssembler>,
}

impl StudioManager {
    pub fn new(
        registry: Option<Registry>,
        storage: Option<Storage>,
        assembler: Option<crate::core::assembler::AgentAssembler>,
    ) -> Self {
        StudioManager {
            registry,
            storage,
            assembler,
        }
    }

    pub fn list_components(&self, type_filter: Option<&str>) -> Vec<ComponentSummary> {
        let mut components = Vec::new();
        if let Some(ref storage) = self.storage {
            if let Ok(agents) = storage.list_agents() {
                for agent in agents {
                    if let Some(filter) = type_filter {
                        if agent.component_type != filter {
                            continue;
                        }
                    }
                    components.push(ComponentSummary {
                        id: agent.id,
                        name: agent.name,
                        component_type: agent.component_type,
                        status: agent.status,
                        trust_score: agent.trust_score,
                    });
                }
            }
        }
        components
    }

    pub fn list_templates(&self) -> Vec<crate::core::registry::AgentTemplate> {
        self.registry
            .as_ref()
            .map(|r| r.list_templates().into_iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn get_component(&self, id: &str) -> Result<ComponentDetail, MornError> {
        if let Some(ref storage) = self.storage {
            if let Some(agent) = storage.get_agent(id)? {
                return Ok(ComponentDetail {
                    id: agent.id,
                    name: agent.name,
                    component_type: agent.component_type,
                    config_json: agent.config_json,
                    status: agent.status,
                    trust_score: agent.trust_score,
                });
            }
        }
        Err(MornError::Internal(format!("Component {} not found", id)))
    }

    pub fn create_component(&self, def: CreateComponentDef) -> Result<String, MornError> {
        let id = format!("comp-{}", uuid::Uuid::new_v4());
        let now = chrono::Utc::now().to_rfc3339();
        if let Some(ref storage) = self.storage {
            storage.insert_agent(&crate::core::storage::AgentRecord {
                id: id.clone(),
                name: def.name,
                component_type: def.component_type,
                config_json: def.config_json,
                status: "inactive".into(),
                trust_score: 70.0,
                created_at: now,
                updated_at: None,
                current_version: "0.1.0".into(),
                update_available: false,
            })?;
        }
        Ok(id)
    }

    pub fn update_component(&self, id: &str, def: UpdateComponentDef) -> Result<(), MornError> {
        if let Some(ref storage) = self.storage {
            if let Some(mut agent) = storage.get_agent(id)? {
                if let Some(name) = def.name {
                    agent.name = name;
                }
                if let Some(config) = def.config_json {
                    agent.config_json = Some(config);
                }
                if let Some(status) = def.status {
                    storage.update_agent_status(id, &status)?;
                }
            }
        }
        Ok(())
    }

    pub fn delete_component(&self, id: &str) -> Result<(), MornError> {
        if let Some(ref storage) = self.storage {
            storage.delete_agent(id)?;
        }
        Ok(())
    }

    pub fn test_component(
        &self,
        id: &str,
        input: crate::core::component::Data,
        component_type: Option<&str>,
    ) -> Result<crate::studio::tester::TestResult, MornError> {
        let tester = crate::studio::tester::StudioTester::new();
        let ctype = component_type.unwrap_or("agent");
        let config = "";
        Ok(tester.run_test(ctype, id, &input, config))
    }

    pub fn rerun_component_step(
        &self,
        component_type: &str,
        component_id: &str,
        step_index: usize,
        new_input: &str,
    ) -> Result<crate::studio::tester::TestStep, MornError> {
        let tester = crate::studio::tester::StudioTester::new();
        Ok(tester.rerun_step(component_type, component_id, step_index, new_input))
    }

    pub fn assemble_agent(
        &self,
        def: crate::core::assembler::AgentDef,
    ) -> Result<String, MornError> {
        if let Some(ref assembler) = self.assembler {
            let component = assembler.assemble(def)?;
            let agent_id = component.id().to_string();
            let now = chrono::Utc::now().to_rfc3339();
            if let Some(ref storage) = self.storage {
                storage.insert_agent(&crate::core::storage::AgentRecord {
                    id: agent_id.clone(),
                    name: "Assembled Agent".into(),
                    component_type: "agent".into(),
                    config_json: Some("{}".into()),
                    status: "active".into(),
                    trust_score: 70.0,
                    created_at: now,
                    updated_at: None,
                    current_version: "0.1.0".into(),
                    update_available: false,
                })?;
            }
            Ok(agent_id)
        } else {
            Err("Assembler not available".into())
        }
    }

    pub fn publish_to_workbench(&self, _id: &str) -> Result<(), MornError> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::component::Data;
    use crate::core::storage::Storage;

    fn manager_with_storage() -> StudioManager {
        StudioManager::new(None, Some(Storage::new_in_memory().unwrap()), None)
    }

    #[test]
    fn create_component_stores_inactive_agent() {
        let manager = manager_with_storage();

        let id = manager
            .create_component(CreateComponentDef {
                name: "Workspace Agent".into(),
                component_type: "agent".into(),
                config_json: Some(r#"{"space":"studio"}"#.into()),
            })
            .unwrap();

        let detail = manager.get_component(&id).unwrap();
        assert_eq!(detail.name, "Workspace Agent");
        assert_eq!(detail.component_type, "agent");
        assert_eq!(detail.status, "inactive");
        assert_eq!(detail.config_json, Some(r#"{"space":"studio"}"#.into()));
    }

    #[test]
    fn list_components_filters_by_type() {
        let manager = manager_with_storage();
        manager
            .create_component(CreateComponentDef {
                name: "Agent".into(),
                component_type: "agent".into(),
                config_json: None,
            })
            .unwrap();
        manager
            .create_component(CreateComponentDef {
                name: "Tool".into(),
                component_type: "tool".into(),
                config_json: None,
            })
            .unwrap();

        let tools = manager.list_components(Some("tool"));

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].component_type, "tool");
    }

    #[test]
    fn update_component_changes_status() {
        let manager = manager_with_storage();
        let id = manager
            .create_component(CreateComponentDef {
                name: "Session Agent".into(),
                component_type: "agent".into(),
                config_json: None,
            })
            .unwrap();

        manager
            .update_component(
                &id,
                UpdateComponentDef {
                    name: Some("Renamed".into()),
                    config_json: Some(r#"{"session":"active"}"#.into()),
                    status: Some("active".into()),
                },
            )
            .unwrap();

        let detail = manager.get_component(&id).unwrap();
        assert_eq!(detail.status, "active");
    }

    #[test]
    fn missing_component_returns_error() {
        let manager = manager_with_storage();

        let err = manager.get_component("missing").unwrap_err();

        assert!(err.contains("Component missing not found"));
    }

    #[test]
    fn test_component_runs_studio_tester() {
        let manager = StudioManager::new(None, None, None);
        let result = manager
            .test_component("agent-1", Data::text("hello"), Some("agent"))
            .unwrap();

        assert!(!result.steps.is_empty());
        assert!(result.output.contains("LLM response"));
    }
}
