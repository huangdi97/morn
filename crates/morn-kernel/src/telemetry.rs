//! OpenTelemetry-oriented semantic contract for Morn.
//!
//! Exporters/SDKs stay replaceable. This module defines stable span names,
//! required Morn semantic attributes and secret-safe validation so telemetry
//! cannot become a second source of Work truth or a credential leak.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

pub const ATTR_WORK_ID: &str = "morn.work.id";
pub const ATTR_WORK_TYPE: &str = "morn.work.type";
pub const ATTR_WORK_GENERATION: &str = "morn.work.generation";
pub const ATTR_PROFILE_ID: &str = "morn.profile.id";
pub const ATTR_BINDING_ID: &str = "morn.binding.id";
pub const ATTR_ATTEMPT_ID: &str = "morn.attempt.id";
pub const ATTR_CAPABILITY_ID: &str = "morn.capability.id";
pub const ATTR_CAPABILITY_DIGEST: &str = "morn.capability.digest";
pub const ATTR_PROVIDER_ID: &str = "morn.provider.id";
pub const ATTR_RUNTIME_ID: &str = "morn.runtime.id";
pub const ATTR_EXECUTION_ENVIRONMENT_ID: &str = "morn.execution_environment.id";
pub const ATTR_AUTHORITY_DECISION_ID: &str = "morn.authority.decision_id";
pub const ATTR_SITE_ID: &str = "morn.site.id";
pub const ATTR_OUTCOME_ID: &str = "morn.outcome.id";
pub const ATTR_ACCEPTANCE_ID: &str = "morn.acceptance.id";
pub const ATTR_RECONCILIATION_ID: &str = "morn.reconciliation.id";
pub const ATTR_EVIDENCE_CLASS: &str = "morn.evidence.class";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum MornTelemetryOperation {
    WorkReconcile,
    ProviderExecute,
    ExternalAction,
    Reconciliation,
    OutcomeObservation,
    AcceptanceDecision,
}

