use crate::core::supervisor::{DecisionLevel, Intent, Supervisor};

impl Supervisor {
    pub fn decide_weighted(&self, intent: &Intent) -> DecisionLevel {
        if let Some(forced_level) = forced_level_from_intent(intent) {
            return forced_level;
        }

        let levels = [
            DecisionLevel::L1DirectAnswer,
            DecisionLevel::L2SingleTool,
            DecisionLevel::L3SingleAgent,
            DecisionLevel::L4Team,
            DecisionLevel::L5Workflow,
            DecisionLevel::L6JumpToStudio,
        ];

        levels
            .into_iter()
            .max_by(|left, right| {
                weighted_level_score(intent, left)
                    .partial_cmp(&weighted_level_score(intent, right))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .unwrap_or(DecisionLevel::L3SingleAgent)
    }
}

pub(crate) fn forced_level_from_intent(intent: &Intent) -> Option<DecisionLevel> {
    let text = format!(
        "{} {} {}",
        intent.intent_type,
        intent.target_agent,
        intent.required_tools.join(" ")
    )
    .to_lowercase();
    super::level::forced_level_from_text(&text)
}

pub(crate) fn weighted_level_score(intent: &Intent, level: &DecisionLevel) -> f64 {
    let complexity = normalized_complexity_fit(intent.complexity, level);
    let cost = normalized_cost_score(level);
    let preference = normalized_preference_score(intent, level);
    complexity * 0.4 + cost * 0.3 + preference * 0.3
}

pub(crate) fn is_advanced_level(level: &DecisionLevel) -> bool {
    matches!(
        level,
        DecisionLevel::L4Team | DecisionLevel::L5Workflow | DecisionLevel::L6JumpToStudio
    )
}

pub(crate) fn is_low_level(level: &DecisionLevel) -> bool {
    matches!(
        level,
        DecisionLevel::L1DirectAnswer | DecisionLevel::L2SingleTool
    )
}

fn normalized_complexity_fit(intent_complexity: u8, level: &DecisionLevel) -> f64 {
    let intent_complexity = intent_complexity.clamp(1, 10) as f64;
    let level_complexity = super::complexity_for_level(level) as f64;
    (10.0 - (intent_complexity - level_complexity).abs()).clamp(0.0, 10.0)
}

fn normalized_cost_score(level: &DecisionLevel) -> f64 {
    match level {
        DecisionLevel::L1DirectAnswer => 10.0,
        DecisionLevel::L2SingleTool => 9.0,
        DecisionLevel::L3SingleAgent => 6.0,
        DecisionLevel::L5Workflow => 5.0,
        DecisionLevel::L4Team => 3.0,
        DecisionLevel::L6JumpToStudio => 1.0,
    }
}

fn normalized_preference_score(intent: &Intent, level: &DecisionLevel) -> f64 {
    let intent_type = super::normalize_intent_type(&intent.intent_type);
    if intent_type == level.as_str() {
        return 10.0;
    }

    let target_agent = intent.target_agent.trim().to_lowercase();
    let target_match = match level {
        DecisionLevel::L1DirectAnswer => target_agent == "assistant",
        DecisionLevel::L2SingleTool => target_agent.contains("tool"),
        DecisionLevel::L3SingleAgent => {
            target_agent.contains("agent") && !target_agent.contains("team")
        }
        DecisionLevel::L4Team => target_agent.contains("team"),
        DecisionLevel::L5Workflow => target_agent.contains("workflow"),
        DecisionLevel::L6JumpToStudio => target_agent.contains("studio"),
    };
    if target_match {
        return 8.0;
    }

    if !intent.required_tools.is_empty() {
        return match level {
            DecisionLevel::L2SingleTool => 7.0,
            DecisionLevel::L3SingleAgent => 6.0,
            DecisionLevel::L5Workflow => 5.0,
            _ => 3.0,
        };
    }

    5.0
}