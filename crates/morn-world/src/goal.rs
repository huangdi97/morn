//! Goals and metrics: why actions happen and how to measure them.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{GoalId, MetricId, WorkspaceId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Goal {
    pub id: GoalId,
    pub workspace_id: WorkspaceId,
    pub statement: String,
    pub metric_ids: Vec<MetricId>,
    pub created_at: Timestamp,
}

impl Goal {
    pub fn new(
        workspace_id: WorkspaceId,
        statement: impl Into<String>,
        metric_ids: Vec<MetricId>,
    ) -> Self {
        Self {
            id: GoalId::generate_with("goal"),
            workspace_id,
            statement: statement.into(),
            metric_ids,
            created_at: Timestamp::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Metric {
    pub id: MetricId,
    pub workspace_id: WorkspaceId,
    pub name: String,
    pub target: Option<String>,
    pub actual: Option<String>,
    pub unit: Option<String>,
    pub created_at: Timestamp,
}

impl Metric {
    pub fn new(workspace_id: WorkspaceId, name: impl Into<String>) -> Self {
        Self {
            id: MetricId::generate_with("met"),
            workspace_id,
            name: name.into(),
            target: None,
            actual: None,
            unit: None,
            created_at: Timestamp::now(),
        }
    }
}
