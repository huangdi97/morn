//! Graph/topology types and logic for component assembly.

use crate::core::error::MornError;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum AtomicComponentType {
    Memory,
    Tool,
    LLM,
    Channel,
    Persona,
    Skill,
    Knowledge,
    SecurityPolicy,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AtomicComponentDef {
    pub id: String,
    pub component_type: AtomicComponentType,
    pub type_name: String,
    pub name: String,
    pub description: String,
    pub input_types: Vec<String>,
    pub output_types: Vec<String>,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ComponentConnection {
    pub source_id: String,
    pub source_output: String,
    pub target_id: String,
    pub target_input: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ComponentGraph {
    pub components: Vec<AtomicComponentDef>,
    pub connections: Vec<ComponentConnection>,
}

impl AtomicComponentType {
    pub fn type_name(&self) -> &'static str {
        match self {
            AtomicComponentType::Memory => "memory",
            AtomicComponentType::Tool => "tool",
            AtomicComponentType::LLM => "model",
            AtomicComponentType::Channel => "channel",
            AtomicComponentType::Persona => "persona",
            AtomicComponentType::Skill => "skill",
            AtomicComponentType::Knowledge => "knowledge",
            AtomicComponentType::SecurityPolicy => "security_policy",
        }
    }
}
impl ComponentGraph {
    pub fn to_json(&self) -> Result<String, MornError> {
        serde_json::to_string_pretty(self).map_err(|e| MornError::Internal(format!("serialization error: {}", e)))
    }

    pub fn from_json(json: &str) -> Result<Self, MornError> {
        serde_json::from_str(json).map_err(|e| MornError::Internal(format!("deserialization error: {}", e)))
    }
}

pub struct ConnectionValidator;

impl ConnectionValidator {
    pub fn validate(graph: &ComponentGraph) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let comp_map: HashMap<&str, &AtomicComponentDef> = graph
            .components
            .iter()
            .map(|c| (c.id.as_str(), c))
            .collect();

        for conn in &graph.connections {
            let source = match comp_map.get(conn.source_id.as_str()) {
                Some(c) => c,
                None => {
                    errors.push(format!("source component '{}' not found", conn.source_id));
                    continue;
                }
            };
            let target = match comp_map.get(conn.target_id.as_str()) {
                Some(c) => c,
                None => {
                    errors.push(format!("target component '{}' not found", conn.target_id));
                    continue;
                }
            };

            let output_type = conn.source_output.as_str();
            let input_type = conn.target_input.as_str();

            let has_output = source.output_types.is_empty()
                || source.output_types.iter().any(|t| t == output_type);
            let has_input =
                target.input_types.is_empty() || target.input_types.iter().any(|t| t == input_type);

            if !has_output {
                errors.push(format!(
                    "source '{}' does not have output type '{}'",
                    source.name, output_type
                ));
            }
            if !has_input {
                errors.push(format!(
                    "target '{}' does not have input type '{}'",
                    target.name, input_type
                ));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub struct ComponentRegistry;

impl ComponentRegistry {
    pub fn available_components() -> Vec<AtomicComponentDef> {
        vec![
            AtomicComponentDef {
                id: "mem_working".into(),
                component_type: AtomicComponentType::Memory,
                type_name: AtomicComponentType::Memory.type_name().into(),
                name: "Working Memory".into(),
                description: "Short-term working memory for active context".into(),
                input_types: vec!["write".into()],
                output_types: vec!["read".into()],
                config: serde_json::json!({"capacity": 4096}),
            },
            AtomicComponentDef {
                id: "mem_long_term".into(),
                component_type: AtomicComponentType::Memory,
                type_name: AtomicComponentType::Memory.type_name().into(),
                name: "Long-term Memory".into(),
                description: "Persistent long-term storage".into(),
                input_types: vec!["store".into()],
                output_types: vec!["recall".into()],
                config: serde_json::json!({"index": "vector"}),
            },
            AtomicComponentDef {
                id: "tool_web_search".into(),
                component_type: AtomicComponentType::Tool,
                type_name: AtomicComponentType::Tool.type_name().into(),
                name: "Web Search".into(),
                description: "Search the web for information".into(),
                input_types: vec!["query".into()],
                output_types: vec!["results".into()],
                config: serde_json::json!({"engine": "duckduckgo"}),
            },
            AtomicComponentDef {
                id: "tool_file_read".into(),
                component_type: AtomicComponentType::Tool,
                type_name: AtomicComponentType::Tool.type_name().into(),
                name: "File Reader".into(),
                description: "Read local files".into(),
                input_types: vec!["path".into()],
                output_types: vec!["content".into()],
                config: serde_json::json!({"max_size": 1048576}),
            },
            AtomicComponentDef {
                id: "tool_code_exec".into(),
                component_type: AtomicComponentType::Tool,
                type_name: AtomicComponentType::Tool.type_name().into(),
                name: "Code Executor".into(),
                description: "Execute code in sandbox".into(),
                input_types: vec!["code".into()],
                output_types: vec!["output".into(), "error".into()],
                config: serde_json::json!({"timeout": 30}),
            },
            AtomicComponentDef {
                id: "llm_deepseek".into(),
                component_type: AtomicComponentType::LLM,
                type_name: AtomicComponentType::LLM.type_name().into(),
                name: "DeepSeek Chat".into(),
                description: "DeepSeek chat model".into(),
                input_types: vec!["prompt".into()],
                output_types: vec!["response".into()],
                config: serde_json::json!({"model": "deepseek-chat", "temperature": 0.7}),
            },
            AtomicComponentDef {
                id: "llm_gpt4".into(),
                component_type: AtomicComponentType::LLM,
                type_name: AtomicComponentType::LLM.type_name().into(),
                name: "GPT-4o".into(),
                description: "OpenAI GPT-4o model".into(),
                input_types: vec!["prompt".into()],
                output_types: vec!["response".into()],
                config: serde_json::json!({"model": "gpt-4o", "temperature": 0.7}),
            },
            AtomicComponentDef {
                id: "ch_telegram".into(),
                component_type: AtomicComponentType::Channel,
                type_name: AtomicComponentType::Channel.type_name().into(),
                name: "Telegram".into(),
                description: "Telegram messaging channel".into(),
                input_types: vec!["send".into()],
                output_types: vec!["receive".into()],
                config: serde_json::json!({"bot_token": ""}),
            },
            AtomicComponentDef {
                id: "ch_desktop".into(),
                component_type: AtomicComponentType::Channel,
                type_name: AtomicComponentType::Channel.type_name().into(),
                name: "Desktop".into(),
                description: "Desktop notification channel".into(),
                input_types: vec!["notify".into()],
                output_types: vec!["event".into()],
                config: serde_json::json!({}),
            },
            AtomicComponentDef {
                id: "persona_assistant".into(),
                component_type: AtomicComponentType::Persona,
                type_name: AtomicComponentType::Persona.type_name().into(),
                name: "Assistant".into(),
                description: "Default helpful assistant persona".into(),
                input_types: vec![],
                output_types: vec!["character".into()],
                config: serde_json::json!({"style": "helpful"}),
            },
            AtomicComponentDef {
                id: "persona_researcher".into(),
                component_type: AtomicComponentType::Persona,
                type_name: AtomicComponentType::Persona.type_name().into(),
                name: "Researcher".into(),
                description: "Deep research-oriented persona".into(),
                input_types: vec![],
                output_types: vec!["character".into()],
                config: serde_json::json!({"style": "analytical"}),
            },
            AtomicComponentDef {
                id: "skill_analysis".into(),
                component_type: AtomicComponentType::Skill,
                type_name: AtomicComponentType::Skill.type_name().into(),
                name: "Data Analysis".into(),
                description: "Data analysis and visualization skill".into(),
                input_types: vec!["data".into()],
                output_types: vec!["insights".into()],
                config: serde_json::json!({"tools": ["python", "sql"]}),
            },
            AtomicComponentDef {
                id: "knowledge_docs".into(),
                component_type: AtomicComponentType::Knowledge,
                type_name: AtomicComponentType::Knowledge.type_name().into(),
                name: "Document Knowledge".into(),
                description: "Document-based knowledge base".into(),
                input_types: vec!["query".into()],
                output_types: vec!["answer".into()],
                config: serde_json::json!({"source": "internal_docs"}),
            },
            AtomicComponentDef {
                id: "sec_policy_default".into(),
                component_type: AtomicComponentType::SecurityPolicy,
                type_name: AtomicComponentType::SecurityPolicy.type_name().into(),
                name: "Default Security Policy".into(),
                description: "Baseline security policy for access control".into(),
                input_types: vec!["request".into()],
                output_types: vec!["decision".into()],
                config: serde_json::json!({"allow_list": ["read", "write"]}),
            },
        ]
    }

    pub fn get_component(id: &str) -> Option<AtomicComponentDef> {
        Self::available_components()
            .into_iter()
            .find(|c| c.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_graph_to_json() {
        let graph = ComponentGraph {
            components: vec![],
            connections: vec![],
        };
        let json = graph.to_json().unwrap();
        assert!(json.contains("\"components\""));
        assert!(json.contains("\"connections\""));
    }

    #[test]
    fn test_component_graph_from_json() {
        let json = r#"{"components":[{"id":"test","component_type":"Memory","type_name":"memory","name":"Test","description":"","input_types":[],"output_types":[],"config":{}}],"connections":[]}"#;
        let graph = ComponentGraph::from_json(json).unwrap();
        assert_eq!(graph.components.len(), 1);
        assert_eq!(graph.components[0].id, "test");
    }

    #[test]
    fn test_component_graph_from_json_invalid() {
        let result = ComponentGraph::from_json("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_connection_validator_valid() {
        let graph = ComponentGraph {
            components: vec![
                AtomicComponentDef {
                    id: "src".into(),
                    component_type: AtomicComponentType::Tool,
                    type_name: AtomicComponentType::Tool.type_name().into(),
                    name: "Source".into(),
                    description: String::new(),
                    input_types: vec![],
                    output_types: vec!["out".into()],
                    config: serde_json::json!({}),
                },
                AtomicComponentDef {
                    id: "dst".into(),
                    component_type: AtomicComponentType::Memory,
                    type_name: AtomicComponentType::Memory.type_name().into(),
                    name: "Dest".into(),
                    description: String::new(),
                    input_types: vec!["in".into()],
                    output_types: vec![],
                    config: serde_json::json!({}),
                },
            ],
            connections: vec![ComponentConnection {
                source_id: "src".into(),
                source_output: "out".into(),
                target_id: "dst".into(),
                target_input: "in".into(),
            }],
        };
        assert!(ConnectionValidator::validate(&graph).is_ok());
    }

    #[test]
    fn test_connection_validator_source_not_found() {
        let graph = ComponentGraph {
            components: vec![],
            connections: vec![ComponentConnection {
                source_id: "missing".into(),
                source_output: "out".into(),
                target_id: "dst".into(),
                target_input: "in".into(),
            }],
        };
        let result = ConnectionValidator::validate(&graph);
        assert!(result.is_err());
        assert!(result.unwrap_err().iter().any(|e| e.contains("not found")));
    }

    #[test]
    fn test_connection_validator_output_type_mismatch() {
        let graph = ComponentGraph {
            components: vec![
                AtomicComponentDef {
                    id: "src".into(),
                    component_type: AtomicComponentType::Tool,
                    type_name: AtomicComponentType::Tool.type_name().into(),
                    name: "Source".into(),
                    description: String::new(),
                    input_types: vec![],
                    output_types: vec!["num".into()],
                    config: serde_json::json!({}),
                },
                AtomicComponentDef {
                    id: "dst".into(),
                    component_type: AtomicComponentType::Memory,
                    type_name: AtomicComponentType::Memory.type_name().into(),
                    name: "Dest".into(),
                    description: String::new(),
                    input_types: vec!["text".into()],
                    output_types: vec![],
                    config: serde_json::json!({}),
                },
            ],
            connections: vec![ComponentConnection {
                source_id: "src".into(),
                source_output: "text".into(),
                target_id: "dst".into(),
                target_input: "num".into(),
            }],
        };
        let result = ConnectionValidator::validate(&graph);
        assert!(result.is_err());
    }

    #[test]
    fn test_connection_validator_empty_types_accepts_any() {
        let graph = ComponentGraph {
            components: vec![
                AtomicComponentDef {
                    id: "src".into(),
                    component_type: AtomicComponentType::Tool,
                    type_name: AtomicComponentType::Tool.type_name().into(),
                    name: "Source".into(),
                    description: String::new(),
                    input_types: vec![],
                    output_types: vec![],
                    config: serde_json::json!({}),
                },
                AtomicComponentDef {
                    id: "dst".into(),
                    component_type: AtomicComponentType::Memory,
                    type_name: AtomicComponentType::Memory.type_name().into(),
                    name: "Dest".into(),
                    description: String::new(),
                    input_types: vec![],
                    output_types: vec![],
                    config: serde_json::json!({}),
                },
            ],
            connections: vec![ComponentConnection {
                source_id: "src".into(),
                source_output: "any".into(),
                target_id: "dst".into(),
                target_input: "any".into(),
            }],
        };
        assert!(ConnectionValidator::validate(&graph).is_ok());
    }

    #[test]
    fn test_component_registry_available_count() {
        let components = ComponentRegistry::available_components();
        assert_eq!(components.len(), 14);
    }

    #[test]
    fn test_component_registry_contains_expected_ids() {
        let components = ComponentRegistry::available_components();
        let ids: Vec<&str> = components.iter().map(|c| c.id.as_str()).collect();
        assert!(ids.contains(&"mem_working"));
        assert!(ids.contains(&"tool_web_search"));
        assert!(ids.contains(&"llm_deepseek"));
        assert!(ids.contains(&"persona_assistant"));
        assert!(ids.contains(&"ch_telegram"));
    }

    #[test]
    fn test_component_registry_get_component_found() {
        let comp = ComponentRegistry::get_component("mem_working");
        assert!(comp.is_some());
        assert_eq!(comp.unwrap().name, "Working Memory");
    }

    #[test]
    fn test_component_registry_get_component_not_found() {
        let comp = ComponentRegistry::get_component("nonexistent");
        assert!(comp.is_none());
    }

    #[test]
    fn test_atomic_component_type_serialization() {
        let json = serde_json::to_string(&AtomicComponentType::Memory).unwrap();
        assert_eq!(json, "\"Memory\"");
        let deserialized: AtomicComponentType = serde_json::from_str("\"Tool\"").unwrap();
        assert_eq!(deserialized, AtomicComponentType::Tool);
    }

    #[test]
    fn test_component_graph_roundtrip() {
        let graph = ComponentGraph {
            components: vec![AtomicComponentDef {
                id: "comp1".into(),
                component_type: AtomicComponentType::Memory,
                type_name: AtomicComponentType::Memory.type_name().into(),
                name: "Comp1".into(),
                description: "desc".into(),
                input_types: vec!["in".into()],
                output_types: vec!["out".into()],
                config: serde_json::json!({"key": "val"}),
            }],
            connections: vec![ComponentConnection {
                source_id: "comp1".into(),
                source_output: "out".into(),
                target_id: "comp1".into(),
                target_input: "in".into(),
            }],
        };
        let json = graph.to_json().unwrap();
        let restored = ComponentGraph::from_json(&json).unwrap();
        assert_eq!(restored.components.len(), 1);
        assert_eq!(restored.connections.len(), 1);
        assert_eq!(restored.components[0].config["key"], "val");
    }
}
