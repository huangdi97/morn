use serde_json::Value;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SOPTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub stages: Vec<SOPStage>,
    pub category: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SOPStage {
    pub name: String,
    pub prompt: String,
    pub expected_output: String,
    pub tools_allowed: Vec<String>,
    pub approval_required: bool,
}

impl SOPTemplate {
    pub fn compile_prompt(&self, context: &Value) -> String {
        let mut prompt = format!(
            "# SOP: {}\n\n{}\n\n## Stages\n",
            self.name, self.description
        );

        for (i, stage) in self.stages.iter().enumerate() {
            prompt.push_str(&format!("\n### Stage {}: {}\n", i + 1, stage.name));
            prompt.push_str(&format!("{}\n", stage.prompt));
            if !stage.tools_allowed.is_empty() {
                prompt.push_str(&format!(
                    "Allowed tools: {}\n",
                    stage.tools_allowed.join(", ")
                ));
            }
            prompt.push_str(&format!("Expected output: {}\n", stage.expected_output));
            if stage.approval_required {
                prompt.push_str("⚠️ Approval required before proceeding\n");
            }
        }

        if let Some(ctx_str) = context.get("context").and_then(|c| c.as_str()) {
            prompt.push_str(&format!("\n## Context\n{}\n", ctx_str));
        }

        prompt
    }

    pub fn from_file(path: &Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let template: SOPTemplate =
            serde_json::from_str(&content).map_err(|e| format!("Parse error: {}", e))?;
        Ok(template)
    }

    pub fn list_builtin() -> Vec<SOPTemplate> {
        vec![
            Self::software_development_template(),
            Self::data_analysis_template(),
            Self::research_template(),
        ]
    }

    fn software_development_template() -> SOPTemplate {
        SOPTemplate {
            id: "sop-software-dev".to_string(),
            name: "Software Development".to_string(),
            description: "Standard software development lifecycle: requirements analysis, design, coding, testing, and review".to_string(),
            category: "development".to_string(),
            tags: vec!["software".to_string(), "development".to_string(), "coding".to_string()],
            stages: vec![
                SOPStage {
                    name: "Requirements Analysis".to_string(),
                    prompt: "Analyze the requirements thoroughly. Identify functional and non-functional requirements. Clarify any ambiguities with the user.".to_string(),
                    expected_output: "Clear requirements document with acceptance criteria".to_string(),
                    tools_allowed: vec![],
                    approval_required: true,
                },
                SOPStage {
                    name: "Design".to_string(),
                    prompt: "Design the architecture based on requirements. Consider modularity, scalability, and maintainability. Define interfaces and data flow.".to_string(),
                    expected_output: "Architecture design document with component diagram".to_string(),
                    tools_allowed: vec![],
                    approval_required: true,
                },
                SOPStage {
                    name: "Coding".to_string(),
                    prompt: "Implement the design following the project's coding standards. Write clean, tested code with appropriate error handling.".to_string(),
                    expected_output: "Working code with inline documentation".to_string(),
                    tools_allowed: vec!["code_exec".to_string(), "file_ops".to_string()],
                    approval_required: false,
                },
                SOPStage {
                    name: "Testing".to_string(),
                    prompt: "Write and run unit tests, integration tests. Verify edge cases and error conditions. Ensure test coverage meets standards.".to_string(),
                    expected_output: "Test report with coverage metrics".to_string(),
                    tools_allowed: vec!["code_exec".to_string()],
                    approval_required: false,
                },
                SOPStage {
                    name: "Code Review".to_string(),
                    prompt: "Review the code for quality, security, and adherence to standards. Check for potential bugs and performance issues.".to_string(),
                    expected_output: "Review report with issues found and recommendations".to_string(),
                    tools_allowed: vec![],
                    approval_required: true,
                },
            ],
        }
    }

    fn data_analysis_template() -> SOPTemplate {
        SOPTemplate {
            id: "sop-data-analysis".to_string(),
            name: "Data Analysis".to_string(),
            description: "Structured approach to data analysis: understanding data, cleaning, analysis, visualization, and reporting".to_string(),
            category: "data".to_string(),
            tags: vec!["data".to_string(), "analysis".to_string(), "reporting".to_string()],
            stages: vec![
                SOPStage {
                    name: "Data Understanding".to_string(),
                    prompt: "Load and examine the dataset. Understand the structure, types, and basic statistics of each column.".to_string(),
                    expected_output: "Data profile summary with column descriptions".to_string(),
                    tools_allowed: vec!["data_loader".to_string(), "code_exec".to_string()],
                    approval_required: false,
                },
                SOPStage {
                    name: "Data Cleaning".to_string(),
                    prompt: "Handle missing values, outliers, and inconsistencies. Document all cleaning decisions.".to_string(),
                    expected_output: "Cleaned dataset with cleaning log".to_string(),
                    tools_allowed: vec!["code_exec".to_string()],
                    approval_required: false,
                },
                SOPStage {
                    name: "Analysis".to_string(),
                    prompt: "Perform statistical analysis and build models. Test hypotheses. Validate results.".to_string(),
                    expected_output: "Analysis results with statistical measures".to_string(),
                    tools_allowed: vec!["code_exec".to_string(), "stat_tools".to_string()],
                    approval_required: false,
                },
                SOPStage {
                    name: "Reporting".to_string(),
                    prompt: "Create a comprehensive report with visualizations. Summarize key findings and provide actionable recommendations.".to_string(),
                    expected_output: "Final report with charts and recommendations".to_string(),
                    tools_allowed: vec![],
                    approval_required: true,
                },
            ],
        }
    }

