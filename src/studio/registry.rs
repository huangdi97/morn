//! Node registry and template catalog: all available node templates organized by category.

use crate::core::error::MornError;
use super::types::{NodeTemplate, NodeType};
use crate::core::registry::{Capability, Registry};

pub struct NodeRegistry;

impl NodeRegistry {
    pub fn all_templates() -> Vec<NodeTemplate> {
        vec![
            NodeTemplate {
                node_type: NodeType::HttpRequest,
                label: "HTTP Request",
                description: "Make HTTP requests to external APIs",
                category: "Network",
                default_config: serde_json::json!({"url": "", "method": "GET", "headers": {}}),
                inputs: vec!["body"],
                outputs: vec!["response"],
            },
            NodeTemplate {
                node_type: NodeType::LLMCall,
                label: "LLM Call",
                description: "Send prompt to a language model",
                category: "AI",
                default_config: serde_json::json!({"model": "deepseek-chat", "prompt": "", "temperature": 0.7}),
                inputs: vec!["context"],
                outputs: vec!["response"],
            },
            NodeTemplate {
                node_type: NodeType::Condition,
                label: "Condition",
                description: "Branch execution based on condition",
                category: "Flow",
                default_config: serde_json::json!({"condition": "", "operator": "equals"}),
                inputs: vec!["value"],
                outputs: vec!["true", "false"],
            },
            NodeTemplate {
                node_type: NodeType::Loop,
                label: "Loop",
                description: "Iterate over items",
                category: "Flow",
                default_config: serde_json::json!({"max_iterations": 10}),
                inputs: vec!["items"],
                outputs: vec!["iteration"],
            },
            NodeTemplate {
                node_type: NodeType::Transform,
                label: "Transform",
                description: "Transform data with expressions",
                category: "Data",
                default_config: serde_json::json!({"expression": ""}),
                inputs: vec!["data"],
                outputs: vec!["transformed"],
            },
            NodeTemplate {
                node_type: NodeType::Merge,
                label: "Merge",
                description: "Merge multiple data streams",
                category: "Data",
                default_config: serde_json::json!({"strategy": "object_merge"}),
                inputs: vec!["source_a", "source_b"],
                outputs: vec!["merged"],
            },
            NodeTemplate {
                node_type: NodeType::Split,
                label: "Split",
                description: "Split array into individual items",
                category: "Data",
                default_config: serde_json::json!({"field": "items"}),
                inputs: vec!["data"],
                outputs: vec!["item"],
            },
            NodeTemplate {
                node_type: NodeType::HttpRequest,
                label: "GET Request",
                description: "Simple GET request",
                category: "Network",
                default_config: serde_json::json!({"url": "", "method": "GET", "headers": {}}),
                inputs: vec![],
                outputs: vec!["response"],
            },
            NodeTemplate {
                node_type: NodeType::HttpRequest,
                label: "POST Request",
                description: "POST data to API endpoint",
                category: "Network",
                default_config: serde_json::json!({"url": "", "method": "POST", "headers": {"Content-Type": "application/json"}}),
                inputs: vec!["body"],
                outputs: vec!["response"],
            },
            NodeTemplate {
                node_type: NodeType::LLMCall,
                label: "Chat Completion",
                description: "Standard chat completion call",
                category: "AI",
                default_config: serde_json::json!({"model": "gpt-4o", "prompt": "", "temperature": 0.7}),
                inputs: vec!["context"],
                outputs: vec!["response"],
            },
            NodeTemplate {
                node_type: NodeType::LLMCall,
                label: "Code Generator",
                description: "Generate code via LLM",
                category: "AI",
                default_config: serde_json::json!({"model": "deepseek-coder", "prompt": "", "temperature": 0.2}),
                inputs: vec!["context"],
                outputs: vec!["response"],
            },
            NodeTemplate {
                node_type: NodeType::Condition,
                label: "If/Else",
                description: "Standard conditional branch",
                category: "Flow",
                default_config: serde_json::json!({"condition": "", "operator": "equals"}),
                inputs: vec!["value"],
                outputs: vec!["true", "false"],
            },
            NodeTemplate {
                node_type: NodeType::Condition,
                label: "Contains Check",
                description: "Check if value contains substring",
                category: "Flow",
                default_config: serde_json::json!({"condition": "", "operator": "contains"}),
                inputs: vec!["value"],
                outputs: vec!["true", "false"],
            },
            NodeTemplate {
                node_type: NodeType::Transform,
                label: "JSON Parse",
                description: "Parse JSON string to object",
                category: "Data",
                default_config: serde_json::json!({"expression": "json_parse"}),
                inputs: vec!["data"],
                outputs: vec!["transformed"],
            },
            NodeTemplate {
                node_type: NodeType::Transform,
                label: "Base64 Encode/Decode",
                description: "Encode or decode base64",
                category: "Data",
                default_config: serde_json::json!({"expression": "base64_encode"}),
                inputs: vec!["data"],
                outputs: vec!["transformed"],
            },
            NodeTemplate {
                node_type: NodeType::Merge,
                label: "Array Concat",
                description: "Concatenate arrays",
                category: "Data",
                default_config: serde_json::json!({"strategy": "array_concat"}),
                inputs: vec!["array_a", "array_b"],
                outputs: vec!["merged"],
            },
            NodeTemplate {
                node_type: NodeType::Loop,
                label: "For Each",
                description: "Loop over array items",
                category: "Flow",
                default_config: serde_json::json!({"max_iterations": 100}),
                inputs: vec!["items"],
                outputs: vec!["iteration"],
            },
            NodeTemplate {
                node_type: NodeType::Split,
                label: "Split Text",
                description: "Split text by delimiter",
                category: "Data",
                default_config: serde_json::json!({"field": "text", "delimiter": ","}),
                inputs: vec!["data"],
                outputs: vec!["item"],
            },
            NodeTemplate {
                node_type: NodeType::HttpRequest,
                label: "GraphQL Query",
                description: "Execute GraphQL query",
                category: "Network",
                default_config: serde_json::json!({"url": "", "method": "POST", "headers": {"Content-Type": "application/json"}, "query": ""}),
                inputs: vec!["variables"],
                outputs: vec!["response"],
            },
            NodeTemplate {
                node_type: NodeType::LLMCall,
                label: "Embedding",
                description: "Generate text embeddings",
                category: "AI",
                default_config: serde_json::json!({"model": "voyage-2", "input": "", "task": "embedding"}),
                inputs: vec!["text"],
                outputs: vec!["embedding"],
            },
            NodeTemplate {
                node_type: NodeType::Code,
                label: "Python Execute",
                description: "Sandboxed Python code execution",
                category: "Flow",
                default_config: serde_json::json!({"code": "", "timeout": 30, "sandbox": true}),
                inputs: vec!["input"],
                outputs: vec!["result", "error"],
            },
            NodeTemplate {
                node_type: NodeType::Code,
                label: "JS Execute",
                description: "Sandboxed JavaScript execution",
                category: "Flow",
                default_config: serde_json::json!({"code": "", "timeout": 30, "sandbox": true}),
                inputs: vec!["input"],
                outputs: vec!["result", "error"],
            },
            NodeTemplate {
                node_type: NodeType::Trigger,
                label: "Cron Trigger",
                description: "Scheduled execution via cron expression",
                category: "Flow",
                default_config: serde_json::json!({"cron": "0 * * * *", "timezone": "UTC"}),
                inputs: vec![],
                outputs: vec!["trigger"],
            },
            NodeTemplate {
                node_type: NodeType::Trigger,
                label: "Webhook",
                description: "HTTP webhook trigger",
                category: "Flow",
                default_config: serde_json::json!({"method": "POST", "path": "/webhook"}),
                inputs: vec![],
                outputs: vec!["payload"],
            },
            NodeTemplate {
                node_type: NodeType::Trigger,
                label: "Manual Trigger",
                description: "Manual button-press trigger",
                category: "Flow",
                default_config: serde_json::json!({}),
                inputs: vec![],
                outputs: vec!["start"],
            },
            NodeTemplate {
                node_type: NodeType::Wait,
                label: "Timer Wait",
                description: "Pause execution for a fixed duration",
                category: "Flow",
                default_config: serde_json::json!({"duration_secs": 10}),
                inputs: vec!["input"],
                outputs: vec!["output"],
            },
            NodeTemplate {
                node_type: NodeType::Wait,
                label: "Condition Wait",
                description: "Pause until a condition is met",
                category: "Flow",
                default_config: serde_json::json!({"condition": "", "poll_interval_secs": 5, "timeout_secs": 300}),
                inputs: vec!["input"],
                outputs: vec!["output", "timeout"],
            },
            NodeTemplate {
                node_type: NodeType::Switch,
                label: "Switch Case",
                description: "Multi-route switch-case routing",
                category: "Flow",
                default_config: serde_json::json!({"cases": [], "default": "default"}),
                inputs: vec!["value"],
                outputs: vec!["case_0", "case_1", "default"],
            },
            NodeTemplate {
                node_type: NodeType::Merge,
                label: "Parallel Merge",
                description: "Aggregate results from parallel branches",
                category: "Data",
                default_config: serde_json::json!({"strategy": "all", "timeout_secs": 60}),
                inputs: vec!["branch_a", "branch_b"],
                outputs: vec!["merged"],
            },
        ]
    }

