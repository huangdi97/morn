//! types — Supervisor type definitions including NLAgentDef, DecisionLevel, and TaskPlan.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::component::persona::{PersonaParameters, PromptLayers};
#[cfg(feature = "computer")]
#[allow(unused_imports)]
pub use crate::computer::SecurityConfig;

fn default_communication_style() -> String {
    "professional".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NLPersonaConfig {
    #[serde(default)]
    pub parameters: PersonaParameters,
    #[serde(default)]
    pub prompt_layers: PromptLayers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLAgentDef {
    pub name: String,
    pub persona: String,
    pub model: String,
    pub tools: Vec<String>,
    pub knowledge: Vec<String>,
    pub skills: Vec<String>,
    #[serde(default)]
    pub memory: Vec<String>,
    #[serde(default)]
    pub persona_config: NLPersonaConfig,
    #[serde(default = "default_communication_style")]
    pub communication_style: String,
    #[serde(default)]
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubTaskDef {
    pub id: String,
    pub agent_id: String,
    pub action: String,
    pub params: Value,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskPlan {
    pub task_id: String,
    pub user_input: String,
    pub subtasks: Vec<SubTaskDef>,
    pub estimated_secs: u64,
    pub decision_level: String,
    #[serde(default)]
    pub approval_required: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubTaskResult {
    pub id: String,
    pub success: bool,
    pub output: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub subtask_results: Vec<SubTaskResult>,
    pub summary: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TurnRecord {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DecisionLevel {
    L1DirectAnswer,
    L2SingleTool,
    L3SingleAgent,
    L4Team,
    L5Workflow,
    L6JumpToStudio,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum DecisionTier {
    Operational, // 运营级: trust>60 + low risk → auto
    Tactical,    // 战术级: suggest + CEO confirm
    Strategic,   // 战略级: must CEO decide
}

impl DecisionTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            DecisionTier::Operational => "operational",
            DecisionTier::Tactical => "tactical",
            DecisionTier::Strategic => "strategic",
        }
    }

    pub fn from_decision_level(level: &DecisionLevel, trust_score: f64) -> Self {
        match level {
            DecisionLevel::L1DirectAnswer => {
                if trust_score > 60.0 {
                    DecisionTier::Operational
                } else {
                    DecisionTier::Tactical
                }
            }
            DecisionLevel::L2SingleTool => {
                if trust_score > 60.0 {
                    DecisionTier::Operational
                } else {
                    DecisionTier::Tactical
                }
            }
            DecisionLevel::L3SingleAgent => {
                if trust_score > 70.0 {
                    DecisionTier::Tactical
                } else {
                    DecisionTier::Strategic
                }
            }
            DecisionLevel::L4Team => DecisionTier::Strategic,
            DecisionLevel::L5Workflow => DecisionTier::Tactical,
            DecisionLevel::L6JumpToStudio => DecisionTier::Strategic,
        }
    }
}

impl DecisionLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            DecisionLevel::L1DirectAnswer => "direct_answer",
            DecisionLevel::L2SingleTool => "single_tool",
            DecisionLevel::L3SingleAgent => "single_agent",
            DecisionLevel::L4Team => "team",
            DecisionLevel::L5Workflow => "workflow",
            DecisionLevel::L6JumpToStudio => "jump_studio",
        }
    }

    pub fn cost_tier(&self) -> &'static str {
        match self {
            DecisionLevel::L1DirectAnswer => "¥0.001/0.5s",
            DecisionLevel::L2SingleTool => "¥0.003/1s",
            DecisionLevel::L3SingleAgent => "¥0.02/5s",
            DecisionLevel::L4Team => "¥0.05/15s",
            DecisionLevel::L5Workflow => "¥0.03/10s",
            DecisionLevel::L6JumpToStudio => "variable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum Mode {
    Proactive,
    Safe,
    Automated,
}

impl Mode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::Proactive => "proactive",
            Mode::Safe => "safe",
            Mode::Automated => "automated",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "active" | "proactive" => Some(Mode::Proactive),
            "safe" => Some(Mode::Safe),
            "auto" | "automated" => Some(Mode::Automated),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum OverrideScope {
    NextTurn,
    Session,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct DecisionOverride {
    pub level: DecisionLevel,
    pub scope: OverrideScope,
}

impl DecisionOverride {
    pub fn parse_prefixed(input: &str) -> Option<(Self, String)> {
        let trimmed = input.trim_start();
        let specs = [
            ("L1:", DecisionLevel::L1DirectAnswer),
            ("L2:", DecisionLevel::L2SingleTool),
            ("L3:", DecisionLevel::L3SingleAgent),
            ("L4:", DecisionLevel::L4Team),
            ("L5:", DecisionLevel::L5Workflow),
            ("L6:", DecisionLevel::L6JumpToStudio),
            ("#level1", DecisionLevel::L1DirectAnswer),
            ("#level2", DecisionLevel::L2SingleTool),
            ("#level3", DecisionLevel::L3SingleAgent),
            ("#level4", DecisionLevel::L4Team),
            ("#level5", DecisionLevel::L5Workflow),
            ("#level6", DecisionLevel::L6JumpToStudio),
        ];

        for (prefix, level) in specs {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                return Some((
                    DecisionOverride {
                        level,
                        scope: OverrideScope::NextTurn,
                    },
                    rest.trim_start_matches([':', ' ', '\t']).to_string(),
                ));
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_tier_operational() {
        let tier = DecisionTier::from_decision_level(&DecisionLevel::L1DirectAnswer, 80.0);
        assert_eq!(tier, DecisionTier::Operational);
    }

    #[test]
    fn test_decision_tier_tactical_low_trust() {
        let tier = DecisionTier::from_decision_level(&DecisionLevel::L1DirectAnswer, 50.0);
        assert_eq!(tier, DecisionTier::Tactical);
    }

    #[test]
    fn test_decision_tier_strategic() {
        let tier = DecisionTier::from_decision_level(&DecisionLevel::L4Team, 90.0);
        assert_eq!(tier, DecisionTier::Strategic);
    }

    #[test]
    fn test_decision_tier_tactical_for_single_agent() {
        let tier = DecisionTier::from_decision_level(&DecisionLevel::L3SingleAgent, 75.0);
        assert_eq!(tier, DecisionTier::Tactical);
    }

    #[test]
    fn test_decision_tier_strategic_for_studio() {
        let tier = DecisionTier::from_decision_level(&DecisionLevel::L6JumpToStudio, 100.0);
        assert_eq!(tier, DecisionTier::Strategic);
    }

    #[test]
    fn test_decision_tier_tactical_for_workflow() {
        let tier = DecisionTier::from_decision_level(&DecisionLevel::L5Workflow, 0.0);
        assert_eq!(tier, DecisionTier::Tactical);
    }

    #[test]
    fn test_decision_tier_as_str() {
        assert_eq!(DecisionTier::Operational.as_str(), "operational");
        assert_eq!(DecisionTier::Tactical.as_str(), "tactical");
        assert_eq!(DecisionTier::Strategic.as_str(), "strategic");
    }
}
