//! AcceptanceSpec: the definition of done for a WorkPackage.

use serde::{Deserialize, Serialize};

use morn_kernel::ids::AcceptanceSpecId;
use morn_kernel::time::Timestamp;

/// A WorkPackage can only be accepted when its AcceptanceSpec is satisfied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AcceptanceSpec {
    pub id: AcceptanceSpecId,
    pub name: String,
    pub required_artifacts: Vec<String>,
    pub schema_checks: Vec<String>,
    pub domain_rules: Vec<String>,
    pub reproducibility: bool,
    pub reviewer_requirements: Vec<String>,
    pub metric_thresholds: Vec<String>,
    pub forbidden_conditions: Vec<String>,
    pub human_approval_required: bool,
    pub created_at: Timestamp,
}

impl AcceptanceSpec {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: AcceptanceSpecId::generate_with("acc"),
            name: name.into(),
            required_artifacts: Vec::new(),
            schema_checks: Vec::new(),
            domain_rules: Vec::new(),
            reproducibility: true,
            reviewer_requirements: Vec::new(),
            metric_thresholds: Vec::new(),
            forbidden_conditions: Vec::new(),
            human_approval_required: false,
            created_at: Timestamp::now(),
        }
    }

    pub fn with_required_artifacts(mut self, artifacts: Vec<String>) -> Self {
        self.required_artifacts = artifacts;
        self
    }

    pub fn with_human_approval(mut self, required: bool) -> Self {
        self.human_approval_required = required;
        self
    }
}
