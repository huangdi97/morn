use std::process::{Command, Stdio};

use crate::core::mcp::{MCPError, MCPResponse};

/// Call a tool via subprocess stdin/stdout.
pub fn call_stdio(
    command: &str,
    cmd_args: &[String],
    tool_name: &str,
    args: &serde_json::Value,
) -> Result<MCPResponse, MCPError> {
    let input = serde_json::json!({
        "tool": tool_name,
        "params": args,
    });

    let output = Command::new(command)
        .args(cmd_args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                serde_json::to_writer(stdin, &input)?;
            }
            child.wait_with_output()
        })
        .map_err(|e| MCPError(format!("Stdio command failed: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(MCPError(format!("Command exited with error: {stderr}")));
    }

    let data: MCPResponse = serde_json::from_slice(&output.stdout)
        .map_err(|e| MCPError(format!("JSON parse failed: {e}")))?;
    Ok(data)
}