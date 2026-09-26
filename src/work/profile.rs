//! profile — Domain Profiles as guarantee requirements.
//!
//! A Profile is a set of guarantee requirements, not a provider list. The same
//! profile must allow replacing providers (DSH or Pi, OPA or reference policy,
//! local store or future Temporal, SAP or fake connector). Released profile
//! versions are frozen; evolution creates a new version; a published version
//! is never silently rewritten.

use serde::{Deserialize, Serialize};

/// Requirement level within a profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RequirementLevel {
    Required,
    Forbidden,
    Optional,
}

/// One guarantee requirement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProfileRequirement {
    pub requirement: String,
    pub level: RequirementLevel,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
}

impl ProfileRequirement {
    pub fn required(requirement: &str) -> Self {
        Self {
            requirement: requirement.to_string(),
            level: RequirementLevel::Required,
            condition: None,
        }
    }

    pub fn forbidden(requirement: &str) -> Self {
        Self {
            requirement: requirement.to_string(),
            level: RequirementLevel::Forbidden,
            condition: None,
        }
    }

    pub fn required_if(requirement: &str, condition: &str) -> Self {
        Self {
            requirement: requirement.to_string(),
            level: RequirementLevel::Required,
            condition: Some(condition.to_string()),
        }
    }
}

/// Profile — the protocol resource.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub version: String,
    #[serde(default = "default_api_version")]
    pub api_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    pub requirements: Vec<ProfileRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub human_gates: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observability: Option<serde_json::Value>,
}

fn default_api_version() -> String {
    "profile.morn.io/v1alpha1".to_string()
}

impl Profile {
    /// Returns the requirement level declared for a given requirement name.
    pub fn level_of(&self, requirement: &str) -> RequirementLevel {
        self.requirements
            .iter()
            .find(|r| r.requirement == requirement)
            .map(|r| r.level)
            .unwrap_or(RequirementLevel::Optional)
    }

    /// `morn.lite`: minimal guarantees for local/personal use.
    pub fn lite() -> Self {
        Self {
            name: "morn.lite".to_string(),
            version: "v1alpha1".to_string(),
            api_version: default_api_version(),
            digest: None,
            requirements: vec![
                ProfileRequirement::required("durable_work_state"),
                ProfileRequirement::required("accepted_outcome_semantics"),
            ],
            human_gates: Vec::new(),
            execution: None,
            observability: None,
        }
    }

    /// `morn.enterprise`: full guarantee chain for enterprise sites.
    pub fn enterprise() -> Self {
        Self {
            name: "morn.enterprise".to_string(),
            version: "v1alpha1".to_string(),
            api_version: default_api_version(),
            digest: None,
            requirements: vec![
                ProfileRequirement::required("durable_work_state"),
                ProfileRequirement::required("source_of_truth_binding"),
                ProfileRequirement::required("provenance"),
                ProfileRequirement::required("authority_before_external_side_effect"),
                ProfileRequirement::required("receipt_after_external_action"),
                ProfileRequirement::required("reconciliation_on_unknown"),
                ProfileRequirement::required("accepted_outcome_semantics"),
                ProfileRequirement::required("profile_version_pinning"),
                ProfileRequirement::required("observability.trace"),
            ],
            human_gates: Vec::new(),
            execution: None,
            observability: Some(serde_json::json!({ "trace_required": true })),
        }
    }

    /// `morn.factory.readonly`: the Factory first-package guarantee profile.
    ///
    /// SAFETY: production_write is FORBIDDEN in the initial profile — no
    /// production write until site qualification and separate authorization
    /// exist.
    pub fn factory_readonly() -> Self {
        Self {
            name: "morn.factory.readonly".to_string(),
            version: "v1alpha1".to_string(),
            api_version: default_api_version(),
            digest: None,
            requirements: vec![
                ProfileRequirement::required("durable_work_state"),
                ProfileRequirement::required("source_of_truth_binding"),
                ProfileRequirement::required("provenance"),
                ProfileRequirement::required("authority_before_external_side_effect"),
                ProfileRequirement::required("receipt_after_external_action"),
                ProfileRequirement::required_if("reconciliation_on_unknown", "outcome-unknown"),
                ProfileRequirement::required("accepted_outcome_semantics"),
                ProfileRequirement::required("profile_version_pinning"),
                ProfileRequirement::forbidden("production_write"),
            ],
            human_gates: vec![
                "schedule_commitment".to_string(),
                "quality_decision".to_string(),
            ],
            execution: Some(serde_json::json!({ "minimum_isolation": "container" })),
            observability: Some(serde_json::json!({ "trace_required": true })),
        }
    }

    /// `morn.research`: research/lab profile with durable state + provenance.
    pub fn research() -> Self {
        Self {
            name: "morn.research".to_string(),
            version: "v1alpha1".to_string(),
            api_version: default_api_version(),
            digest: None,
            requirements: vec![
                ProfileRequirement::required("durable_work_state"),
                ProfileRequirement::required("provenance"),
                ProfileRequirement::required("accepted_outcome_semantics"),
            ],
            human_gates: Vec::new(),
            execution: None,
            observability: None,
        }
    }
}

/// SolutionPackage — a released bundle of blueprint + capability releases
/// under a profile.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SolutionPackage {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub work_blueprint_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capability_release_refs: Vec<String>,
    pub profile_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<serde_json::Value>,
}

impl SolutionPackage {
    pub fn new(id: &str, name: &str, version: &str, profile_ref: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            version: version.to_string(),
            work_blueprint_ref: None,
            capability_release_refs: Vec::new(),
            profile_ref: profile_ref.to_string(),
            digest: None,
            created_at: crate::work::now_iso(),
            provenance: None,
        }
    }
}