    pub fn find_template(label: &str) -> Option<NodeTemplate> {
        Self::all_templates()
            .into_iter()
            .find(|template| template.label.eq_ignore_ascii_case(label))
    }

    pub fn templates_by_category(category: &str) -> Vec<NodeTemplate> {
        Self::all_templates()
            .into_iter()
            .filter(|template| template.category.eq_ignore_ascii_case(category))
            .collect()
    }

    pub fn categories() -> Vec<&'static str> {
        let mut categories = Vec::new();
        for template in Self::all_templates() {
            if !categories.contains(&template.category) {
                categories.push(template.category);
            }
        }
        categories
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct StudioRegistration {
    pub id: String,
    pub version: String,
    pub name: String,
    pub component_type: String,
    pub actions: Vec<String>,
    pub description: String,
    #[serde(default = "default_visibility")]
    pub visibility: String,
    #[serde(default)]
    pub owner_id: Option<String>,
    #[serde(default)]
    pub team_id: Option<String>,
}

fn default_visibility() -> String {
    "private".to_string()
}

impl StudioRegistration {
    pub fn new(id: &str, name: &str, component_type: &str) -> Self {
        Self {
            id: id.to_string(),
            version: "0.1.0".to_string(),
            name: name.to_string(),
            component_type: component_type.to_string(),
            actions: Vec::new(),
            description: String::new(),
            visibility: default_visibility(),
            owner_id: None,
            team_id: None,
        }
    }