impl MornTelemetryOperation {
    pub const fn span_name(self) -> &'static str {
        match self {
            Self::WorkReconcile => "morn.work.reconcile",
            Self::ProviderExecute => "morn.provider.execute",
            Self::ExternalAction => "morn.external_action.execute",
            Self::Reconciliation => "morn.external_action.reconcile",
            Self::OutcomeObservation => "morn.outcome.observe",
            Self::AcceptanceDecision => "morn.acceptance.decide",
        }
    }

    pub const fn required_attributes(self) -> &'static [&'static str] {
        match self {
            Self::WorkReconcile => &[ATTR_WORK_ID, ATTR_WORK_GENERATION, ATTR_PROFILE_ID],
            Self::ProviderExecute => &[
                ATTR_WORK_ID,
                ATTR_BINDING_ID,
                ATTR_PROVIDER_ID,
                ATTR_EXECUTION_ENVIRONMENT_ID,
            ],
            Self::ExternalAction => &[ATTR_WORK_ID, ATTR_BINDING_ID, ATTR_ATTEMPT_ID],
            Self::Reconciliation => &[ATTR_WORK_ID, ATTR_ATTEMPT_ID, ATTR_RECONCILIATION_ID],
            Self::OutcomeObservation => &[ATTR_WORK_ID, ATTR_OUTCOME_ID],
            Self::AcceptanceDecision => &[ATTR_WORK_ID, ATTR_OUTCOME_ID, ATTR_ACCEPTANCE_ID],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct TelemetryContext {
    pub attributes: BTreeMap<String, String>,
}

impl TelemetryContext {
    /// Compatibility setter retained for existing callers. Semantic records
    /// still validate the complete context before export.
    pub fn set(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.attributes.insert(key.into(), value.into());
    }

    /// Strict setter for new code that wants immediate validation.
    pub fn try_set(&mut self, key: impl Into<String>, value: impl Into<String>) -> Result<()> {
        let key = key.into();
        let value = value.into();
        validate_telemetry_attribute(&key, &value)?;
        self.attributes.insert(key, value);
        Ok(())
    }

    /// Backward-compatible Work context without generation.
    pub fn with_work(mut self, work_id: impl Into<String>, profile_id: impl Into<String>) -> Self {
        self.set(ATTR_WORK_ID, work_id);
        self.set(ATTR_PROFILE_ID, profile_id);
        self
    }

    /// v11.5 desired/observed Work spans should pin the generation explicitly.
    pub fn with_work_generation(
        mut self,
        work_id: impl Into<String>,
        generation: u64,
        profile_id: impl Into<String>,
    ) -> Self {
        self.set(ATTR_WORK_ID, work_id);
        self.set(ATTR_WORK_GENERATION, generation.to_string());
        self.set(ATTR_PROFILE_ID, profile_id);
        self
    }

    pub fn validate_for(&self, operation: MornTelemetryOperation) -> Result<()> {
        for required in operation.required_attributes() {
            if self
                .attributes
                .get(*required)
                .is_none_or(|value| value.trim().is_empty())
            {
                return Err(Error::validation(format!(
                    "{} requires telemetry attribute {required}",
                    operation.span_name()
                )));
            }
        }
        for (key, value) in &self.attributes {
            validate_telemetry_attribute(key, value)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TelemetryRecord {
    pub operation: MornTelemetryOperation,
    pub span_name: String,
    pub context: TelemetryContext,
    /// Evidence references are identifiers/URIs only. Raw evidence remains in
    /// the authoritative evidence/source system.
    pub evidence_refs: Vec<String>,
}

impl TelemetryRecord {
    pub fn new(
        operation: MornTelemetryOperation,
        context: TelemetryContext,
        evidence_refs: Vec<String>,
    ) -> Result<Self> {
        context.validate_for(operation)?;
        if evidence_refs
            .iter()
            .any(|reference| reference.trim().is_empty())
        {
            return Err(Error::validation(
                "telemetry evidence references must be non-empty identifiers",
            ));
        }
        Ok(Self {
            operation,
            span_name: operation.span_name().to_string(),
            context,
            evidence_refs,
        })
    }
}

fn validate_telemetry_attribute(key: &str, value: &str) -> Result<()> {
    if key.trim().is_empty() || value.trim().is_empty() {
        return Err(Error::validation(
            "telemetry attributes require non-empty key/value",
        ));
    }
    if !key.starts_with("morn.") {
        return Err(Error::validation(
            "Morn semantic telemetry attributes must use the morn.* namespace",
        ));
    }
    let normalized = key.to_ascii_lowercase();
    if [
        "token",
        "password",
        "secret",
        "credential",
        "api_key",
        "apikey",
    ]
    .iter()
    .any(|sensitive| normalized.contains(sensitive))
    {
        return Err(Error::validation(
            "credentials/secrets must not be emitted as telemetry attributes",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatibility_work_context_remains_available() {
        let ctx = TelemetryContext::default().with_work("work-1", "morn.lite@1.0.0");
        assert_eq!(ctx.attributes[ATTR_WORK_ID], "work-1");
        assert_eq!(ctx.attributes[ATTR_PROFILE_ID], "morn.lite@1.0.0");
    }

    #[test]
    fn work_reconcile_has_stable_span_and_generation() {
        let context = TelemetryContext::default().with_work_generation(
            "work-1",
            3,
            "morn.factory.readonly@1.0.0",
        );
        let record = TelemetryRecord::new(
            MornTelemetryOperation::WorkReconcile,
            context,
            vec!["condition-evidence://capability-resolved".to_string()],
        )
        .unwrap();
        assert_eq!(record.span_name, "morn.work.reconcile");
        assert_eq!(record.context.attributes[ATTR_WORK_GENERATION], "3");
    }

    #[test]
    fn provider_execution_requires_binding_provider_and_environment_identity() {
        let mut context =
            TelemetryContext::default().with_work_generation("work-1", 1, "morn.lite@1.0.0");
        context.set(ATTR_BINDING_ID, "binding-1");
        context.set(ATTR_PROVIDER_ID, "deepseek-harness");
        assert!(context
            .validate_for(MornTelemetryOperation::ProviderExecute)
            .is_err());
        context.set(ATTR_EXECUTION_ENVIRONMENT_ID, "env://sandbox-a");
        context
            .validate_for(MornTelemetryOperation::ProviderExecute)
            .unwrap();
    }

    #[test]
    fn acceptance_span_requires_outcome_and_acceptance_identity() {
        let mut context =
            TelemetryContext::default().with_work_generation("work-1", 1, "morn.lite@1.0.0");
        assert!(TelemetryRecord::new(
            MornTelemetryOperation::AcceptanceDecision,
            context.clone(),
            vec![]
        )
        .is_err());
        context.set(ATTR_OUTCOME_ID, "outcome-1");
        context.set(ATTR_ACCEPTANCE_ID, "acceptance-1");
        TelemetryRecord::new(
            MornTelemetryOperation::AcceptanceDecision,
            context,
            vec!["review://ticket-1".to_string()],
        )
        .unwrap();
    }

    #[test]
    fn strict_semantic_attributes_reject_secrets_and_foreign_namespaces() {
        let mut context = TelemetryContext::default();
        assert!(context.try_set("authorization.token", "abc").is_err());
        assert!(context.try_set("morn.credential.secret", "abc").is_err());
        assert!(context.try_set("morn.work.id", "work-1").is_ok());
    }
}
