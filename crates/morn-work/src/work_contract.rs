//! WorkContract and OutcomeContract.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::{OutcomeContractId, WorkContractId, WorkPackageId, WorkspaceId};
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum BillingBasis {
    Fixed,
    Usage,
    Milestone,
    Outcome,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeContract {
    pub id: OutcomeContractId,
    pub workspace_id: WorkspaceId,
    pub deliverable: String,
    pub quality_slo: String,
    pub deadline: Option<Timestamp>,
    pub acceptance_method: String,
    pub business_or_scientific_metric: String,
    pub target: String,
    pub billing_basis: Vec<BillingBasis>,
    pub human_fallback: bool,
    pub evidence_required: Vec<String>,
}

impl OutcomeContract {
    pub fn new(
        workspace_id: WorkspaceId,
        deliverable: impl Into<String>,
        acceptance_method: impl Into<String>,
        metric: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            id: OutcomeContractId::generate_with("outc"),
            workspace_id,
            deliverable: deliverable.into(),
            quality_slo: "".to_string(),
            deadline: None,
            acceptance_method: acceptance_method.into(),
            business_or_scientific_metric: metric.into(),
            target: target.into(),
            billing_basis: vec![BillingBasis::Outcome],
            human_fallback: true,
            evidence_required: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkContract {
    pub id: WorkContractId,
    pub workspace_id: WorkspaceId,
    pub work_package_id: WorkPackageId,
    pub outcome_contract_id: Option<OutcomeContractId>,
    pub created_at: Timestamp,
}

impl WorkContract {
    pub fn new(workspace_id: WorkspaceId, work_package_id: WorkPackageId) -> Self {
        Self {
            id: WorkContractId::generate_with("wct"),
            workspace_id,
            work_package_id,
            outcome_contract_id: None,
            created_at: Timestamp::now(),
        }
    }
}
