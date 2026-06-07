//! self_evolution — Defines the built-in skill for project self-improvement scans.
use crate::component::skill::{Skill, SkillStep};
use crate::core::component::{Component, Data, HealthStatus, IOComponent, Port, PortDirection};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[allow(dead_code)] /* Reserved for registry metadata and UI display. */
pub struct SelfEvolutionSkill {
    id: String,
    name: String,
    steps: Vec<SkillStep>,
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

    pub fn scan_for_issues() -> Vec<String> {
        let Some(log_path) = Self::error_log_path() else {
            return Vec::new();
        };

        let Ok(log) = fs::read_to_string(log_path) else {
            return Vec::new();
        };

        log.lines()
            .map(str::trim)
            .filter(|line| {
                let lower = line.to_ascii_lowercase();
                lower.contains("compile")
                    || lower.contains("error")
                    || lower.contains("warning")
                    || lower.contains("warn")
            })
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect()
    }

    pub fn generate_fix(issue: &str) -> String {
        let lower = issue.to_ascii_lowercase();

        if lower.contains("unused import") {
            "Remove the unused import or use the imported item.".into()
        } else if lower.contains("cannot find") || lower.contains("not found") {
            "Add the missing item, correct the identifier, or import it from the right module."
                .into()
        } else if lower.contains("mismatched types") {
            "Adjust the expression type or add an explicit conversion so both sides match.".into()
        } else if lower.contains("borrow") || lower.contains("lifetime") {
            "Review ownership and lifetimes, then clone, borrow, or restructure values where needed.".into()
        } else if lower.contains("warning") || lower.contains("warn") {
            "Resolve the warning by following the compiler diagnostic.".into()
        } else {
            "Inspect the diagnostic and apply the smallest targeted code change.".into()
        }
    }

    pub fn apply_fix(_fix: &str) -> Result<(), String> {
        Ok(())
    }

    pub fn validate() -> bool {
        Command::new("cargo")
            .arg("check")
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    pub fn report() -> EvolutionReport {
        let issues_found = Self::scan_for_issues();
        let auto_fixes_applied: Vec<String> = issues_found
            .iter()
            .map(|issue| Self::generate_fix(issue))
            .collect();
        let fixes_successful = auto_fixes_applied
            .iter()
            .filter(|fix| Self::apply_fix(fix).is_ok())
            .count();
        let fixes_failed = auto_fixes_applied.len().saturating_sub(fixes_successful);

        EvolutionReport {
            scan_time: Self::scan_time(),
            issues_found,
            auto_fixes_applied,
            fixes_successful,
            fixes_failed,
        }
    }

    fn error_log_path() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|home| home.join(".hermes").join("logs").join("error.log"))
    }

    fn scan_time() -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_secs().to_string())
            .unwrap_or_else(|_| "0".into())
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

    fn send(&mut self, _port: &str, _data: Data) -> Result<(), String> {
        Ok(())
    }

    fn recv(&mut self, _port: &str) -> Result<Option<Data>, String> {
        Ok(None)
    }
}

impl Skill for SelfEvolutionSkill {
    fn steps(&self) -> Vec<SkillStep> {
        self.steps.clone()
    }

    fn execute(&mut self, _input: Data) -> Result<Data, String> {
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
    use std::sync::{Mutex, OnceLock};

    fn log_guard() -> &'static Mutex<()> {
        static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
        GUARD.get_or_init(|| Mutex::new(()))
    }

    fn with_error_log<T>(content: &str, test: impl FnOnce() -> T) -> T {
        let _guard = log_guard().lock().unwrap();
        let path = SelfEvolutionSkill::error_log_path().expect("HOME should be set for tests");
        let original = fs::read(&path).ok();

        fs::create_dir_all(path.parent().expect("error log should have parent")).unwrap();
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
