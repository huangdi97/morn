use crate::core::component::Component;
use crate::core::event_bus::SimpleEventBus;
use crate::core::registry::Registry;
use crate::core::storage::Storage;

#[derive(Debug, Clone)]
pub struct ComponentSummary {
    pub id: String,
    pub name: String,
    pub component_type: String,
    pub status: String,
    pub trust_score: f64,
}

#[derive(Debug, Clone)]
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

    pub fn get_component(&self, id: &str) -> Result<ComponentDetail, String> {
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
        Err(format!("Component {} not found", id))
    }

    pub fn create_component(&self, def: CreateComponentDef) -> Result<String, String> {
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
            })?;
        }
        Ok(id)
    }

    pub fn update_component(&self, id: &str, def: UpdateComponentDef) -> Result<(), String> {
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

    pub fn delete_component(&self, id: &str) -> Result<(), String> {
        if let Some(ref storage) = self.storage {
            storage.delete_agent(id)?;
        }
        Ok(())
    }

    pub fn test_component(
        &self,
        _id: &str,
        _input: crate::core::component::Data,
    ) -> Result<crate::studio::tester::TestResult, String> {
        Ok(crate::studio::tester::TestResult {
            steps: vec![],
            total_duration_ms: 0.0,
            total_tokens: 0,
            total_cost: 0.0,
            output: "test output".into(),
        })
    }

    pub fn assemble_agent(&self, def: crate::core::assembler::AgentDef) -> Result<String, String> {
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
                })?;
            }
            Ok(agent_id)
        } else {
            Err("Assembler not available".into())
        }
    }

    pub fn publish_to_workbench(&self, _id: &str) -> Result<(), String> {
        Ok(())
    }
}
