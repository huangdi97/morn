//! assembler — Builds agents from persona, model, skill, and tool components.
use crate::component::model::ModelConfig;
use crate::component::persona::Persona;
use crate::core::assembly::AssemblyBuilder;
use crate::core::component::Component;
use crate::core::error::MornError;
use crate::core::registry::Registry;

#[derive(Debug, Clone)]
pub struct AgentDef {
    pub id: String,
    pub name: String,
    pub persona: Persona,
    pub model: ModelConfig,
    pub tools: Vec<String>,
    pub knowledge: Vec<String>,
    pub skills: Vec<String>,
    pub memory: Option<String>,
}

#[derive(Debug, Clone)]
pub enum AfterBuildAction {
    Save(AgentDef),
    Modify(String, serde_json::Value),
    Preview(AgentDef),
}

#[allow(dead_code)] /* 预留：agent 装配器 registry 注入 */
pub struct AgentAssembler {
    registry: Option<Registry>,
    assembly_builder: Option<AssemblyBuilder>,
}

impl AgentAssembler {
    pub fn new(registry: Option<Registry>) -> Self {
        AgentAssembler {
            registry,
            assembly_builder: Some(AssemblyBuilder::new()),
        }
    }

    pub fn assemble(&self, def: AgentDef) -> Result<Box<dyn Component>, MornError> {
        let agent_id = def.id.clone();
        let _agent_name = def.name.clone();
        let _persona = def.persona;
        let _model = def.model;

        #[allow(dead_code)] /* 预留：装配后的轻量 agent 占位实现 */
        struct AssembledAgent {
            id: String,
            name: String,
        }

        impl Component for AssembledAgent {
            fn id(&self) -> &str {
                &self.id
            }
            fn type_name(&self) -> &str {
                "agent"
            }
            fn init(&mut self) -> Result<(), MornError> {
                Ok(())
            }
            fn run(&mut self) -> Result<(), MornError> {
                Ok(())
            }
            fn pause(&mut self) -> Result<(), MornError> {
                Ok(())
            }
            fn stop(&mut self) -> Result<(), MornError> {
                Ok(())
            }
            fn health_check(&self) -> crate::core::component::HealthStatus {
                crate::core::component::HealthStatus::Healthy
            }
        }

        impl crate::core::component::IOComponent for AssembledAgent {
            fn ports(&self) -> Vec<crate::core::component::Port> {
                vec![
                    crate::core::component::Port {
                        id: "input".into(),
                        direction: crate::core::component::PortDirection::Input,
                        data_type: "text".into(),
                        description: "user input".into(),
                    },
                    crate::core::component::Port {
                        id: "output".into(),
                        direction: crate::core::component::PortDirection::Output,
                        data_type: "text".into(),
                        description: "agent response".into(),
                    },
                ]
            }
            fn send(
                &mut self,
                _port: &str,
                _data: crate::core::component::Data,
            ) -> Result<(), MornError> {
                Ok(())
            }
            fn recv(
                &mut self,
                _port: &str,
            ) -> Result<Option<crate::core::component::Data>, MornError> {
                Ok(None)
            }
        }

        impl crate::core::component::SecureComponent for AssembledAgent {
            fn required_permissions(&self) -> Vec<crate::core::component::Permission> {
                vec![]
            }
        }

        Ok(Box::new(AssembledAgent {
            id: agent_id,
            name: _agent_name,
        }))
    }

