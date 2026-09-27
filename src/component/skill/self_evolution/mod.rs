//! self_evolution — Defines the built-in skill for project self-improvement scans.
use crate::component::skill::{Skill, SkillStep};
use crate::core::component::{Component, Data, HealthStatus, IOComponent, Port, PortDirection};
use crate::core::error::MornError;

mod scan;

pub struct SelfEvolutionSkill {
    pub id: String,
    pub name: String,
    pub steps: Vec<SkillStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvolutionReport {
    pub scan_time: String,
    pub issues_found: Vec<String>,
    pub auto_fixes_applied: Vec<String>,
    pub fixes_successful: usize,
    pub fixes_failed: usize,
}

impl SelfEvolutionSkill {
    pub fn new() -> Self {
        SelfEvolutionSkill {
            id: "skill-self-evolution".into(),
            name: "Self Evolution".into(),
            steps: vec![],
        }
    }
}

impl Default for SelfEvolutionSkill {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for SelfEvolutionSkill {
    fn id(&self) -> &str {
        &self.id
    }

    fn type_name(&self) -> &str {
        "skill"
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

    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl IOComponent for SelfEvolutionSkill {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "scan request".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "evolution report".into(),
            },
        ]
    }

    fn send(&mut self, _port: &str, _data: Data) -> Result<(), MornError> {
        Ok(())
    }

    fn recv(&mut self, _port: &str) -> Result<Option<Data>, MornError> {
        Ok(None)
    }
}

impl Skill for SelfEvolutionSkill {
    fn steps(&self) -> Vec<SkillStep> {
        self.steps.clone()
    }

    fn execute(&mut self, _input: Data) -> Result<Data, MornError> {
        let report = Self::report();
        Ok(Data::text(&format!(
            "[self_evolution] scan complete: {} issue(s), {} fix(es) applied, {} failed",
            report.issues_found.len(),
            report.fixes_successful,
            report.fixes_failed
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_correctly() {
        let skill = SelfEvolutionSkill::new();

        assert_eq!(skill.id(), "skill-self-evolution");
        assert_eq!(skill.name, "Self Evolution");
        assert!(skill.steps().is_empty());
        assert_eq!(skill.type_name(), "skill");
    }

    // Tests that touch the real user-home error log live in scan_tests.rs
    // (single shared guard; two guards would not be mutually exclusive).
}
