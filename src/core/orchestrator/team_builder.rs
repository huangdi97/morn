//! team_builder — Converts short natural-language requests into team definitions.
use crate::core::error::MornError;
use super::{team_presets, CollaborationMode, ConsensusMechanism, TeamDef};

/// Builds a team from a natural-language description using preset keyword matching.
pub fn nl_to_team(input: &str) -> Result<TeamDef, MornError> {
    let normalized = input.trim().to_lowercase();
    if normalized.is_empty() {
        return Err(MornError::Internal("team description is empty".to_string()))
    }

    if let Some(team) = team_presets::find_preset(&normalized) {
        return Ok(team);
    }

    let mode = if contains_any(&normalized, &["handoff", "step", "pipeline"]) {
        CollaborationMode::Chain
    } else if contains_any(&normalized, &["vote", "review", "quality"]) {
        CollaborationMode::Voting
    } else if contains_any(&normalized, &["route", "triage", "ticket"]) {
        CollaborationMode::Routing
    } else {
        CollaborationMode::ManagerWorker
    };

    let members = if contains_any(&normalized, &["plan", "manage", "coordinate"]) {
        vec![
            "agent-decision".to_string(),
            "agent-execution".to_string(),
            "agent-evaluation".to_string(),
        ]
    } else {
        vec![
            "agent-lead".to_string(),
            "agent-specialist".to_string(),
            "agent-reviewer".to_string(),
        ]
    };

    Ok(TeamDef {
        id: "team-nl-generated".to_string(),
        name: "Generated Team".to_string(),
        members,
        mode,
        consensus: ConsensusMechanism::AutoSynthesis,
    })
}

fn contains_any(input: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|keyword| input.contains(keyword))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nl_to_team_uses_keyword_preset() {
        let team = nl_to_team("need devops deployment monitoring").unwrap();

        assert_eq!(team.id, "preset-devops");
        assert_eq!(team.members.len(), 3);
    }

    #[test]
    fn nl_to_team_rejects_empty_input() {
        let err = nl_to_team("   ").unwrap_err();

        assert!(err.contains("empty"));
    }

    #[test]
    fn nl_to_team_builds_generated_management_shape() {
        let team = nl_to_team("assemble a custom planning group").unwrap();

        assert_eq!(team.id, "team-nl-generated");
        assert_eq!(team.members[0], "agent-decision");
    }
}
