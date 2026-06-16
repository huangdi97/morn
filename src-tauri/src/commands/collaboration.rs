use crate::AppState;
use crate::MornError;
use serde::Serialize;
use tauri::State;

#[derive(Debug, Serialize)]
pub(crate) struct ModeInfo {
    pub id: String,
    pub name: String,
    pub description: String,
}

#[tauri::command]
pub(crate) fn list_collaboration_modes() -> Vec<ModeInfo> {
    vec![
        ModeInfo {
            id: "debate".into(),
            name: "Debate".into(),
            description: "Agents debate over two rounds, building on each other's responses."
                .into(),
        },
        ModeInfo {
            id: "chain".into(),
            name: "Chain".into(),
            description: "Each agent passes its output to the next in sequence.".into(),
        },
        ModeInfo {
            id: "manager_worker".into(),
            name: "ManagerWorker".into(),
            description: "Manager delegates work to workers and synthesizes results.".into(),
        },
        ModeInfo {
            id: "broadcast".into(),
            name: "Broadcast".into(),
            description: "All agents receive the same input simultaneously.".into(),
        },
        ModeInfo {
            id: "voting".into(),
            name: "Voting".into(),
            description: "Agents vote and the highest confidence result wins.".into(),
        },
        ModeInfo {
            id: "round_robin".into(),
            name: "RoundRobin".into(),
            description: "Agents process input sequentially in round-robin order.".into(),
        },
        ModeInfo {
            id: "routing".into(),
            name: "Routing".into(),
            description: "Input is routed to the most suitable agent.".into(),
        },
        ModeInfo {
            id: "agent_as_tool".into(),
            name: "AgentAsTool".into(),
            description: "First agent uses others as tools to complete the task.".into(),
        },
        ModeInfo {
            id: "blackboard".into(),
            name: "Blackboard".into(),
            description: "Agents share a common blackboard, reading and posting updates.".into(),
        },
        ModeInfo {
            id: "consensus".into(),
            name: "Consensus".into(),
            description: "Broadcast followed by automated consensus synthesis.".into(),
        },
        ModeInfo {
            id: "swarm".into(),
            name: "Swarm".into(),
            description: "Multiple broadcast iterations for emergent collective results.".into(),
        },
    ]
}

#[tauri::command]
pub(crate) fn set_team_mode(
    team_id: String,
    mode: String,
    state: State<AppState>,
) -> Result<(), MornError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| MornError::Internal("Storage not initialized".to_string()))?;
    s.set_setting(&format!("team_mode:{}", team_id), &mode)?;
    Ok(())
}

#[tauri::command]
pub(crate) fn get_team_mode(
    team_id: String,
    state: State<AppState>,
) -> Result<Option<String>, MornError> {
    let storage = state
        .storage
        .lock()
        .map_err(|e| MornError::Internal(e.to_string()))?;
    let s = storage
        .as_ref()
        .ok_or_else(|| MornError::Internal("Storage not initialized".to_string()))?;
    let mode = s.get_setting(&format!("team_mode:{}", team_id))?;
    Ok(mode)
}
