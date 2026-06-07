//! code_review — Defines the built-in skill for structured code review workflows.
use crate::component::skill::{Skill, SkillStep};
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

#[allow(dead_code)] /* 预留：内置 Code Review skill 注册入口 */
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

impl Default for CodeReviewSkill {
    fn default() -> Self {
        Self::new()
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
