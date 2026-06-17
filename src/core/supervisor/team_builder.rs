//! team_builder — Natural-language team creation for Supervisor.
use crate::core::error::MornError;
use crate::core::orchestrator::team_presets;
use crate::core::orchestrator::{CollaborationMode, ConsensusMechanism, TeamDef};

use super::{ChatFn, Supervisor};

impl Supervisor {
    /// Determines whether the user needs a single agent or a full team,
    /// then creates a TeamDef via keyword matching on presets or LLM generation.
    pub fn create_team_from_nl(&self, nl: &str, chat_fn: &ChatFn) -> Result<TeamDef, MornError> {
        let system_prompt = "You are a team configuration assistant. Determine if the user needs a single agent or a multi-agent team. Reply with exactly one line: either 'SINGLE' or 'TEAM'.";
        let response = chat_fn(nl, system_prompt)?;
        let trimmed = response.trim().to_uppercase();

        if trimmed.starts_with("SINGLE") {
            return Err(MornError::Internal(
                "Single agent sufficient, no team needed".to_string(),
            ));
        }

        if let Some(preset) = team_presets::find_preset(nl) {
            return Ok(preset);
        }

        let gen_prompt = format!(
            r#"The user wants to form a team for this task:
{}
Generate a TeamDef JSON. Available collaboration modes: Chain, ManagerWorker, Broadcast, Voting, Routing, AgentAsTool, Blackboard.
Available consensus mechanisms: Vote, CeoDecides, MungerVeto, AutoSynthesis.
Return only valid JSON with fields: id, name, members (string array of agent IDs), mode, consensus.
Example:
{{"id":"team-custom","name":"Custom Team","members":["agent-a","agent-b"],"mode":"Chain","consensus":"CeoDecides"}}"#,
            nl
        );
        let gen_response = chat_fn(&gen_prompt, "Only return valid JSON, no markdown.")?;
        let cleaned = gen_response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        #[derive(serde::Deserialize)]
        struct GenTeamDef {
            id: String,
            name: String,
            members: Vec<String>,
            mode: String,
            consensus: String,
        }

        let gen: GenTeamDef = serde_json::from_str(cleaned).map_err(|e| {
            MornError::Internal(format!(
                "Failed to parse LLM team response: {}. Raw: {}",
                e, cleaned
            ))
        })?;

        let mode = match gen.mode.to_lowercase().as_str() {
            "chain" => CollaborationMode::Chain,
            "managerworker" | "manager_worker" => CollaborationMode::ManagerWorker,
            "broadcast" => CollaborationMode::Broadcast,
            "voting" => CollaborationMode::Voting,
            "routing" => CollaborationMode::Routing,
            "agentastool" | "agent_as_tool" => CollaborationMode::AgentAsTool,
            "blackboard" => CollaborationMode::Blackboard,
            _ => CollaborationMode::Chain,
        };

        let consensus = match gen.consensus.to_lowercase().as_str() {
            "vote" => ConsensusMechanism::Vote,
            "ceodecides" | "ceo_decides" => ConsensusMechanism::CeoDecides,
            "mungerveto" | "munger_veto" => ConsensusMechanism::MungerVeto,
            "autosynthesis" | "auto_synthesis" => ConsensusMechanism::AutoSynthesis,
            _ => ConsensusMechanism::CeoDecides,
        };

        Ok(TeamDef {
            id: gen.id,
            name: gen.name,
            members: gen.members,
            mode,
            consensus,
        })
    }
}
