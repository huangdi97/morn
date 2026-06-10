use super::child_process::{ChildProcess, ProcessStatus};

#[test]
fn test_child_process_new_is_idle() {
    let cp = ChildProcess::new();
    assert!(!cp.is_running());
    assert_eq!(cp.status(), ProcessStatus::Idle);
}

#[test]
fn test_child_process_kill_idle_is_ok() {
    let mut cp = ChildProcess::new();
    assert!(cp.kill().is_ok());
}

#[test]
fn test_child_process_default_state() {
    let cp = ChildProcess::new();
    assert_eq!(cp.status(), ProcessStatus::Idle);
    assert!(!cp.is_running());
}