    fn research_template() -> SOPTemplate {
        SOPTemplate {
            id: "sop-research".to_string(),
            name: "Research & Investigation".to_string(),
            description: "Systematic research process: query formulation, multi-source search, cross-validation, and synthesis".to_string(),
            category: "research".to_string(),
            tags: vec!["research".to_string(), "investigation".to_string(), "synthesis".to_string()],
            stages: vec![
                SOPStage {
                    name: "Query Formulation".to_string(),
                    prompt: "Break down the research question into specific search queries. Identify key terms and concepts.".to_string(),
                    expected_output: "List of search queries and key terms".to_string(),
                    tools_allowed: vec![],
                    approval_required: false,
                },
                SOPStage {
                    name: "Multi-Source Search".to_string(),
                    prompt: "Execute search across multiple sources. Collect diverse perspectives and data points.".to_string(),
                    expected_output: "Raw search results from multiple sources".to_string(),
                    tools_allowed: vec!["web_search".to_string(), "knowledge_query".to_string()],
                    approval_required: false,
                },
                SOPStage {
                    name: "Cross-Validation".to_string(),
                    prompt: "Cross-reference findings from different sources. Identify contradictions and consensus points.".to_string(),
                    expected_output: "Cross-validation report with confidence ratings".to_string(),
                    tools_allowed: vec![],
                    approval_required: false,
                },
                SOPStage {
                    name: "Synthesis".to_string(),
                    prompt: "Synthesize validated findings into a coherent response. Support conclusions with evidence.".to_string(),
                    expected_output: "Final research summary with citations".to_string(),
                    tools_allowed: vec![],
                    approval_required: true,
                },
            ],
        }
    }

    pub fn get_builtin(id: &str) -> Option<SOPTemplate> {
        Self::list_builtin().into_iter().find(|t| t.id == id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_builtin() {
        let templates = SOPTemplate::list_builtin();
        assert_eq!(templates.len(), 3);
    }

    #[test]
    fn test_get_builtin() {
        let t = SOPTemplate::get_builtin("sop-software-dev");
        assert!(t.is_some());
        assert_eq!(t.unwrap().name, "Software Development");
    }

    #[test]
    fn test_compile_prompt() {
        let template = SOPTemplate::get_builtin("sop-software-dev").unwrap();
        let prompt = template.compile_prompt(&serde_json::json!({}));
        assert!(prompt.contains("SOP:"));
        assert!(prompt.contains("Requirements Analysis"));
        assert!(prompt.contains("Stage 1"));
    }

    #[test]
    fn test_compile_prompt_with_context() {
        let template = SOPTemplate::get_builtin("sop-data-analysis").unwrap();
        let prompt = template.compile_prompt(&serde_json::json!({
            "context": "Analyze customer churn data for Q1 2025"
        }));
        assert!(prompt.contains("Q1 2025"));
    }

    #[test]
    fn test_approval_required_flag() {
        let template = SOPTemplate::get_builtin("sop-software-dev").unwrap();
        assert!(template.stages[0].approval_required);
        assert!(template.stages[4].approval_required);
        assert!(!template.stages[2].approval_required);
    }

    #[test]
    fn test_tools_allowed() {
        let template = SOPTemplate::get_builtin("sop-research").unwrap();
        let search_stage = &template.stages[1];
        assert!(search_stage
            .tools_allowed
            .contains(&"web_search".to_string()));
    }

    #[test]
    fn test_all_templates_have_stages() {
        for t in SOPTemplate::list_builtin() {
            assert!(!t.stages.is_empty(), "Template '{}' has no stages", t.id);
        }
    }

    #[test]
    fn test_serialization_roundtrip() {
        let template = SOPTemplate::get_builtin("sop-software-dev").unwrap();
        let json = serde_json::to_string(&template).unwrap();
        let deserialized: SOPTemplate = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, template.id);
        assert_eq!(deserialized.stages.len(), template.stages.len());
    }

    #[test]
    fn test_from_file_nonexistent() {
        let result = SOPTemplate::from_file(Path::new("/nonexistent/sop.json"));
        assert!(result.is_err());
    }
}
