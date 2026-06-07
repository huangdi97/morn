use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct SkillStep {
    pub step_id: String,
    pub tool_id: String,
    pub input_mapping: HashMap<String, String>,
    pub output_mapping: HashMap<String, String>,
    pub depends_on: Vec<String>,
    pub llm_step: bool,
    pub llm_prompt: Option<String>,
}

pub trait Skill: IOComponent {
    fn steps(&self) -> Vec<SkillStep>;
    fn execute(&mut self, input: Data) -> Result<Data, String>;
}

#[allow(dead_code)]
pub struct WebResearchSkill {
    id: String,
    name: String,
    steps: Vec<SkillStep>,
}

impl WebResearchSkill {
    pub fn new() -> Self {
        let steps = vec![
            SkillStep {
                step_id: "search".into(),
                tool_id: "web_search".into(),
                input_mapping: [("query".into(), "input.query".into())].into(),
                output_mapping: [("results".into(), "search_results".into())].into(),
                depends_on: vec![],
                llm_step: false,
                llm_prompt: None,
            },
            SkillStep {
                step_id: "summarize".into(),
                tool_id: "".into(),
                input_mapping: [("text".into(), "search_results".into())].into(),
                output_mapping: [("summary".into(), "output".into())].into(),
                depends_on: vec!["search".into()],
                llm_step: true,
                llm_prompt: Some("Summarize the following search results concisely:".into()),
            },
        ];
        WebResearchSkill {
            id: "skill-web-research".into(),
            name: "Web Research".into(),
            steps,
        }
    }
}

impl Component for WebResearchSkill {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "skill"
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
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl IOComponent for WebResearchSkill {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "research topic".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "research summary".into(),
            },
        ]
    }
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), String> {
        Ok(())
    }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, String> {
        Ok(None)
    }
}

impl SecureComponent for WebResearchSkill {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::NetworkAccess]
    }
}

impl Skill for WebResearchSkill {
    fn steps(&self) -> Vec<SkillStep> {
        self.steps.clone()
    }
    fn execute(&mut self, input: Data) -> Result<Data, String> {
        let topic = input.content.as_str().unwrap_or("").to_string();
        let mut search_tool = crate::component::tool::get_tool_by_name("web_search")
            .ok_or("web_search tool not found")?;
        let search_result = search_tool.execute(Data::text(&topic))?;
        let summary = format!(
            "[web_research] summary for '{}': {}",
            topic,
            search_result.content.as_str().unwrap_or("")
        );
        Ok(Data::text(&summary))
    }
}

#[allow(dead_code)]
pub struct DataAnalysisSkill {
    id: String,
    name: String,
    steps: Vec<SkillStep>,
}

impl DataAnalysisSkill {
    pub fn new() -> Self {
        let steps = vec![
            SkillStep {
                step_id: "read".into(),
                tool_id: "read_file".into(),
                input_mapping: [("path".into(), "input.path".into())].into(),
                output_mapping: [("data".into(), "raw_data".into())].into(),
                depends_on: vec![],
                llm_step: false,
                llm_prompt: None,
            },
            SkillStep {
                step_id: "analyze".into(),
                tool_id: "calc".into(),
                input_mapping: [("expression".into(), "raw_data".into())].into(),
                output_mapping: [("result".into(), "analysis".into())].into(),
                depends_on: vec!["read".into()],
                llm_step: true,
                llm_prompt: Some("Analyze the following data and provide insights:".into()),
            },
            SkillStep {
                step_id: "format".into(),
                tool_id: "".into(),
                input_mapping: [("text".into(), "analysis".into())].into(),
                output_mapping: [("report".into(), "output".into())].into(),
                depends_on: vec!["analyze".into()],
                llm_step: true,
                llm_prompt: Some("Format the analysis into a clear report:".into()),
            },
        ];
        DataAnalysisSkill {
            id: "skill-data-analysis".into(),
            name: "Data Analysis".into(),
            steps,
        }
    }
}

impl Component for DataAnalysisSkill {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "skill"
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
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl IOComponent for DataAnalysisSkill {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "file path".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "analysis report".into(),
            },
        ]
    }
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), String> {
        Ok(())
    }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, String> {
        Ok(None)
    }
}

