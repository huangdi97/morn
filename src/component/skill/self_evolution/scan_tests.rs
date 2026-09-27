//! scan_tests — SelfEvolutionSkill scan/fix tests (external, repo convention).

use super::*;
use std::sync::{Mutex, OnceLock};

/// Serializes tests that touch the real user-home error log. Without this
/// guard, a test that assumes "no log" races with tests that write the log
/// (REGRESSION: test_scan_for_issues_no_log_path failed non-deterministically).
fn log_guard() -> &'static Mutex<()> {
    static GUARD: OnceLock<Mutex<()>> = OnceLock::new();
    GUARD.get_or_init(|| Mutex::new(()))
}

#[test]
fn test_generate_fix_unused_import() {
    let fix = SelfEvolutionSkill::generate_fix("unused import: `Foo`");
    assert!(fix.contains("Remove the unused import"));
}

#[test]
fn test_generate_fix_cannot_find() {
    let fix = SelfEvolutionSkill::generate_fix("cannot find value `x`");
    assert!(fix.contains("missing item"));
}

#[test]
fn test_generate_fix_not_found() {
    let fix = SelfEvolutionSkill::generate_fix("not found: `bar`");
    assert!(fix.contains("missing item"));
}

#[test]
fn test_generate_fix_mismatched_types() {
    let fix = SelfEvolutionSkill::generate_fix("mismatched types expected i32 found u32");
    assert!(fix.contains("conversion"));
}

#[test]
fn test_generate_fix_borrow() {
    let fix = SelfEvolutionSkill::generate_fix("borrow of moved value");
    assert!(fix.contains("ownership"));
}

#[test]
fn test_generate_fix_lifetime() {
    let fix = SelfEvolutionSkill::generate_fix("lifetime mismatch");
    assert!(fix.contains("ownership"));
}

#[test]
fn test_generate_fix_warning() {
    let fix = SelfEvolutionSkill::generate_fix("warning: dead_code");
    assert!(fix.contains("compiler diagnostic"));
}

#[test]
fn test_generate_fix_default() {
    let fix = SelfEvolutionSkill::generate_fix("some unknown error");
    assert!(fix.contains("smallest"));
}

#[test]
fn test_scan_time_is_non_empty() {
    let t = SelfEvolutionSkill::scan_time();
    assert!(!t.is_empty());
}

#[test]
fn test_error_log_path_contains_hermes_logs() {
    let path = SelfEvolutionSkill::error_log_path();
    assert!(path.is_some());
    let s = path.unwrap().to_string_lossy().to_string();
    assert!(s.contains(".hermes"));
    assert!(s.contains("error.log"));
}

#[test]
fn test_fix_empty_warnings() {
    let fixes = SelfEvolutionSkill::fix(&[]);
    assert!(fixes.is_empty());
}

#[test]
fn test_fix_with_warnings() {
    let fixes = SelfEvolutionSkill::fix(&["unused import: `Foo`".to_string()]);
    assert_eq!(fixes.len(), 1);
    assert!(fixes[0].contains("Remove the unused import"));
}

#[test]
fn test_apply_fixes_in_test_mode() {
    let result = SelfEvolutionSkill::apply_fixes(&["test fix".to_string()]);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test validation skipped");
}

#[test]
fn test_apply_fix_single() {
    let result = SelfEvolutionSkill::apply_fix("fix something");
    assert!(result.is_ok());
}

#[test]
fn test_scan_for_issues_no_log_path() {
    // Hold the guard, then ensure the error log is absent for this test so the
    // "no log" scenario is deterministic even on machines where the real
    // user-home log exists.
    let _guard = log_guard().lock().unwrap();
    let path = SelfEvolutionSkill::error_log_path().expect("error log path must resolve");
    let original = std::fs::read(&path).ok();
    let _ = std::fs::remove_file(&path);

    let issues = SelfEvolutionSkill::scan_for_issues();
    assert!(
        issues.is_empty(),
        "scan with no log file must return no issues"
    );

    match original {
        Some(bytes) => {
            let parent = path.parent().expect("log has parent");
            std::fs::create_dir_all(parent).unwrap();
            std::fs::write(&path, bytes).unwrap();
        }
        None => {}
    }
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

/// Runs a test with a temporary error log (single shared guard).
fn with_error_log<T>(content: &str, test: impl FnOnce() -> T) -> T {
    let _guard = log_guard().lock().unwrap();
    let path = SelfEvolutionSkill::error_log_path().expect("error log path must resolve");
    let original = std::fs::read(&path).ok();

    let parent = path.parent().expect("error log should have parent");
    std::fs::create_dir_all(parent).unwrap();
    std::fs::write(&path, content).unwrap();

    let result = test();

    match original {
        Some(bytes) => std::fs::write(&path, bytes).unwrap(),
        None => {
            let _ = std::fs::remove_file(&path);
        }
    }
    result
}

#[test]
fn test_fix_includes_warnings() {
    let warnings = vec!["dead_code: unused function".to_string()];
    let fixes = SelfEvolutionSkill::fix(&warnings);
    assert_eq!(fixes.len(), 1);
}
