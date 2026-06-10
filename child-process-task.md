在 src/core/task_engine/ 下新建 child_process.rs，内容如下。然后在 src/core/task_engine/mod.rs 加 pub mod child_process;。完成后 cargo build。

```rust
use std::process::{Command, Child, Stdio};
use std::io::{Write, Read};
use std::time::Duration;

pub struct ChildProcess {
    handle: Option<Child>,
}

impl ChildProcess {
    pub fn spawn(task_json: &str, timeout_secs: u64) -> Result<Self, String> {
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
                    child.stdout.take().unwrap().read_to_string(&mut output).ok();
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

    pub fn is_alive(&self) -> bool {
        self.handle.as_ref().map_or(false, |c| {
            c.try_wait().map_or(true, |s| s.is_none())
        })
    }
}
```
