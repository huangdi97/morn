//! workflow_builder — Builds executable workflows from registered capabilities and tasks.
use crate::core::registry::Registry;
use crate::core::workflow::{WorkflowAction, WorkflowStep, WorkflowTemplate};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowPlan {
    pub description: String,
    pub nodes: Vec<WorkflowNode>,
    pub edges: Vec<WorkflowEdge>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub action_type: String,
    pub label: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkflowEdge {
    pub from: String,
    pub to: String,
}

pub struct WorkflowBuilder {
    #[allow(dead_code)] /* 预留：后续按 registry 校验 workflow action/tool */
    registry: Arc<Registry>,
}

impl WorkflowBuilder {
    pub fn new(registry: Arc<Registry>) -> Self {
        WorkflowBuilder { registry }
    }

    pub async fn nl_to_workflow(&self, description: &str) -> Result<WorkflowPlan, String> {
        let lower = description.to_lowercase();
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        let plan_node = WorkflowNode {
            id: "understand".to_string(),
            action_type: "llm_call".to_string(),
            label: "Analyze Requirements".to_string(),
            params: serde_json::json!({"prompt": description}),
        };
        nodes.push(plan_node);

        if lower.contains("search") || lower.contains("research") || lower.contains("查找") {
            let search_node = WorkflowNode {
                id: "search".to_string(),
                action_type: "tool_call".to_string(),
                label: "Search Information".to_string(),
                params: serde_json::json!({"tool": "web_search", "query": description}),
            };
            nodes.push(search_node);
            edges.push(WorkflowEdge {
                from: "understand".to_string(),
                to: "search".to_string(),
            });
        }

        if lower.contains("report")
            || lower.contains("write")
            || lower.contains("write")
            || lower.contains("生成")
        {
            let gen_node = WorkflowNode {
                id: "generate".to_string(),
                action_type: "llm_call".to_string(),
                label: "Generate Output".to_string(),
                params: serde_json::json!({"task": "generate_report"}),
            };
            nodes.push(gen_node);
            let last = if nodes.len() > 2 {
                "search"
            } else {
                "understand"
            };
            edges.push(WorkflowEdge {
                from: last.to_string(),
                to: "generate".to_string(),
            });
        }

        let summary_node = WorkflowNode {
            id: "summarize".to_string(),
            action_type: "llm_call".to_string(),
            label: "Summarize".to_string(),
            params: serde_json::json!({"task": "summarize"}),
        };
        nodes.push(summary_node);
        let prev = if nodes.len() > 2 {
            &nodes[nodes.len() - 2].id
        } else {
            "understand"
        };
        edges.push(WorkflowEdge {
            from: prev.to_string(),
            to: "summarize".to_string(),
        });

        Ok(WorkflowPlan {
            description: description.to_string(),
            nodes,
            edges,
        })
    }

    pub async fn auto_fix(
        &self,
        workflow: &WorkflowPlan,
        error: &str,
    ) -> Result<WorkflowPlan, String> {
        let mut fixed = workflow.clone();
        fixed.nodes.push(WorkflowNode {
            id: "fix".to_string(),
            action_type: "llm_call".to_string(),
            label: format!("Fix: {}", error),
            params: serde_json::json!({"error": error}),
        });
        fixed.edges.push(WorkflowEdge {
            from: "summarize".to_string(),
            to: "fix".to_string(),
        });
        Ok(fixed)
    }

    pub fn compile(&self, plan: &WorkflowPlan) -> Result<WorkflowTemplate, String> {
        let mut steps = Vec::new();
        for node in &plan.nodes {
            let action = match node.action_type.as_str() {
                "tool_call" => WorkflowAction::ToolCall {
                    tool_id: node
                        .params
                        .get("tool")
                        .and_then(|v| v.as_str())
                        .unwrap_or("default")
                        .to_string(),
                    params: node.params.clone(),
                },
                _ => WorkflowAction::LLMCall {
                    system_prompt: node.label.clone(),
                    model: "default".to_string(),
                },
            };

            let depends_on: Vec<String> = plan
                .edges
                .iter()
                .filter(|e| e.to == node.id)
                .map(|e| e.from.clone())
                .collect();

            steps.push(WorkflowStep {
                id: node.id.clone(),
                action,
                depends_on,
                timeout_secs: 30,
                retry_count: 1,
                approval_required: false,
                input_mapping: HashMap::new(),
                output_mapping: HashMap::new(),
            });
        }

        let step_count = steps.len();
        Ok(WorkflowTemplate {
            id: format!("wf-{}", uuid::Uuid::new_v4()),
            name: format!(
                "Workflow: {}",
                &plan.description[..plan.description.len().min(40)]
            ),
            description: plan.description.clone(),
            steps,
            estimated_duration_secs: (step_count * 30) as u64,
            category: "generated".to_string(),
            tags: vec!["auto".to_string(), "generated".to_string()],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage::Storage;

    fn setup_builder() -> WorkflowBuilder {
        let storage = Arc::new(Storage::new_in_memory().unwrap());
        let event_bus = None;
        let registry = Arc::new(Registry::new(Some((*storage).clone()), event_bus));
        WorkflowBuilder::new(registry)
    }

    #[tokio::test]
    async fn test_nl_to_workflow_simple() {
        let builder = setup_builder();
        let plan = builder
            .nl_to_workflow("search for AI news and write a report")
            .await
            .unwrap();
        assert!(!plan.nodes.is_empty());
        assert!(plan.nodes.len() >= 3);
    }

    #[tokio::test]
    async fn test_nl_to_workflow_has_edges() {
        let builder = setup_builder();
        let plan = builder
            .nl_to_workflow("research topic and generate report")
            .await
            .unwrap();
        assert!(!plan.edges.is_empty());
    }

    #[tokio::test]
    async fn test_compile_to_template() {
        let builder = setup_builder();
        let plan = builder.nl_to_workflow("write a summary").await.unwrap();
        let template = builder.compile(&plan).unwrap();
        assert!(!template.steps.is_empty());
        assert_eq!(template.category, "generated");
    }

    #[tokio::test]
    async fn test_auto_fix_adds_node() {
        let builder = setup_builder();
        let plan = builder.nl_to_workflow("test").await.unwrap();
        let fixed = builder.auto_fix(&plan, "Tool not found").await.unwrap();
        assert!(fixed.nodes.len() > plan.nodes.len());
    }

    #[tokio::test]
    async fn test_workflow_roundtrip() {
        let builder = setup_builder();
        let plan = builder
            .nl_to_workflow("search data and generate report")
            .await
            .unwrap();
        let template = builder.compile(&plan).unwrap();
        assert!(!template.steps.is_empty());
        assert!(template.tags.contains(&"auto".to_string()));
    }
}
