//! Graph/topology types and logic for component assembly.

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

impl ComponentGraph {
    pub fn to_json(&self) -> Result<String, String> {
        serde_json::to_string_pretty(self).map_err(|e| format!("serialization error: {}", e))
    }

    pub fn from_json(json: &str) -> Result<Self, String> {
        serde_json::from_str(json).map_err(|e| format!("deserialization error: {}", e))
    }
}

pub struct ConnectionValidator;

impl ConnectionValidator {
    pub fn validate(graph: &ComponentGraph) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        let comp_map: HashMap<&str, &AtomicComponentDef> =
            graph.components.iter().map(|c| (c.id.as_str(), c)).collect();

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
            let has_input = target.input_types.is_empty()
                || target.input_types.iter().any(|t| t == input_type);

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
                name: "Working Memory".into(),
                description: "Short-term working memory for active context".into(),
                input_types: vec!["write".into()],
                output_types: vec!["read".into()],
                config: serde_json::json!({"capacity": 4096}),
            },
            AtomicComponentDef {
                id: "mem_long_term".into(),
                component_type: AtomicComponentType::Memory,
                name: "Long-term Memory".into(),
                description: "Persistent long-term storage".into(),
                input_types: vec!["store".into()],
                output_types: vec!["recall".into()],
                config: serde_json::json!({"index": "vector"}),
            },
            AtomicComponentDef {
                id: "tool_web_search".into(),
                component_type: AtomicComponentType::Tool,
                name: "Web Search".into(),
                description: "Search the web for information".into(),
                input_types: vec!["query".into()],
                output_types: vec!["results".into()],
                config: serde_json::json!({"engine": "duckduckgo"}),
            },
            AtomicComponentDef {
                id: "tool_file_read".into(),
                component_type: AtomicComponentType::Tool,
                name: "File Reader".into(),
                description: "Read local files".into(),
                input_types: vec!["path".into()],
                output_types: vec!["content".into()],
                config: serde_json::json!({"max_size": 1048576}),
            },
            AtomicComponentDef {
                id: "tool_code_exec".into(),
                component_type: AtomicComponentType::Tool,
                name: "Code Executor".into(),
                description: "Execute code in sandbox".into(),
                input_types: vec!["code".into()],
                output_types: vec!["output".into(), "error".into()],
                config: serde_json::json!({"timeout": 30}),
            },
            AtomicComponentDef {
                id: "llm_deepseek".into(),
                component_type: AtomicComponentType::LLM,
                name: "DeepSeek Chat".into(),
                description: "DeepSeek chat model".into(),
                input_types: vec!["prompt".into()],
                output_types: vec!["response".into()],
                config: serde_json::json!({"model": "deepseek-chat", "temperature": 0.7}),
            },
            AtomicComponentDef {
                id: "llm_gpt4".into(),
                component_type: AtomicComponentType::LLM,
                name: "GPT-4o".into(),
                description: "OpenAI GPT-4o model".into(),
                input_types: vec!["prompt".into()],
                output_types: vec!["response".into()],
                config: serde_json::json!({"model": "gpt-4o", "temperature": 0.7}),
            },
            AtomicComponentDef {
                id: "ch_telegram".into(),
                component_type: AtomicComponentType::Channel,
                name: "Telegram".into(),
                description: "Telegram messaging channel".into(),
                input_types: vec!["send".into()],
                output_types: vec!["receive".into()],
                config: serde_json::json!({"bot_token": ""}),
            },
            AtomicComponentDef {
                id: "ch_desktop".into(),
                component_type: AtomicComponentType::Channel,
                name: "Desktop".into(),
                description: "Desktop notification channel".into(),
                input_types: vec!["notify".into()],
                output_types: vec!["event".into()],
                config: serde_json::json!({}),
            },
            AtomicComponentDef {
                id: "persona_assistant".into(),
                component_type: AtomicComponentType::Persona,
                name: "Assistant".into(),
                description: "Default helpful assistant persona".into(),
                input_types: vec![],
                output_types: vec!["character".into()],
                config: serde_json::json!({"style": "helpful"}),
            },
            AtomicComponentDef {
                id: "persona_researcher".into(),
                component_type: AtomicComponentType::Persona,
                name: "Researcher".into(),
                description: "Deep research-oriented persona".into(),
                input_types: vec![],
                output_types: vec!["character".into()],
                config: serde_json::json!({"style": "analytical"}),
            },
            AtomicComponentDef {
                id: "skill_analysis".into(),
                component_type: AtomicComponentType::Skill,
                name: "Data Analysis".into(),
                description: "Data analysis and visualization skill".into(),
                input_types: vec!["data".into()],
                output_types: vec!["insights".into()],
                config: serde_json::json!({"tools": ["python", "sql"]}),
            },
            AtomicComponentDef {
                id: "knowledge_docs".into(),
                component_type: AtomicComponentType::Knowledge,
                name: "Document Knowledge".into(),
                description: "Document-based knowledge base".into(),
                input_types: vec!["query".into()],
                output_types: vec!["answer".into()],
                config: serde_json::json!({"source": "internal_docs"}),
            },
            AtomicComponentDef {
                id: "sec_policy_default".into(),
                component_type: AtomicComponentType::SecurityPolicy,
                name: "Default Security Policy".into(),
                description: "Baseline security policy for access control".into(),
                input_types: vec!["request".into()],
                output_types: vec!["decision".into()],
                config: serde_json::json!({"allow_list": ["read", "write"]}),
            },
        ]
    }

    pub fn get_component(id: &str) -> Option<AtomicComponentDef> {
        Self::available_components().into_iter().find(|c| c.id == id)
    }
}