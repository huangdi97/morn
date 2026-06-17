use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TeamTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub members: Vec<String>,
    pub mode: String,
    pub consensus: String,
}

#[tauri::command]
pub(crate) fn list_team_templates() -> Result<Vec<TeamTemplate>, String> {
    let presets = morn::core::orchestrator::team_presets::get_presets();
    Ok(presets
        .into_iter()
        .map(|p| TeamTemplate {
            id: p.team.id,
            name: p.team.name.clone(),
            description: format!("Pre-built team template: {}", p.team.name),
            members: p.team.members,
            mode: format!("{:?}", p.team.mode),
            consensus: format!("{:?}", p.team.consensus),
        })
        .collect())
}
