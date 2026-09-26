//! code_tool — Provides code-oriented file inspection and execution helpers.
use crate::core::error::MornError;
use std::fs;
use std::io::Write;
use std::path::Path;
#[cfg(unix)]
use std::process::Command;

use uuid::Uuid;

#[cfg(not(unix))]
#[path = "code_tool_runner.rs"]
mod runner;

#[cfg(feature = "sandbox")]
#[allow(unused_imports)]
use crate::sandbox::wasm::Sandbox;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CodeToolResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub timed_out: bool,
}

#[derive(Debug, Clone)]
pub struct CodeToolExecutor {
    timeout_secs: u64,
    #[cfg_attr(not(unix), allow(dead_code))] // memory ceiling is only read by run_unix
    max_memory_mb: u64,
    forbidden_patterns: Vec<String>,
    #[cfg(feature = "sandbox")]
    #[allow(dead_code)]
    pub sandbox: Option<Sandbox>,
    #[cfg(not(feature = "sandbox"))]
    #[allow(dead_code)]
    pub sandbox: Option<()>,
}

impl Default for CodeToolExecutor {
    fn default() -> Self {
        Self {
            timeout_secs: 5,
            max_memory_mb: 256,
            forbidden_patterns: vec![
                "rm -rf /".to_string(),
                "rm -r /".to_string(),
                "rm -rf /*".to_string(),
                ":(){".to_string(),
                "dd if=".to_string(),
                "mkfs".to_string(),
                "fdisk".to_string(),
                "chmod 777 /".to_string(),
                "> /dev/sda".to_string(),
                "wget.*|.*bash".to_string(),
                "curl.*|.*bash".to_string(),
                "eval.*wget".to_string(),
                "eval.*curl".to_string(),
            ],
            sandbox: None,
        }
    }
}

impl CodeToolExecutor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn with_max_memory(mut self, mb: u64) -> Self {
        self.max_memory_mb = mb;
        self
    }

    #[cfg(feature = "sandbox")]
    pub fn with_sandbox(mut self, sandbox: Sandbox) -> Self {
        self.sandbox = Some(sandbox);
        self
    }

    pub fn add_forbidden_pattern(&mut self, pattern: &str) {
        self.forbidden_patterns.push(pattern.to_string());
    }

    pub fn execute(&self, code: &str, language: &str) -> Result<CodeToolResult, MornError> {
        let ext = match language {
            "python" | "py" => "py",
            // COMPATIBILITY: Windows has no `.sh` file association and no
            // `timeout`/`ulimit`; shell scripts run as .cmd batch files there.
            #[cfg(windows)]
            "shell" | "sh" | "bash" => "cmd",
            #[cfg(not(windows))]
            "shell" | "sh" | "bash" => "sh",
            other => {
                return Err(MornError::Internal(format!(
                    "unsupported language: {}",
                    other
                )))
            }
        };

        let code_lower = code.to_lowercase();
        for pattern in &self.forbidden_patterns {
            if code_lower.contains(&pattern.to_lowercase()) {
                return Err(MornError::Internal(format!(
                    "dangerous pattern detected: '{}' is not allowed",
                    pattern
                )));
            }
        }

        #[cfg(feature = "sandbox")]
        if let Some(ref sb) = self.sandbox {
            match sb.execute(code) {
                Ok(output) => {
                    return Ok(CodeToolResult {
                        stdout: output,
                        stderr: String::new(),
                        exit_code: 0,
                        timed_out: false,
                    });
                }
                Err(e) => {
                    tracing::debug!(
                        "sandbox execution failed, falling back to subprocess: {}",
                        e
                    );
                }
            }
        }

        let file_name = format!("code_tool_{}.{}", Uuid::new_v4(), ext);
        let temp_dir = std::env::temp_dir().join("morn_code_tool");
        fs::create_dir_all(&temp_dir)
            .map_err(|e| MornError::Internal(format!("failed to create temp dir: {}", e)))?;
        let file_path = temp_dir.join(&file_name);

        // COMPATIBILITY: cmd.exe batch parsing is line-ending sensitive;
        // write shell text as CRLF on Windows.
        let content = if ext == "cmd" {
            code.replace("\r\n", "\n").replace('\n', "\r\n")
        } else {
            code.to_string()
        };

        let mut file = fs::File::create(&file_path)
            .map_err(|e| MornError::Internal(format!("failed to create temp file: {}", e)))?;
        file.write_all(content.as_bytes()).map_err(|e| {
            MornError::Internal(format!("failed to write code to temp file: {}", e))
        })?;
        drop(file);

        let candidates: &[&str] = if ext == "py" {
            // COMPATIBILITY: on Windows the `python3` alias is a Microsoft
            // Store stub; real installs expose `python.exe`/`py.exe`.
            #[cfg(windows)]
            {
                &["python", "py"]
            }
            #[cfg(not(windows))]
            {
                &["python3", "python"]
            }
        } else {
            #[cfg(windows)]
            {
                &["cmd"]
            }
            #[cfg(not(windows))]
            {
                &["bash"]
            }
        };

        let mut last_err: Option<MornError> = None;
        for candidate in candidates {
            match self.run_with_limits(candidate, &file_path) {
                Ok(result) => {
                    let _ = fs::remove_file(&file_path);
                    return Ok(result);
                }
                Err(e) => last_err = Some(e),
            }
        }
        let _ = fs::remove_file(&file_path);
        Err(last_err.unwrap_or_else(|| MornError::Internal("no interpreter available".into())))
    }

    fn run_with_limits(
        &self,
        interpreter: &str,
        file_path: &Path,
    ) -> Result<CodeToolResult, MornError> {
        #[cfg(unix)]
        return self.run_unix(interpreter, file_path);
        #[cfg(not(unix))]
        return self.run_windows(interpreter, file_path);
    }

    #[cfg(unix)]
    fn run_unix(&self, interpreter: &str, file_path: &Path) -> Result<CodeToolResult, MornError> {
        let mem_limit_kb = self.max_memory_mb * 1024;

        let output = Command::new("timeout")
            .arg(self.timeout_secs.to_string())
            .arg("bash")
            .arg("-c")
            .arg(format!(
                "ulimit -v {} && exec {} {}",
                mem_limit_kb,
                interpreter,
                file_path.to_string_lossy()
            ))
            .output()
            .map_err(|e| MornError::Internal(format!("failed to execute code: {}", e)))?;

        let exit_code = output.status.code().unwrap_or(-1);
        let timed_out = exit_code == 124;

        Ok(CodeToolResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code,
            timed_out,
        })
    }

    #[cfg(not(unix))]
    fn run_windows(
        &self,
        interpreter: &str,
        file_path: &Path,
    ) -> Result<CodeToolResult, MornError> {
        runner::run_windows(self.timeout_secs, interpreter, file_path)
    }
}

#[cfg(test)]
#[path = "code_tool_tests.rs"]
mod tests;