    pub fn natural_language_build(description: &str) -> Result<AfterBuildAction, MornError> {
        let desc_lower = description.to_lowercase();
        let (persona, persona_id) = if desc_lower.contains("biology")
            || desc_lower.contains("research")
            || desc_lower.contains("science")
        {
            (
                crate::component::persona::create_researcher_persona(),
                "researcher",
            )
        } else if desc_lower.contains("write")
            || desc_lower.contains("content")
            || desc_lower.contains("blog")
        {
            (crate::component::persona::create_writer_persona(), "writer")
        } else if desc_lower.contains("code")
            || desc_lower.contains("program")
            || desc_lower.contains("develop")
        {
            (crate::component::persona::create_coder_persona(), "coder")
        } else if desc_lower.contains("analyst")
            || desc_lower.contains("analyze")
            || desc_lower.contains("data")
        {
            (
                crate::component::persona::create_analyst_persona(),
                "analyst",
            )
        } else {
            (
                crate::component::persona::create_assistant_persona(),
                "assistant",
            )
        };

        let mut tools = Vec::new();
        if desc_lower.contains("search") || desc_lower.contains("web") {
            tools.push("web_search".into());
        }
        if desc_lower.contains("file") || desc_lower.contains("read") {
            tools.push("read_file".into());
        }
        if desc_lower.contains("write") || desc_lower.contains("create") {
            tools.push("write_file".into());
        }
        if tools.is_empty() {
            tools.push("web_search".into());
            tools.push("read_file".into());
        }

        let model_id = format!("model-{}-default", persona_id);
        let model = crate::component::model::ModelConfig {
            id: model_id.clone(),
            provider: "deepseek".into(),
            model_name: "deepseek-chat".into(),
            base_url: "https://api.deepseek.com".into(),
            api_key: String::new(),
            parameters: crate::component::model::ModelParameters::default(),
            fallback: None,
            cost_tier: crate::component::model::CostTier::Low,
        };

        let def = AgentDef {
            id: format!("agent-{}", uuid::Uuid::new_v4()),
            name: description
                .split(|c: char| !c.is_alphanumeric() && c != ' ')
                .take(3)
                .collect::<Vec<_>>()
                .join(" "),
            persona,
            model,
            tools,
            knowledge: vec![],
            skills: vec![],
            memory: None,
        };

        Ok(after_build_choices(def))
    }
}

fn after_build_choices(def: AgentDef) -> AfterBuildAction {
    AfterBuildAction::Preview(def)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::component::model::{CostTier, ModelParameters};

    fn model_config() -> ModelConfig {
        ModelConfig {
            id: "model-test".to_string(),
            provider: "local".to_string(),
            model_name: "test-model".to_string(),
            base_url: "http://localhost".to_string(),
            api_key: String::new(),
            parameters: ModelParameters::default(),
            fallback: None,
            cost_tier: CostTier::Low,
        }
    }

    fn agent_def(id: &str, name: &str) -> AgentDef {
        AgentDef {
            id: id.to_string(),
            name: name.to_string(),
            persona: Persona::new("persona-test", "Test Persona"),
            model: model_config(),
            tools: vec!["web_search".to_string()],
            knowledge: vec![],
            skills: vec!["summarization".to_string()],
            memory: None,
        }
    }

    fn preview_def(action: AfterBuildAction) -> AgentDef {
        match action {
            AfterBuildAction::Preview(def) => def,
            _ => panic!("expected preview action"),
        }
    }

    #[test]
    fn assembles_single_agent_component() {
        let assembler = AgentAssembler::new(None);

        let component = assembler
            .assemble(agent_def("agent-1", "Agent One"))
            .unwrap();

        assert_eq!(component.id(), "agent-1");
        assert_eq!(component.type_name(), "agent");
        assert_eq!(
            component.health_check(),
            crate::core::component::HealthStatus::Healthy
        );
    }

    #[test]
    fn assembles_multiple_agent_defs_for_team_like_use() {
        let assembler = AgentAssembler::new(None);
        let analyst = assembler
            .assemble(agent_def("agent-analyst", "Analyst"))
            .unwrap();
        let writer = assembler
            .assemble(agent_def("agent-writer", "Writer"))
            .unwrap();

        let team = [analyst.id().to_string(), writer.id().to_string()];

        assert_eq!(
            team,
            ["agent-analyst".to_string(), "agent-writer".to_string()]
        );
    }

    #[test]
    fn natural_language_build_validates_basic_config() {
        let def =
            AgentAssembler::natural_language_build("build a code review agent that can read files")
                .unwrap();
        let def = preview_def(def);

        assert!(def.id.starts_with("agent-"));
        assert_eq!(def.persona.id, "persona-coder");
        assert_eq!(def.model.model_name, "deepseek-chat");
        assert!(def.tools.contains(&"read_file".to_string()));
    }

    #[test]
    fn natural_language_build_defaults_missing_config() {
        let def = preview_def(AgentAssembler::natural_language_build("").unwrap());

        assert_eq!(def.persona.id, "persona-assistant");
        assert!(def.tools.contains(&"web_search".to_string()));
        assert!(def.tools.contains(&"read_file".to_string()));
    }
}
