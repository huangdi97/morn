use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use uuid::Uuid;

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
    max_memory_mb: u64,
    forbidden_patterns: Vec<String>,
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

    pub fn add_forbidden_pattern(&mut self, pattern: &str) {
        self.forbidden_patterns.push(pattern.to_string());
    }

    pub fn execute(&self, code: &str, language: &str) -> Result<CodeToolResult, String> {
        let ext = match language {
            "python" | "py" => "py",
            "shell" | "sh" | "bash" => "sh",
            other => return Err(format!("unsupported language: {}", other)),
        };

        let code_lower = code.to_lowercase();
        for pattern in &self.forbidden_patterns {
            if code_lower.contains(&pattern.to_lowercase()) {
                return Err(format!(
                    "dangerous pattern detected: '{}' is not allowed",
                    pattern
                ));
            }
        }

        let file_name = format!("code_tool_{}.{}", Uuid::new_v4(), ext);
        let temp_dir = std::env::temp_dir().join("morn_code_tool");
        fs::create_dir_all(&temp_dir).map_err(|e| format!("failed to create temp dir: {}", e))?;
        let file_path = temp_dir.join(&file_name);

        let mut file = fs::File::create(&file_path)
            .map_err(|e| format!("failed to create temp file: {}", e))?;
        file.write_all(code.as_bytes())
            .map_err(|e| format!("failed to write code to temp file: {}", e))?;
        drop(file);

        let result = if ext == "py" {
            self.run_with_limits("python3", &file_path)
                .or_else(|_| self.run_with_limits("python", &file_path))
        } else {
            self.run_with_limits("bash", &file_path)
        };

        let _ = fs::remove_file(&file_path);

        result
    }

    fn run_with_limits(
        &self,
        interpreter: &str,
        file_path: &Path,
    ) -> Result<CodeToolResult, String> {
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
            .map_err(|e| format!("failed to execute code: {}", e))?;

        let exit_code = output.status.code().unwrap_or(-1);
        let timed_out = exit_code == 124;

        Ok(CodeToolResult {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code,
            timed_out,
        })
    }
}

#[cfg(test)]
mod tests {
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
        let result = executor.execute("sleep 10", "sh").unwrap();
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
}
