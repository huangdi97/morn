//! scan — Scan, fix, and apply-fix methods for SelfEvolutionSkill.
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use super::EvolutionReport;
use super::SelfEvolutionSkill;

impl SelfEvolutionSkill {
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
            "Review ownership and lifetimes, then clone, borrow, or restructure values where needed."
                .into()
        } else if lower.contains("warning") || lower.contains("warn") {
            "Resolve the warning by following the compiler diagnostic.".into()
        } else {
            "Inspect the diagnostic and apply the smallest targeted code change.".into()
        }
    }

    pub fn scan() -> Vec<String> {
        #[cfg(test)]
        {
            Self::scan_for_issues()
        }

        #[cfg(not(test))]
        {
            let output = Command::new("cargo").args(["clippy", "--lib"]).output();
            let Ok(output) = output else {
                return Vec::new();
            };

            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );

            combined
                .lines()
                .map(str::trim)
                .filter(|line| {
                    line.starts_with("warning:")
                        || line.starts_with("warning[")
                        || line.contains(": warning:")
                })
                .filter(|line| !line.is_empty())
                .map(ToOwned::to_owned)
                .collect()
        }
    }

    pub fn fix(warnings: &[String]) -> Vec<String> {
        if warnings.is_empty() {
            return Vec::new();
        }

        #[cfg(test)]
        {
            warnings
                .iter()
                .map(|warning| Self::generate_fix(warning))
                .collect()
        }

        #[cfg(not(test))]
        {
            let has_auto_fixable_warning = warnings.iter().any(|warning| {
                let lower = warning.to_ascii_lowercase();
                lower.contains("dead_code")
                    || lower.contains("unused import")
                    || lower.contains("unused_imports")
                    || lower.contains("unused variable")
                    || lower.contains("unused_variables")
            });

            if !has_auto_fixable_warning {
                return warnings
                    .iter()
                    .map(|warning| Self::generate_fix(warning))
                    .collect();
            }

            match Command::new("cargo")
                .args(["fix", "--allow-dirty"])
                .output()
            {
                Ok(output) if output.status.success() => {
                    vec!["cargo fix --allow-dirty applied automatic fixes".to_string()]
                }
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    vec![format!("cargo fix --allow-dirty failed: {}", stderr)]
                }
                Err(e) => {
                    vec![format!("cargo fix --allow-dirty failed to start: {}", e)]
                }
            }
        }
    }

    pub fn apply_fixes(fixes: &[String]) -> Result<String, String> {
        #[cfg(test)]
        {
            let _ = fixes;
            Ok("test validation skipped".to_string())
        }

        #[cfg(not(test))]
        {
            let _ = fixes;
            let output = Command::new("cargo")
                .arg("build")
                .output()
                .map_err(|e| format!("cargo build failed to start: {}", e))?;
            let combined = format!(
                "{}{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            if output.status.success() {
                Ok(combined)
            } else {
                Err(combined)
            }
        }
    }

    pub fn apply_fix(fix: &str) -> Result<(), String> {
        Self::apply_fixes(&[fix.to_string()]).map(|_| ())
    }

    pub fn validate() -> bool {
        Command::new("cargo")
            .arg("check")
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    pub fn report() -> EvolutionReport {
        let issues_found = Self::scan();
        let auto_fixes_applied = Self::fix(&issues_found);
        let validation = Self::apply_fixes(&auto_fixes_applied);
        let fixes_successful = if validation.is_ok() {
            auto_fixes_applied.len()
        } else {
            0
        };
        let fixes_failed = auto_fixes_applied.len().saturating_sub(fixes_successful);

        EvolutionReport {
            scan_time: Self::scan_time(),
            issues_found,
            auto_fixes_applied,
            fixes_successful,
            fixes_failed,
        }
    }

    pub(crate) fn error_log_path() -> Option<PathBuf> {
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
