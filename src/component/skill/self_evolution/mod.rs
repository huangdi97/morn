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
    use std::fs;
    use std::sync::{Mutex, OnceLock};

    fn log_guard() -> &'static Mutex<()> {
        static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
        GUARD.get_or_init(|| Mutex::new(()))
    }

    fn with_error_log<T>(content: &str, test: impl FnOnce() -> T) -> T {
        let _guard = log_guard().lock().unwrap();
        let path = SelfEvolutionSkill::error_log_path()
            .unwrap_or_else(|| panic!("HOME should be set for tests"));
        let original = fs::read(&path).ok();

        let parent = path
            .parent()
            .unwrap_or_else(|| panic!("error log should have parent"));
        fs::create_dir_all(parent).unwrap();
        fs::write(&path, content).unwrap();

        let result = test();

        match original {
            Some(bytes) => fs::write(&path, bytes).unwrap(),
            None => {
                let _ = fs::remove_file(&path);
            }
        }

        result
    }

    #[test]
    fn test_new_creates_correctly() {
        let skill = SelfEvolutionSkill::new();

        assert_eq!(skill.id(), "skill-self-evolution");
        assert_eq!(skill.name, "Self Evolution");
        assert!(skill.steps().is_empty());
        assert_eq!(skill.type_name(), "skill");
    }

    #[test]
    fn test_scan_for_issues_returns_results() {
        with_error_log(
            "info: startup complete\nwarning: unused import: `Path`\nerror[E0425]: cannot find value `x`\n",
            || {
                let issues = SelfEvolutionSkill::scan_for_issues();

                assert_eq!(issues.len(), 2);
                assert!(issues.iter().any(|issue| issue.contains("warning")));
                assert!(issues.iter().any(|issue| issue.contains("error")));
            },
        );
    }

    #[test]
    fn test_report_structure() {
        with_error_log("compile error: mismatched types\n", || {
            let report = SelfEvolutionSkill::report();

            assert!(!report.scan_time.is_empty());
            assert_eq!(report.issues_found, vec!["compile error: mismatched types"]);
            assert_eq!(report.auto_fixes_applied.len(), 1);
            assert_eq!(report.fixes_successful, 1);
            assert_eq!(report.fixes_failed, 0);
        });
    }
}