impl SecureComponent for DataAnalysisSkill {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::ReadFile]
    }
}

impl Skill for DataAnalysisSkill {
    fn steps(&self) -> Vec<SkillStep> {
        self.steps.clone()
    }
    fn execute(&mut self, input: Data) -> Result<Data, String> {
        let path = input.content.as_str().unwrap_or("").to_string();
        Ok(Data::text(&format!(
            "[data_analysis] analysis of '{}' complete",
            path
        )))
    }
}

#[allow(dead_code)]
pub struct ReportGenSkill {
    id: String,
    name: String,
    steps: Vec<SkillStep>,
}

impl ReportGenSkill {
    pub fn new() -> Self {
        ReportGenSkill {
            id: "skill-report-gen".into(),
            name: "Report Generation".into(),
            steps: vec![],
        }
    }
}

impl Component for ReportGenSkill {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "skill"
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
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl IOComponent for ReportGenSkill {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "report topic".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "generated report".into(),
            },
        ]
    }
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), String> {
        Ok(())
    }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, String> {
        Ok(None)
    }
}

impl SecureComponent for ReportGenSkill {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::NetworkAccess]
    }
}

impl Skill for ReportGenSkill {
    fn steps(&self) -> Vec<SkillStep> {
        vec![]
    }
    fn execute(&mut self, input: Data) -> Result<Data, String> {
        let topic = input.content.as_str().unwrap_or("").to_string();
        Ok(Data::text(&format!(
            "[report_gen] report on '{}' generated",
            topic
        )))
    }
}

#[allow(dead_code)]
pub struct CodeReviewSkill {
    id: String,
    name: String,
    steps: Vec<SkillStep>,
}

impl CodeReviewSkill {
    pub fn new() -> Self {
        CodeReviewSkill {
            id: "skill-code-review".into(),
            name: "Code Review".into(),
            steps: vec![],
        }
    }
}

impl Component for CodeReviewSkill {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "skill"
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
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl IOComponent for CodeReviewSkill {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "file path".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "review result".into(),
            },
        ]
    }
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), String> {
        Ok(())
    }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, String> {
        Ok(None)
    }
}

impl SecureComponent for CodeReviewSkill {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![Permission::ReadFile]
    }
}

impl Skill for CodeReviewSkill {
    fn steps(&self) -> Vec<SkillStep> {
        vec![]
    }
    fn execute(&mut self, input: Data) -> Result<Data, String> {
        let path = input.content.as_str().unwrap_or("").to_string();
        Ok(Data::text(&format!(
            "[code_review] review of '{}' complete",
            path
        )))
    }
}

pub fn create_default_skills() -> Vec<Box<dyn Skill>> {
    vec![
        Box::new(WebResearchSkill::new()),
        Box::new(DataAnalysisSkill::new()),
        Box::new(ReportGenSkill::new()),
        Box::new(CodeReviewSkill::new()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_load() {
        let skills = create_default_skills();
        assert_eq!(skills.len(), 4);

        let research = WebResearchSkill::new();
        assert_eq!(research.id, "skill-web-research");
        assert_eq!(research.name, "Web Research");
        assert_eq!(research.steps().len(), 2);

        let analysis = DataAnalysisSkill::new();
        assert_eq!(analysis.steps().len(), 3);

        let report = ReportGenSkill::new();
        assert!(report.steps().is_empty());

        let review = CodeReviewSkill::new();
        assert!(review.steps().is_empty());
    }

    #[test]
    fn test_skill_execute() {
        let mut skill = WebResearchSkill::new();
        let result = skill.execute(Data::text("machine learning")).unwrap();
        assert!(result.content.as_str().unwrap().contains("web_research"));
        assert!(result
            .content
            .as_str()
            .unwrap()
            .contains("machine learning"));
    }

    #[test]
    fn test_skill_invalid() {
        let mut skill = WebResearchSkill::new();
        let result = skill.execute(Data::text(""));
        assert!(result.is_ok());
    }
}