    fn validate(&self) -> Result<(), MornError> {
        if self.id.trim().is_empty() {
            return Err("registration id cannot be empty".into());
        }
        if self.name.trim().is_empty() {
            return Err("registration name cannot be empty".into());
        }
        if self.component_type.trim().is_empty() {
            return Err("registration component_type cannot be empty".into());
        }
        if self.version.trim().is_empty() {
            return Err("registration version cannot be empty".into());
        }
        Ok(())
    }
}

impl From<StudioRegistration> for Capability {
    fn from(registration: StudioRegistration) -> Self {
        Capability {
            id: registration.id,
            version: registration.version,
            name: registration.name,
            domain: registration.component_type.to_lowercase(),
            actions: registration.actions,
            description: registration.description,
            trust_score: 70.0,
            total_calls: 0,
            success_calls: 0,
            avg_latency_ms: 0.0,
            visibility: registration.visibility,
            owner_id: registration.owner_id,
            team_id: registration.team_id,
            daily_quota: 0,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct StudioVersionInfo {
    pub id: String,
    pub current: Option<String>,
    pub history: Vec<String>,
}

pub struct StudioRegistry {
    registry: Registry,
}

impl StudioRegistry {
    pub fn new(registry: Registry) -> Self {
        Self { registry }
    }

    pub fn empty() -> Self {
        Self::new(Registry::new(None, None))
    }

    pub fn register_component(
        &mut self,
        registration: StudioRegistration,
    ) -> Result<Capability, MornError> {
        registration.validate()?;
        let capability: Capability = registration.into();
        if self.registry.get(&capability.id).is_some() {
            self.registry.register(capability.clone());
        } else {
            self.registry.register_dynamic(capability.clone())?;
        }
        Ok(capability)
    }

    pub fn unregister_component(&mut self, id: &str) -> Result<Capability, MornError> {
        Ok(self.registry
            .unregister(id)
            .ok_or_else(|| MornError::Internal(format!("component '{}' is not registered", id)))?)
    }

    pub fn get_component(&self, id: &str) -> Option<&Capability> {
        self.registry.get(id)
    }

    pub fn get_version(&self, id: &str) -> Option<&str> {
        self.registry.get_version(id)
    }

    pub fn get_version_info(&self, id: &str) -> StudioVersionInfo {
        StudioVersionInfo {
            id: id.to_string(),
            current: self.registry.get_version(id).map(str::to_string),
            history: self
                .registry
                .get_version_history(id)
                .into_iter()
                .map(str::to_string)
                .collect(),
        }
    }

    pub fn list_by_version(&self, version: &str) -> Vec<&Capability> {
        self.registry.list_by_version(version)
    }

    pub fn into_inner(self) -> Registry {
        self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_registry_has_templates() {
        let templates = NodeRegistry::all_templates();
        assert!(!templates.is_empty());
    }

    #[test]
    fn test_template_lookup_helpers() {
        assert!(NodeRegistry::find_template("HTTP Request").is_some());
        assert!(!NodeRegistry::templates_by_category("AI").is_empty());
        assert!(NodeRegistry::categories().contains(&"Flow"));
    }

    #[test]
    fn studio_registry_registers_updates_versions_and_unregisters() {
        let mut registry = StudioRegistry::empty();
        let mut registration = StudioRegistration::new("studio-agent", "Studio Agent", "agent");
        registration.actions = vec!["chat".into()];

        registry.register_component(registration.clone()).unwrap();
        assert_eq!(registry.get_version("studio-agent"), Some("0.1.0"));

        registration.version = "0.2.0".into();
        registry.register_component(registration).unwrap();
        assert_eq!(registry.get_version("studio-agent"), Some("0.2.0"));
        assert_eq!(
            registry.get_version_info("studio-agent").history,
            vec!["0.1.0".to_string(), "0.2.0".to_string()]
        );

        let removed = registry.unregister_component("studio-agent").unwrap();
        assert_eq!(removed.id, "studio-agent");
        assert!(registry.get_component("studio-agent").is_none());
    }
}
