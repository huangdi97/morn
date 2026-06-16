//! visibility — Applies capability visibility rules for owners, teams, and public access.
use crate::core::error::MornError;
use super::Capability;

pub(super) fn is_capability_visible(
    capability: &Capability,
    user_id: Option<&str>,
    user_teams: &[String],
) -> bool {
    match capability.visibility.as_str() {
        "public" => true,
        "private" => user_id
            .map(|uid| capability.owner_id.as_deref() == Some(uid))
            .unwrap_or(false),
        "team" => capability
            .team_id
            .as_ref()
            .map(|tid| user_teams.iter().any(|ut| ut == tid))
            .unwrap_or(false),
        _ => true,
    }
}
