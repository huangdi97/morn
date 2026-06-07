//! report_generation — Defines the built-in skill for assembling reports.
use crate::component::skill::{Skill, SkillStep};
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

#[allow(dead_code)] /* 预留：内置 Report Generation skill 注册入口 */
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

impl Default for ReportGenSkill {
    fn default() -> Self {
        Self::new()
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
