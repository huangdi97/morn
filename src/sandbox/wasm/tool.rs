use super::{Sandbox, SandboxError};

pub struct WasmSandboxTool {
    sandbox: Sandbox,
}

impl WasmSandboxTool {
    pub fn new() -> Result<Self, SandboxError> {
        Ok(WasmSandboxTool { sandbox: Sandbox::new()? })
    }

    pub fn execute_wat(&self, code: &str) -> Result<String, SandboxError> {
        self.sandbox.execute(code)
    }

    pub fn execute_wasm(&self, bytes: &[u8]) -> Result<String, SandboxError> {
        self.sandbox.execute_bytes(bytes)
    }
}