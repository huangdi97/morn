//! Windows process runner for CodeToolExecutor: spawn, pipe draining, timeout.

use super::CodeToolResult;
use crate::core::error::MornError;
use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

pub(super) fn run_windows(
    timeout_secs: u64,
    interpreter: &str,
    file_path: &Path,
) -> Result<CodeToolResult, MornError> {
    let (program, args): (&str, Vec<String>) = if interpreter == "cmd" {
        (
            "cmd",
            vec!["/C".to_string(), file_path.to_string_lossy().into_owned()],
        )
    } else {
        (interpreter, vec![file_path.to_string_lossy().into_owned()])
    };

    let mut child = Command::new(program)
        .args(&args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| MornError::Internal(format!("failed to execute code: {}", e)))?;

    // Read pipes on background threads so a chatty child cannot deadlock on a
    // full pipe buffer while the parent polls for completion.
    let mut stdout_pipe = child.stdout.take().expect("stdout was piped");
    let mut stderr_pipe = child.stderr.take().expect("stderr was piped");
    let stdout_reader = std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = read_to_string(&mut stdout_pipe, &mut buf);
        buf
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut buf = String::new();
        let _ = read_to_string(&mut stderr_pipe, &mut buf);
        buf
    });

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(timeout_secs);
    let mut timed_out = false;
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break Some(status),
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    // SAFETY: kill only the child this runner spawned.
                    let _ = child.kill();
                    let _ = child.wait();
                    timed_out = true;
                    break None;
                }
                std::thread::sleep(std::time::Duration::from_millis(25));
            }
            Err(e) => {
                return Err(MornError::Internal(format!(
                    "failed to wait for code: {}",
                    e
                )))
            }
        }
    };

    let stdout = stdout_reader.join().unwrap_or_default();
    let stderr = stderr_reader.join().unwrap_or_default();
    // COMPATIBILITY: Windows has no unix exit-code-124 timeout convention, so a
    // process killed on expiry reports -1 with timed_out=true. The memory
    // ceiling (ulimit -v) is unix-only; enforcing it here via Job Objects is
    // future work.
    let exit_code = if timed_out {
        -1
    } else {
        status.and_then(|s| s.code()).unwrap_or(-1)
    };

    Ok(CodeToolResult {
        stdout,
        stderr,
        exit_code,
        timed_out,
    })
}

fn read_to_string(reader: &mut impl Read, buf: &mut String) -> std::io::Result<()> {
    std::io::Read::read_to_string(reader, buf).map(|_| ())
}
