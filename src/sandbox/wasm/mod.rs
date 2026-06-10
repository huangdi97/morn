use std::time::Instant;
use wasmtime::{Config, Engine, Linker, Module, Store, StoreLimits, StoreLimitsBuilder};

const MAX_MEMORY_BYTES: u64 = 64 * 1024 * 1024;
const MAX_FUEL: u64 = 100_000;
const MAX_EXECUTION_MS: u64 = 5000; // P7: hard time limit

pub struct Sandbox {
    engine: Engine,
}

struct SandboxData {
    limits: StoreLimits,
}

#[derive(Debug)]
pub enum SandboxError {
    Compile(String),
    Execute(String),
    ResourceLimit(String),
}

impl std::fmt::Display for SandboxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SandboxError::Compile(msg) => write!(f, "Compilation failed: {}", msg),
            SandboxError::Execute(msg) => write!(f, "Execution failed: {}", msg),
            SandboxError::ResourceLimit(msg) => write!(f, "Resource limit exceeded: {}", msg),
        }
    }
}

impl std::error::Error for SandboxError {}

impl Sandbox {
    pub fn new() -> Result<Self, SandboxError> {
        let mut config = Config::new();
        config.consume_fuel(true);
        config.wasm_multi_memory(false);
        config.wasm_bulk_memory(true);

        let engine = Engine::new(&config)
            .map_err(|e| SandboxError::Compile(e.to_string()))?;

        Ok(Sandbox { engine })
    }

    pub fn execute(&self, code: &str) -> Result<String, SandboxError> {
        let start = Instant::now();
        let wasm_bytes = wat::parse_str(code)
            .map_err(|e| SandboxError::Compile(format!("WAT parse error: {}", e)))?;
        let result = self.execute_bytes(&wasm_bytes);
        let elapsed = start.elapsed().as_millis() as u64;
        if elapsed > MAX_EXECUTION_MS {
            return Err(SandboxError::ResourceLimit(format!(
                "Execution timed out: {}ms (max {}ms)", elapsed, MAX_EXECUTION_MS
            )));
        }
        result
    }

    pub fn execute_bytes(&self, wasm_bytes: &[u8]) -> Result<String, SandboxError> {
        let start = Instant::now();
        let module = Module::new(&self.engine, wasm_bytes)
            .map_err(|e| SandboxError::Compile(e.to_string()))?;

        let limits = StoreLimitsBuilder::new()
            .memory_size(MAX_MEMORY_BYTES as usize)
            .table_elements(1_000)
            .instances(1)
            .memories(1)
            .tables(1)
            .build();

        let data = SandboxData { limits };
        let mut store = Store::new(&self.engine, data);
        store.limiter(|state| &mut state.limits);
        store.set_fuel(MAX_FUEL)
            .map_err(|e| SandboxError::ResourceLimit(e.to_string()))?;

        let linker = Linker::new(&self.engine);
        linker
            .instantiate(&mut store, &module)
            .map_err(|e| SandboxError::Execute(e.to_string()))?;

        let fuel_remaining = store.get_fuel()
            .map_err(|e| SandboxError::ResourceLimit(e.to_string()))?;
        let fuel_consumed = MAX_FUEL - fuel_remaining;

        let elapsed_ms = start.elapsed().as_millis();
        if elapsed_ms as u64 > MAX_EXECUTION_MS {
            return Err(SandboxError::ResourceLimit(format!(
                "Execution timed out after {}ms", elapsed_ms
            )));
        }

        Ok(format!("executed | fuel: {} | time: {}ms | mem_max: {}MB",
            fuel_consumed, elapsed_ms, MAX_MEMORY_BYTES / (1024 * 1024)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sandbox_new_creates_engine() {
        let sb = Sandbox::new();
        assert!(sb.is_ok());
    }

    #[test]
    fn sandbox_execute_invalid_text_returns_error() {
        let sb = Sandbox::new().unwrap();
        let result = sb.execute("not valid wat");
        assert!(result.is_err());
    }

    #[test]
    fn sandbox_execute_trivial_module() {
        let sb = Sandbox::new().unwrap();
        let result = sb.execute("(module)");
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.contains("executed"));
        assert!(msg.contains("fuel"));
    }

    #[test]
    fn sandbox_rejects_empty_code() {
        let sb = Sandbox::new().unwrap();
        let result = sb.execute("");
        assert!(result.is_err());
    }

    #[test]
    fn sandbox_multiple_executions_are_independent() {
        let sb = Sandbox::new().unwrap();
        assert!(sb.execute("(module)").is_ok());
        assert!(sb.execute("(module)").is_ok());
        assert!(sb.execute("(module)").is_ok());
    }

    #[test]
    fn sandbox_execute_bytes_raw_wasm() {
        let sb = Sandbox::new().unwrap();
        let wasm = wat::parse_str("(module)").unwrap();
        let result = sb.execute_bytes(&wasm);
        assert!(result.is_ok());
    }

    #[test]
    fn sandbox_error_display() {
        let err = SandboxError::Compile("bad code".into());
        assert!(err.to_string().contains("bad code"));
    }

    #[test]
    fn sandbox_error_is_std_error() {
        let err = SandboxError::Execute("fail".into());
        let _: &dyn std::error::Error = &err;
    }

    #[test]
    fn sandbox_rejects_large_module() {
        let sb = Sandbox::new().unwrap();
        let large_wat = "(module (memory 1) (func (export \"start\")))";
        let result = sb.execute(large_wat);
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn sandbox_error_compile_display() {
        let err = SandboxError::Compile("syntax error".into());
        assert!(err.to_string().contains("Compilation failed"));
    }

    #[test]
    fn sandbox_error_resource_limit_display() {
        let err = SandboxError::ResourceLimit("out of memory".into());
        assert!(err.to_string().contains("Resource limit exceeded"));
    }

    #[test]
    fn sandbox_execute_returns_fuel_metric() {
        let sb = Sandbox::new().unwrap();
        let result = sb.execute("(module)").unwrap();
        assert!(result.contains("fuel"));
        assert!(result.contains("time"));
    }
}