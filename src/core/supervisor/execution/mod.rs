pub mod dispatch;
pub mod dual_llm;
pub mod events;
pub mod planner;
pub mod scheduler;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ExecutionTier {
    Direct,
    Interactive,
    Background,
}

pub fn classify_execution_level(decision_level: &str) -> Option<ExecutionTier> {
    match decision_level {
        "direct_answer" => Some(ExecutionTier::Direct),
        "single_agent" => Some(ExecutionTier::Interactive),
        "team" => Some(ExecutionTier::Background),
        _ => None,
    }
}

pub fn classify_execution_time(estimated_secs: u64) -> ExecutionTier {
    if estimated_secs < 3 {
        ExecutionTier::Direct
    } else if estimated_secs <= 30 {
        ExecutionTier::Interactive
    } else {
        ExecutionTier::Background
    }
}

#[cfg(test)]
mod tests;
