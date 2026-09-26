//! Tests for CodeToolExecutor. Interpreter availability is platform-dependent:
//! `python`/`py` on Windows, `python3`/`python` on unix; shell scripts run via
//! cmd on Windows and bash on unix.

use super::*;

#[test]
fn test_unsupported_language() {
    let executor = CodeToolExecutor::new();
    let result = executor.execute("print('hi')", "ruby");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("unsupported language"));
}

#[test]
fn test_dangerous_pattern_blocked() {
    let executor = CodeToolExecutor::new();
    let result = executor.execute("rm -rf /", "sh");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("dangerous pattern"));
}

#[test]
fn test_python_execution() {
    let executor = CodeToolExecutor::new();
    let result = executor
        .execute("print('hello from code_tool')", "python")
        .unwrap();
    assert!(result.stdout.contains("hello from code_tool"));
    assert_eq!(result.exit_code, 0);
    assert!(!result.timed_out);
}

#[test]
fn test_shell_execution() {
    let executor = CodeToolExecutor::new();
    let result = executor.execute("echo 'shell test'", "sh").unwrap();
    assert!(result.stdout.contains("shell test"));
    assert_eq!(result.exit_code, 0);
    assert!(!result.timed_out);
}

#[test]
fn test_timeout() {
    let executor = CodeToolExecutor::new().with_timeout(1);
    // The long-running command must exist on the current platform: `sleep` is
    // a unix binary; `ping -n` is the Windows equivalent delay.
    #[cfg(windows)]
    let script = "ping -n 10 127.0.0.1";
    #[cfg(not(windows))]
    let script = "sleep 10";
    let result = executor.execute(script, "sh").unwrap();
    assert!(result.timed_out);
    assert!(result.exit_code == 124 || result.exit_code == -1);
}

#[test]
fn test_stderr_capture() {
    let executor = CodeToolExecutor::new();
    let result = executor
        .execute("import sys; sys.stderr.write('error msg')", "py")
        .unwrap();
    assert!(result.stderr.contains("error msg"));
}

#[test]
fn test_exit_code_nonzero() {
    let executor = CodeToolExecutor::new();
    let result = executor.execute("exit 42", "sh").unwrap();
    assert_eq!(result.exit_code, 42);
}
