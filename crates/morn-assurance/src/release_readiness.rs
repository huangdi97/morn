//! Evidence-driven release/readiness projection.
//!
//! This module does not create evidence. It only projects the current
//! deployment-owned EvidenceLedger together with ephemeral live-runtime health.
//! Historical CI/fixture success can therefore never promote RealRuntime,
//! RealSite or ProductionWrite readiness.

use serde::{Deserialize, Serialize};

use crate::evidence_class::{EvidenceClaimState, EvidenceClass, EvidenceLedger};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReadinessState {
    Proven,
    BlockedExternal,
    Revoked,
    MissingEvidence,
    IdentityMismatch,
    RuntimeUnhealthy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeReadinessEvidence {
    pub fresh_healthy: bool,
    pub evidence_refs: Vec<String>,
    pub reason: String,
}

impl RuntimeReadinessEvidence {
    pub fn healthy(evidence_refs: Vec<String>, reason: impl Into<String>) -> Self {
        Self {
            fresh_healthy: true,
            evidence_refs,
            reason: reason.into(),
        }
    }

    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            fresh_healthy: false,
            evidence_refs: Vec::new(),
            reason: reason.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReleaseReadinessAxis {
    pub id: String,
    pub subject: String,
    pub evidence_class: EvidenceClass,
    pub state: ReadinessState,
    pub reason: String,
    pub evidence_refs: Vec<String>,
}

fn claim_state(claim: Option<&crate::evidence_class::EvidenceClaim>) -> ReadinessState {
    match claim.map(|claim| claim.state) {
        Some(EvidenceClaimState::Proven) => ReadinessState::Proven,
        Some(EvidenceClaimState::BlockedExternal) => ReadinessState::BlockedExternal,
        Some(EvidenceClaimState::Revoked) => ReadinessState::Revoked,
        None => ReadinessState::MissingEvidence,
    }
}

fn evidence_axis(
    ledger: &EvidenceLedger,
    id: &str,
    subject: &str,
    class: EvidenceClass,
    missing_reason: &str,
) -> ReleaseReadinessAxis {
    let claim = ledger.current_claim(subject, class);
    ReleaseReadinessAxis {
        id: id.to_string(),
        subject: subject.to_string(),
        evidence_class: class,
        state: claim_state(claim),
        reason: claim
            .map(|claim| claim.reason.clone())
            .unwrap_or_else(|| missing_reason.to_string()),
        evidence_refs: claim
            .map(|claim| claim.evidence_refs.clone())
            .unwrap_or_default(),
    }
}

fn ci_axis(ledger: &EvidenceLedger, build_identity_ref: Option<&str>) -> ReleaseReadinessAxis {
    let mut axis = evidence_axis(
        ledger,
        "ci-conformance",
        "morn-v11.5-ci-conformance",
        EvidenceClass::CiConformance,
        "this deployment has not loaded an explicit CI conformance proof",
    );
    if axis.state != ReadinessState::Proven {
        return axis;
    }

    let Some(build_identity_ref) = build_identity_ref
        .map(str::trim)
        .filter(|reference| !reference.is_empty())
    else {
        axis.state = ReadinessState::MissingEvidence;
        axis.reason =
            "CI proof exists, but the running deployment has no exact build identity configured"
                .to_string();
        return axis;
    };

    if !axis
        .evidence_refs
        .iter()
        .any(|reference| reference == build_identity_ref)
    {
        axis.state = ReadinessState::IdentityMismatch;
        axis.reason =
            format!("CI proof does not reference the running build identity {build_identity_ref}");
    }
    axis
}

fn runtime_axis(
    ledger: &EvidenceLedger,
    id: &str,
    subject: &str,
    runtime: RuntimeReadinessEvidence,
) -> ReleaseReadinessAxis {
    let mut axis = evidence_axis(
        ledger,
        id,
        subject,
        EvidenceClass::RealRuntime,
        "no deployment-owned real-runtime evidence claim exists",
    );

    // Real runtime readiness is conjunctive: a deployment proof without a
    // current health lease is stale, while a healthy process without explicit
    // RealRuntime evidence remains unproven.
    if axis.state == ReadinessState::Proven {
        if runtime.fresh_healthy {
            axis.evidence_refs.extend(runtime.evidence_refs);
            axis.evidence_refs.sort();
            axis.evidence_refs.dedup();
            if !runtime.reason.trim().is_empty() {
                axis.reason = format!("{}; {}", axis.reason, runtime.reason);
            }
        } else {
            axis.state = ReadinessState::RuntimeUnhealthy;
            axis.reason = format!(
                "deployment evidence exists but current runtime is not fresh Healthy: {}",
                runtime.reason
            );
            axis.evidence_refs.clear();
        }
    }
    axis
}

/// Current independent readiness axes. This intentionally has no aggregate
/// "production ready" boolean because a deployment chooses which providers and
/// effect classes are in scope. Callers must inspect the exact required axes.
pub fn release_readiness_axes(
    ledger: &EvidenceLedger,
    dsh_runtime: RuntimeReadinessEvidence,
    pi_runtime: RuntimeReadinessEvidence,
) -> Vec<ReleaseReadinessAxis> {
    release_readiness_axes_for_build(ledger, dsh_runtime, pi_runtime, None)
}

pub fn release_readiness_axes_for_build(
    ledger: &EvidenceLedger,
    dsh_runtime: RuntimeReadinessEvidence,
    pi_runtime: RuntimeReadinessEvidence,
    build_identity_ref: Option<&str>,
) -> Vec<ReleaseReadinessAxis> {
    vec![
        evidence_axis(
            ledger,
            "architecture-baseline",
            "morn-v11.5-architecture",
            EvidenceClass::DesignSpec,
            "v11.5 architecture baseline is not proven",
        ),
        evidence_axis(
            ledger,
            "local-reference-slice",
            "factory-readonly-wedge",
            EvidenceClass::LocalFixture,
            "local reference slice is not proven",
        ),
        ci_axis(ledger, build_identity_ref),
        runtime_axis(
            ledger,
            "deepseek-live-runtime",
            "deepseek-harness",
            dsh_runtime,
        ),
        runtime_axis(ledger, "pi-live-runtime", "pi-harness", pi_runtime),
        evidence_axis(
            ledger,
            "customer-real-site",
            "factory-customer",
            EvidenceClass::RealSite,
            "no authorized real-site evidence exists",
        ),
        evidence_axis(
            ledger,
            "production-write",
            "factory-production-write",
            EvidenceClass::ProductionWrite,
            "no explicit production-write evidence exists",
        ),
    ]
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeploymentReadinessScope {
    LocalReference,
    #[serde(rename = "deepseek-read-only")]
    DeepSeekReadOnly,
    PiReadOnly,
    #[serde(rename = "customer-read-only-deepseek")]
    CustomerReadOnlyDeepSeek,
    CustomerReadOnlyPi,
    #[serde(rename = "production-write-deepseek")]
    ProductionWriteDeepSeek,
    ProductionWritePi,
}

impl DeploymentReadinessScope {
    pub const fn id(self) -> &'static str {
        match self {
            Self::LocalReference => "local-reference",
            Self::DeepSeekReadOnly => "deepseek-read-only",
            Self::PiReadOnly => "pi-read-only",
            Self::CustomerReadOnlyDeepSeek => "customer-read-only-deepseek",
            Self::CustomerReadOnlyPi => "customer-read-only-pi",
            Self::ProductionWriteDeepSeek => "production-write-deepseek",
            Self::ProductionWritePi => "production-write-pi",
        }
    }

    fn required_axes(self) -> &'static [&'static str] {
        const LOCAL: &[&str] = &[
            "architecture-baseline",
            "local-reference-slice",
            "ci-conformance",
        ];
        const DSH: &[&str] = &[
            "architecture-baseline",
            "local-reference-slice",
            "ci-conformance",
            "deepseek-live-runtime",
        ];
        const PI: &[&str] = &[
            "architecture-baseline",
            "local-reference-slice",
            "ci-conformance",
            "pi-live-runtime",
        ];
        const CUSTOMER_DSH: &[&str] = &[
            "architecture-baseline",
            "local-reference-slice",
            "ci-conformance",
            "deepseek-live-runtime",
            "customer-real-site",
        ];
        const CUSTOMER_PI: &[&str] = &[
            "architecture-baseline",
            "local-reference-slice",
            "ci-conformance",
            "pi-live-runtime",
            "customer-real-site",
        ];
        const WRITE_DSH: &[&str] = &[
            "architecture-baseline",
            "local-reference-slice",
            "ci-conformance",
            "deepseek-live-runtime",
            "customer-real-site",
            "production-write",
        ];
        const WRITE_PI: &[&str] = &[
            "architecture-baseline",
            "local-reference-slice",
            "ci-conformance",
            "pi-live-runtime",
            "customer-real-site",
            "production-write",
        ];
        match self {
            Self::LocalReference => LOCAL,
            Self::DeepSeekReadOnly => DSH,
            Self::PiReadOnly => PI,
            Self::CustomerReadOnlyDeepSeek => CUSTOMER_DSH,
            Self::CustomerReadOnlyPi => CUSTOMER_PI,
            Self::ProductionWriteDeepSeek => WRITE_DSH,
            Self::ProductionWritePi => WRITE_PI,
        }
    }

    const fn requires_production_write_authority(self) -> bool {
        matches!(
            self,
            Self::ProductionWriteDeepSeek | Self::ProductionWritePi
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentReadinessDecision {
    pub scope: DeploymentReadinessScope,
    pub ready: bool,
    pub required_axes: Vec<String>,
    pub blockers: Vec<String>,
}

/// Evaluate one explicit deployment target. This consumes readiness evidence
/// only; it never creates Authority. Production-write scopes additionally
/// require the caller to supply the independently evaluated Profile/Authority
/// posture.
pub fn evaluate_deployment_readiness(
    axes: &[ReleaseReadinessAxis],
    scope: DeploymentReadinessScope,
    production_write_authority: bool,
) -> DeploymentReadinessDecision {
    let mut blockers = Vec::new();
    for required in scope.required_axes() {
        match axes.iter().find(|axis| axis.id == *required) {
            Some(axis) if axis.state == ReadinessState::Proven => {}
            Some(axis) => blockers.push(format!("{}={:?}: {}", required, axis.state, axis.reason)),
            None => blockers.push(format!("{required}=missing-axis")),
        }
    }
    if scope.requires_production_write_authority() && !production_write_authority {
        blockers.push(
            "production-write-authority=forbidden: evidence cannot grant Profile/Authority permission"
                .to_string(),
        );
    }

    DeploymentReadinessDecision {
        scope,
        ready: blockers.is_empty(),
        required_axes: scope
            .required_axes()
            .iter()
            .map(|axis| (*axis).to_string())
            .collect(),
        blockers,
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence_class::{reference_evidence_ledger, EvidenceClaim};

    #[test]
    fn deployment_scope_wire_names_keep_deepseek_as_one_provider_token() {
        assert_eq!(
            serde_json::to_string(&DeploymentReadinessScope::DeepSeekReadOnly).unwrap(),
            "\"deepseek-read-only\""
        );
        assert_eq!(
            serde_json::to_string(&DeploymentReadinessScope::CustomerReadOnlyDeepSeek).unwrap(),
            "\"customer-read-only-deepseek\""
        );
        assert_eq!(
            serde_json::to_string(&DeploymentReadinessScope::ProductionWriteDeepSeek).unwrap(),
            "\"production-write-deepseek\""
        );
    }

    #[test]
    fn deployment_scope_never_collapses_evidence_into_write_authority() {
        let mut axes = vec![
            ReleaseReadinessAxis {
                id: "architecture-baseline".to_string(),
                subject: "architecture".to_string(),
                evidence_class: EvidenceClass::DesignSpec,
                state: ReadinessState::Proven,
                reason: "ok".to_string(),
                evidence_refs: vec!["design://1".to_string()],
            },
            ReleaseReadinessAxis {
                id: "local-reference-slice".to_string(),
                subject: "fixture".to_string(),
                evidence_class: EvidenceClass::LocalFixture,
                state: ReadinessState::Proven,
                reason: "ok".to_string(),
                evidence_refs: vec!["fixture://1".to_string()],
            },
            ReleaseReadinessAxis {
                id: "ci-conformance".to_string(),
                subject: "ci".to_string(),
                evidence_class: EvidenceClass::CiConformance,
                state: ReadinessState::Proven,
                reason: "ok".to_string(),
                evidence_refs: vec!["ci://1".to_string()],
            },
            ReleaseReadinessAxis {
                id: "deepseek-live-runtime".to_string(),
                subject: "dsh".to_string(),
                evidence_class: EvidenceClass::RealRuntime,
                state: ReadinessState::Proven,
                reason: "ok".to_string(),
                evidence_refs: vec!["runtime://1".to_string()],
            },
            ReleaseReadinessAxis {
                id: "customer-real-site".to_string(),
                subject: "site".to_string(),
                evidence_class: EvidenceClass::RealSite,
                state: ReadinessState::Proven,
                reason: "ok".to_string(),
                evidence_refs: vec!["site://1".to_string()],
            },
            ReleaseReadinessAxis {
                id: "production-write".to_string(),
                subject: "write".to_string(),
                evidence_class: EvidenceClass::ProductionWrite,
                state: ReadinessState::Proven,
                reason: "historical production evidence".to_string(),
                evidence_refs: vec!["write://1".to_string()],
            },
        ];

        let customer = evaluate_deployment_readiness(
            &axes,
            DeploymentReadinessScope::CustomerReadOnlyDeepSeek,
            false,
        );
        assert!(customer.ready);

        let write_without_authority = evaluate_deployment_readiness(
            &axes,
            DeploymentReadinessScope::ProductionWriteDeepSeek,
            false,
        );
        assert!(!write_without_authority.ready);
        assert!(write_without_authority
            .blockers
            .iter()
            .any(|blocker| blocker.contains("production-write-authority=forbidden")));

        let write_with_authority = evaluate_deployment_readiness(
            &axes,
            DeploymentReadinessScope::ProductionWriteDeepSeek,
            true,
        );
        assert!(write_with_authority.ready);

        axes.iter_mut()
            .find(|axis| axis.id == "deepseek-live-runtime")
            .unwrap()
            .state = ReadinessState::RuntimeUnhealthy;
        let customer = evaluate_deployment_readiness(
            &axes,
            DeploymentReadinessScope::CustomerReadOnlyDeepSeek,
            false,
        );
        assert!(!customer.ready);
    }

    #[test]
    fn reference_evidence_never_promotes_external_axes() {
        let ledger = reference_evidence_ledger();
        let axes = release_readiness_axes(
            &ledger,
            RuntimeReadinessEvidence::healthy(
                vec!["runtime://dsh/healthy".to_string()],
                "healthy process",
            ),
            RuntimeReadinessEvidence::healthy(
                vec!["runtime://pi/healthy".to_string()],
                "healthy process",
            ),
        );
        assert_eq!(axes[0].state, ReadinessState::Proven);
        assert_eq!(axes[1].state, ReadinessState::Proven);
        assert_eq!(axes[2].state, ReadinessState::MissingEvidence);
        assert_eq!(axes[3].state, ReadinessState::BlockedExternal);
        assert_eq!(axes[4].state, ReadinessState::BlockedExternal);
        assert_eq!(axes[5].state, ReadinessState::BlockedExternal);
        assert_eq!(axes[6].state, ReadinessState::BlockedExternal);
    }

    #[test]
    fn ci_proof_must_name_the_exact_running_build_identity() {
        let mut ledger = reference_evidence_ledger();
        ledger
            .append(
                EvidenceClaim::proven(
                    "morn-v11.5-ci-conformance",
                    EvidenceClass::CiConformance,
                    vec![
                        "github-actions://run/42".to_string(),
                        "git://huangdi97/morn/abc123".to_string(),
                    ],
                    "release-controller",
                    "configured CI gates passed",
                )
                .unwrap(),
            )
            .unwrap();

        let no_identity = release_readiness_axes_for_build(
            &ledger,
            RuntimeReadinessEvidence::unavailable("not configured"),
            RuntimeReadinessEvidence::unavailable("not configured"),
            None,
        );
        assert_eq!(no_identity[2].state, ReadinessState::MissingEvidence);

        let wrong_identity = release_readiness_axes_for_build(
            &ledger,
            RuntimeReadinessEvidence::unavailable("not configured"),
            RuntimeReadinessEvidence::unavailable("not configured"),
            Some("git://huangdi97/morn/other"),
        );
        assert_eq!(wrong_identity[2].state, ReadinessState::IdentityMismatch);

        let exact_identity = release_readiness_axes_for_build(
            &ledger,
            RuntimeReadinessEvidence::unavailable("not configured"),
            RuntimeReadinessEvidence::unavailable("not configured"),
            Some("git://huangdi97/morn/abc123"),
        );
        assert_eq!(exact_identity[2].state, ReadinessState::Proven);
    }

    #[test]
    fn runtime_proof_alone_is_not_enough_without_fresh_health() {
        let mut ledger = reference_evidence_ledger();
        ledger
            .append(
                EvidenceClaim::proven(
                    "deepseek-harness",
                    EvidenceClass::RealRuntime,
                    vec!["attestation://dsh/live-1".to_string()],
                    "deployment-attestor",
                    "authenticated official runtime observed",
                )
                .unwrap(),
            )
            .unwrap();

        let axes = release_readiness_axes(
            &ledger,
            RuntimeReadinessEvidence::unavailable("health lease expired"),
            RuntimeReadinessEvidence::unavailable("not configured"),
        );
        assert_eq!(axes[3].state, ReadinessState::RuntimeUnhealthy);
        assert!(axes[3].evidence_refs.is_empty());
    }

    #[test]
    fn real_runtime_requires_both_current_claim_and_live_health() {
        let mut ledger = reference_evidence_ledger();
        ledger
            .append(
                EvidenceClaim::proven(
                    "deepseek-harness",
                    EvidenceClass::RealRuntime,
                    vec!["attestation://dsh/live-2".to_string()],
                    "deployment-attestor",
                    "authenticated official runtime observed",
                )
                .unwrap(),
            )
            .unwrap();

        let axes = release_readiness_axes(
            &ledger,
            RuntimeReadinessEvidence::healthy(
                vec!["health://dsh/lease-2".to_string()],
                "fresh settled-turn lease",
            ),
            RuntimeReadinessEvidence::unavailable("not configured"),
        );
        assert_eq!(axes[3].state, ReadinessState::Proven);
        assert!(axes[3]
            .evidence_refs
            .contains(&"attestation://dsh/live-2".to_string()));
        assert!(axes[3]
            .evidence_refs
            .contains(&"health://dsh/lease-2".to_string()));
    }

    #[test]
    fn revocation_beats_a_healthy_process() {
        let mut ledger = reference_evidence_ledger();
        ledger
            .append(
                EvidenceClaim::proven(
                    "pi-harness",
                    EvidenceClass::RealRuntime,
                    vec!["attestation://pi/live".to_string()],
                    "deployment-attestor",
                    "live runtime observed",
                )
                .unwrap(),
            )
            .unwrap();
        ledger
            .append(
                EvidenceClaim::revoked(
                    "pi-harness",
                    EvidenceClass::RealRuntime,
                    vec!["incident://pi/credential-revoked".to_string()],
                    "deployment-attestor",
                    "credential withdrawn",
                )
                .unwrap(),
            )
            .unwrap();

        let axes = release_readiness_axes(
            &ledger,
            RuntimeReadinessEvidence::unavailable("not configured"),
            RuntimeReadinessEvidence::healthy(
                vec!["health://pi/live".to_string()],
                "process is healthy",
            ),
        );
        assert_eq!(axes[4].state, ReadinessState::Revoked);
    }
}
