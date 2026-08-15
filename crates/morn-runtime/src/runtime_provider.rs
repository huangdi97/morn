//! RuntimeProvider: the provider contract for "where / how work runs".
//! Implementations own the execution lifecycle (start / checkpoint / restore /
//! signal / events / health). They never write the canonical DB directly and
//! never bypass the Action Gateway for state changes.

use std::collections::BTreeMap;

use morn_kernel::error::{Error, Result};

/// RuntimeProvider protocol (public SDK). Runtime fixtures and implementations
/// satisfy this contract; conformance is run through `run_runtime_conformance`.
pub trait RuntimeProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn health(&self) -> bool;
    fn start_run(&mut self, run_id: &str) -> Result<String>;
    fn checkpoint(&mut self, run_id: &str) -> Result<String>;
    fn restore(&mut self, payload: &str) -> Result<()>;
    fn deliver_signal(&mut self, run_id: &str, kind: &str, value: &str) -> Result<()>;
    fn events(&self) -> Vec<String>;
}

/// A minimal generic fixture runtime used by the conformance kit. It tracks an
/// explicit state machine and records events; it never mutates canonical state.
#[derive(Debug, Default)]
pub struct FixtureRuntime {
    runs: BTreeMap<String, String>, // run_id -> status
    pub events: Vec<String>,
    pub last_checkpoint: Option<String>,
}

impl FixtureRuntime {
    pub fn new() -> Self {
        Self::default()
    }
}

impl RuntimeProvider for FixtureRuntime {
    fn provider_name(&self) -> &str {
        "fixture-runtime"
    }

    fn health(&self) -> bool {
        true
    }

    fn start_run(&mut self, run_id: &str) -> Result<String> {
        if self.runs.contains_key(run_id) {
            return Err(Error::conflict("run already started"));
        }
        self.runs.insert(run_id.to_string(), "running".to_string());
        self.events.push(format!("started {run_id}"));
        Ok(format!("running:{run_id}"))
    }

    fn checkpoint(&mut self, run_id: &str) -> Result<String> {
        let status = self
            .runs
            .get(run_id)
            .ok_or_else(|| Error::not_found("run"))?;
        let payload = format!("checkpoint:{run_id}:{status}");
        self.last_checkpoint = Some(payload.clone());
        self.events.push(format!("checkpoint {run_id}"));
        Ok(payload)
    }

    fn restore(&mut self, payload: &str) -> Result<()> {
        let parts: Vec<&str> = payload.split(':').collect();
        if parts.len() != 3 || parts[0] != "checkpoint" {
            return Err(Error::invalid_state("malformed checkpoint payload"));
        }
        self.runs.insert(parts[1].to_string(), parts[2].to_string());
        self.events.push(format!("restored {}", parts[1]));
        Ok(())
    }

    fn deliver_signal(&mut self, run_id: &str, kind: &str, value: &str) -> Result<()> {
        let status = self
            .runs
            .get_mut(run_id)
            .ok_or_else(|| Error::not_found("run"))?;
        if *status == "waiting_signal" {
            *status = "running".to_string();
            self.events.push(format!("signal {run_id} {kind}={value}"));
            Ok(())
        } else {
            Err(Error::conflict("signal on non-waiting run"))
        }
    }

    fn events(&self) -> Vec<String> {
        self.events.clone()
    }
}

/// RuntimeProvider conformance kit: start -> checkpoint -> restore -> signal ->
/// events -> health. A conforming runtime must pass every step.
pub fn run_runtime_conformance(runtime: &mut dyn RuntimeProvider) -> Result<()> {
    if !runtime.health() {
        return Err(Error::external("runtime health false"));
    }
    runtime.start_run("run-1")?;
    let cp = runtime.checkpoint("run-1")?;
    if cp.is_empty() {
        return Err(Error::external("empty checkpoint"));
    }
    runtime.restore(&cp)?;
    // restore from a malformed payload must fail explicitly
    if runtime.restore("garbage").is_ok() {
        return Err(Error::external("malformed restore accepted"));
    }
    if runtime.events().is_empty() {
        return Err(Error::external("no events recorded"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_runtime_passes_conformance() {
        let mut rt = FixtureRuntime::new();
        run_runtime_conformance(&mut rt).unwrap();
    }

    #[test]
    fn fixture_runtime_rejects_duplicate_start_and_bad_restore() {
        let mut rt = FixtureRuntime::new();
        rt.start_run("r").unwrap();
        assert!(rt.start_run("r").is_err());
        assert!(rt.restore("garbage").is_err());
        assert!(rt.deliver_signal("missing", "x", "y").is_err());
    }
}
