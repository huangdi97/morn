use crate::component::model::ModelConfig;
use crate::component::persona::Persona;
use crate::core::component::Component;
use crate::core::registry::Registry;

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

#[allow(dead_code)]
pub struct AgentAssembler {
    registry: Option<Registry>,
}

impl AgentAssembler {
    pub fn new(registry: Option<Registry>) -> Self {
        AgentAssembler { registry }
    }

    pub fn assemble(&self, def: AgentDef) -> Result<Box<dyn Component>, String> {
        let agent_id = def.id.clone();
        let _agent_name = def.name.clone();
        let _persona = def.persona;
        let _model = def.model;

        #[allow(dead_code)]
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
            fn init(&mut self) -> Result<(), String> {
                Ok(())
            }
            fn run(&mut self) -> Result<(), String> {
                Ok(())
            }
            fn pause(&mut self) -> Result<(), String> {
                Ok(())
            }
            fn stop(&mut self) -> Result<(), String> {
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
            ) -> Result<(), String> {
                Ok(())
            }
            fn recv(
                &mut self,
                _port: &str,
            ) -> Result<Option<crate::core::component::Data>, String> {
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

    pub fn natural_language_build(description: &str) -> Result<AgentDef, String> {
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

        Ok(AgentDef {
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
        })
    }
}
