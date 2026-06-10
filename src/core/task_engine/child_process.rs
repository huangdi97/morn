use std::process::{Command, Child, Stdio};
use std::io::{Write, Read};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessStatus {
    Idle,
    Running,
    Terminated,
}

pub struct ChildProcess {
    handle: Option<Child>,
}

impl ChildProcess {
    pub fn new() -> Self {
        ChildProcess { handle: None }
    }

    pub fn is_running(&self) -> bool {
        self.handle.is_some()
    }

    pub fn status(&self) -> ProcessStatus {
        match self.handle {
            Some(_) => ProcessStatus::Running,
            None => ProcessStatus::Idle,
        }
    }

    pub fn spawn(task_json: &str, _timeout_secs: u64) -> Result<Self, String> {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let mut child = Command::new(exe)
            .arg("--execute-task")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;

        if let Some(ref mut stdin) = child.stdin {
            stdin.write_all(task_json.as_bytes()).map_err(|e| e.to_string())?;
        }

        let handle = Some(child);
        Ok(ChildProcess { handle })
    }

    pub fn wait(&mut self, timeout_secs: u64) -> Result<String, String> {
        let mut child = self.handle.take().ok_or("No child process")?;
        let start = std::time::Instant::now();

        loop {
            if start.elapsed() > Duration::from_secs(timeout_secs) {
                let _ = child.kill();
                return Err("Timeout".to_string());
            }
            match child.try_wait() {
                Ok(Some(_)) => {
                    let mut output = String::new();
                    child.stdout.take().expect("child process stdout should be piped").read_to_string(&mut output).ok();
                    return Ok(output);
                }
                Ok(None) => std::thread::sleep(Duration::from_millis(50)),
                Err(e) => return Err(e.to_string()),
            }
        }
    }

    pub fn kill(&mut self) -> Result<(), String> {
        if let Some(ref mut child) = self.handle {
            child.kill().map_err(|e| e.to_string())?;
            child.wait().ok();
        }
        Ok(())
    }

    pub fn is_alive(&mut self) -> bool {
        self.handle.as_mut().is_some_and(|c| {
            c.try_wait().map_or(true, |s| s.is_none())
        })
    }
}
