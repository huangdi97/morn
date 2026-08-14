//! ExecutionMode: classify the nature of work before choosing an executor.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum WorkNature {
    Deterministic,
    Probabilistic,
    Physical,
    Regulated,
    Social,
    Mixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ExecutorType {
    Program,
    Actor,
    Human,
    ExternalService,
    Device,
    Hybrid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionMode {
    pub nature: Vec<WorkNature>,
    pub executor: Vec<ExecutorType>,
    pub rationale: String,
    pub fallback: Option<ExecutorType>,
}

impl ExecutionMode {
    pub fn deterministic(executor: ExecutorType) -> Self {
        Self {
            nature: vec![WorkNature::Deterministic],
            executor: vec![executor],
            rationale: "deterministic work prefers program/solver execution".to_string(),
            fallback: None,
        }
    }

    pub fn hybrid(natures: Vec<WorkNature>, executors: Vec<ExecutorType>) -> Self {
        Self {
            nature: natures,
            executor: executors,
            rationale: "mixed work requires a hybrid execution cell".to_string(),
            fallback: None,
        }
    }

    pub fn is_deterministic(&self) -> bool {
        self.nature.contains(&WorkNature::Deterministic)
    }

    pub fn has_executor(&self, executor: ExecutorType) -> bool {
        self.executor.contains(&executor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_mode_classification() {
        let det = ExecutionMode::deterministic(ExecutorType::Program);
        assert!(det.is_deterministic());
        assert!(det.has_executor(ExecutorType::Program));

        let hybrid = ExecutionMode::hybrid(
            vec![WorkNature::Probabilistic, WorkNature::Regulated],
            vec![ExecutorType::Actor, ExecutorType::Human],
        );
        assert!(hybrid.has_executor(ExecutorType::Human));
        assert!(!hybrid.has_executor(ExecutorType::Device));
    }
}
