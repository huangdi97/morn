//! Evidence-driven release/readiness projection.
//!
//! This module does not create evidence. It only projects the current
//! deployment-owned EvidenceLedger together with ephemeral live-runtime health.
//! Historical CI/fixture success can therefore never promote RealRuntime,
//! RealSite or ProductionWrite readiness.

use serde::{Deserialize, Serialize};

use morn_kernel::time::Timestamp;

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
    NotYetValid,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeReadinessEvidence {
    pub fresh_healthy: bool,
    pub evidence_refs: Vec<String>,
    /// Exact deployment-attested provider artifact currently running.
    /// When present, the RealRuntime claim must cite this same identity.
    pub runtime_identity_ref: Option<String>,
    /// Additional active semantic identities for this provider route, such as
    /// DSH profile composition or provider/model route attestation. A live
    /// executable digest alone must not authorize a reinterpreted route.
    #[serde(default)]
    pub required_identity_refs: Vec<String>,
    pub reason: String,
}

impl RuntimeReadinessEvidence {
    pub fn healthy(evidence_refs: Vec<String>, reason: impl Into<String>) -> Self {
        Self {
            fresh_healthy: true,
            evidence_refs,
            runtime_identity_ref: None,
            required_identity_refs: Vec::new(),
            reason: reason.into(),
        }
    }

    pub fn healthy_for_identity(
        evidence_refs: Vec<String>,
        runtime_identity_ref: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            fresh_healthy: true,
            evidence_refs,
            runtime_identity_ref: Some(runtime_identity_ref.into()),
            required_identity_refs: Vec::new(),
            reason: reason.into(),
        }
    }

    pub fn healthy_for_identities(
        evidence_refs: Vec<String>,
        runtime_identity_ref: impl Into<String>,
        required_identity_refs: Vec<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            fresh_healthy: true,
            evidence_refs,
            runtime_identity_ref: Some(runtime_identity_ref.into()),
            required_identity_refs,
            reason: reason.into(),
        }
    }

    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            fresh_healthy: false,
            evidence_refs: Vec::new(),
            runtime_identity_ref: None,
            required_identity_refs: Vec::new(),
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

fn claim_state(
    claim: Option<&crate::evidence_class::EvidenceClaim>,
    now: Timestamp,
) -> ReadinessState {
    match claim {
        Some(claim) if claim.observed_at > now => ReadinessState::NotYetValid,
        Some(claim) => match claim.state {
            EvidenceClaimState::Proven => ReadinessState::Proven,
            EvidenceClaimState::BlockedExternal => ReadinessState::BlockedExternal,
            EvidenceClaimState::Revoked => ReadinessState::Revoked,
        },
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
        state: claim_state(claim, Timestamp::now()),
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

fn build_bound_evidence_axis(
    ledger: &EvidenceLedger,
    id: &str,
    subject: &str,
    class: EvidenceClass,
    missing_reason: &str,
    build_identity_ref: Option<&str>,
) -> ReleaseReadinessAxis {
    let mut axis = evidence_axis(ledger, id, subject, class, missing_reason);
    if axis.state != ReadinessState::Proven {
        return axis;
    }

    let Some(build_identity_ref) = build_identity_ref
        .map(str::trim)
        .filter(|reference| !reference.is_empty())
    else {
        axis.state = ReadinessState::MissingEvidence;
        axis.reason = format!(
            "{} proof exists, but the running deployment has no exact build identity configured",
            class.key()
        );
        return axis;
    };

    if !axis
        .evidence_refs
        .iter()
        .any(|reference| reference == build_identity_ref)
    {
        axis.state = ReadinessState::IdentityMismatch;
        axis.reason = format!(
            "{} proof does not reference the running build identity {build_identity_ref}",
            class.key()
        );
    }
    axis
}

fn runtime_axis(
    ledger: &EvidenceLedger,
    id: &str,
    subject: &str,
    runtime: RuntimeReadinessEvidence,
    build_identity_ref: Option<&str>,
    require_build_identity: bool,
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
    // RealRuntime evidence remains unproven. Deployment-facing projections
    // require an explicit running-build identity; absence is not a wildcard.
    // Legacy/reference callers may opt out only when they are not projecting a
    // concrete deployment.
    if axis.state == ReadinessState::Proven {
        let configured_build_identity = build_identity_ref
            .map(str::trim)
            .filter(|reference| !reference.is_empty());
        if require_build_identity && configured_build_identity.is_none() {
            axis.state = ReadinessState::MissingEvidence;
            axis.reason =
                "real-runtime proof exists, but the running deployment has no exact build identity configured"
                    .to_string();
            return axis;
        }
        if let Some(build_identity_ref) = configured_build_identity {
            if !axis
                .evidence_refs
                .iter()
                .any(|reference| reference == build_identity_ref)
            {
                axis.state = ReadinessState::IdentityMismatch;
                axis.reason = format!(
                    "real-runtime proof does not reference the running Morn build identity {build_identity_ref}"
                );
                return axis;
            }
        }
        if runtime.fresh_healthy {
            let Some(runtime_identity_ref) = runtime
                .runtime_identity_ref
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
            else {
                axis.state = ReadinessState::IdentityMismatch;
                axis.reason =
                    "fresh provider health has no exact active runtime identity".to_string();
                return axis;
            };
            let mut required_identities = runtime.required_identity_refs.clone();
            required_identities.push(runtime_identity_ref.to_string());
            required_identities.sort();
            required_identities.dedup();
            if let Some(missing) = required_identities.iter().find(|identity_ref| {
                !axis
                    .evidence_refs
                    .iter()
                    .any(|reference| reference == *identity_ref)
            }) {
                axis.state = ReadinessState::IdentityMismatch;
                axis.reason = format!(
                    "real-runtime proof does not reference the active deployment identity {missing}"
                );
                return axis;
            }
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
    release_readiness_axes_internal(ledger, dsh_runtime, pi_runtime, None, None, false, false)
}

pub fn release_readiness_axes_for_build(
    ledger: &EvidenceLedger,
    dsh_runtime: RuntimeReadinessEvidence,
    pi_runtime: RuntimeReadinessEvidence,
    build_identity_ref: Option<&str>,
) -> Vec<ReleaseReadinessAxis> {
    release_readiness_axes_internal(
        ledger,
        dsh_runtime,
        pi_runtime,
        build_identity_ref,
        None,
        true,
        false,
    )
}

/// Project readiness for one explicit deployment site. RealSite and
/// ProductionWrite evidence is site-scoped and build-bound so proof from
/// plant-a or from an older deployment build can never make a different target
/// ready.
pub fn release_readiness_axes_for_deployment(
    ledger: &EvidenceLedger,
    dsh_runtime: RuntimeReadinessEvidence,
    pi_runtime: RuntimeReadinessEvidence,
    build_identity_ref: Option<&str>,
    site_ref: Option<&str>,
) -> Vec<ReleaseReadinessAxis> {
    release_readiness_axes_internal(
        ledger,
        dsh_runtime,
        pi_runtime,
        build_identity_ref,
        site_ref,
        true,
        true,
    )
}

fn release_readiness_axes_internal(
    ledger: &EvidenceLedger,
    dsh_runtime: RuntimeReadinessEvidence,
    pi_runtime: RuntimeReadinessEvidence,
    build_identity_ref: Option<&str>,
    site_ref: Option<&str>,
    require_runtime_build_identity: bool,
    require_site_identity: bool,
) -> Vec<ReleaseReadinessAxis> {
    let site_ref = site_ref.map(str::trim).filter(|value| !value.is_empty());
    let customer_subject = site_ref
        .map(|site| format!("factory-customer:{site}"))
        .unwrap_or_else(|| "factory-customer".to_string());
    let write_subject = site_ref
        .map(|site| format!("factory-production-write:{site}"))
        .unwrap_or_else(|| "factory-production-write".to_string());
    let mut customer_axis = build_bound_evidence_axis(
        ledger,
        "customer-real-site",
        &customer_subject,
        EvidenceClass::RealSite,
        "no authorized real-site evidence exists",
        build_identity_ref,
    );
    let mut write_axis = build_bound_evidence_axis(
        ledger,
        "production-write",
        &write_subject,
        EvidenceClass::ProductionWrite,
        "no explicit production-write evidence exists",
        build_identity_ref,
    );
    if require_site_identity && site_ref.is_none() {
        for axis in [&mut customer_axis, &mut write_axis] {
            axis.state = ReadinessState::MissingEvidence;
            axis.reason =
                "deployment evidence requires an explicit exact deployment site identity"
                    .to_string();
            axis.evidence_refs.clear();
        }
    }

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
            build_identity_ref,
            require_runtime_build_identity,
        ),
        runtime_axis(
            ledger,
            "pi-live-runtime",
            "pi-harness",
            pi_runtime,
            build_identity_ref,
            require_runtime_build_identity,
        ),
        customer_axis,
        write_axis,
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

    const fn requires_site_target(self) -> bool {
        matches!(
            self,
            Self::CustomerReadOnlyDeepSeek
                | Self::CustomerReadOnlyPi
                | Self::ProductionWriteDeepSeek
                | Self::ProductionWritePi
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceAcquisitionStep {
    pub axis_id: String,
    pub external_required: bool,
    pub acquisition: String,
    pub workflow_ref: Option<String>,
    pub artifact_or_input: String,
    pub deployment_inputs: Vec<String>,
    pub safety_boundary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentReadinessDecision {
    pub scope: DeploymentReadinessScope,
    pub site_ref: Option<String>,
    pub ready: bool,
    pub required_axes: Vec<String>,
    pub blockers: Vec<String>,
    pub evidence_acquisition_plan: Vec<EvidenceAcquisitionStep>,
}

fn acquisition_step(
    axis: &ReleaseReadinessAxis,
    site_ref: Option<&str>,
) -> EvidenceAcquisitionStep {
    match axis.id.as_str() {
        "architecture-baseline" => EvidenceAcquisitionStep {
            axis_id: axis.id.clone(),
            external_required: false,
            acquisition: "Restore the versioned v11.5 architecture evidence bundled with this exact build.".to_string(),
            workflow_ref: Some(".github/workflows/ci.yml".to_string()),
            artifact_or_input: "DesignSpec claim for morn-v11.5-architecture".to_string(),
            deployment_inputs: vec![],
            safety_boundary: "Design evidence never proves a live provider, customer site or write authority.".to_string(),
        },
        "local-reference-slice" => EvidenceAcquisitionStep {
            axis_id: axis.id.clone(),
            external_required: false,
            acquisition: "Run the exact-head reference CI and factory read-only slice; load only its repository-local fixture proof.".to_string(),
            workflow_ref: Some(".github/workflows/ci.yml".to_string()),
            artifact_or_input: "LocalFixture claim for factory-readonly-wedge".to_string(),
            deployment_inputs: vec![],
            safety_boundary: "Fixture evidence cannot be promoted to RealRuntime, RealSite or ProductionWrite.".to_string(),
        },
        "ci-conformance" => EvidenceAcquisitionStep {
            axis_id: axis.id.clone(),
            external_required: false,
            acquisition: "Run exact-head CI and pin the running deployment to that exact build identity.".to_string(),
            workflow_ref: Some(".github/workflows/ci.yml".to_string()),
            artifact_or_input: "MORN_BUILD_IDENTITY_REF matching the CI evidence reference".to_string(),
            deployment_inputs: vec!["MORN_BUILD_IDENTITY_REF".to_string()],
            safety_boundary: "A green older SHA cannot satisfy a newer running build.".to_string(),
        },
        "deepseek-live-runtime" => EvidenceAcquisitionStep {
            axis_id: axis.id.clone(),
            external_required: true,
            acquisition: "Run authenticated E0 DSH smoke for the exact Morn build on the attested provider runner, then load the emitted RealRuntime claim and start the exact same pinned runtime route to obtain a fresh Healthy lease.".to_string(),
            workflow_ref: Some(".github/workflows/live-dsh-evidence.yml".to_string()),
            artifact_or_input: "real-runtime-evidence-claim.json citing the exact MORN_BUILD_IDENTITY_REF + fresh DSH Healthy lease".to_string(),
            deployment_inputs: vec![
                "MORN_BUILD_IDENTITY_REF".to_string(),
                "MORN_EVIDENCE_CLAIMS_FILE".to_string(),
                "MORN_EVIDENCE_ISSUER".to_string(),
                "MORN_DSH_MODE=real".to_string(),
                "MORN_EXECUTION_ATTESTATION_FILE".to_string(),
            ],
            safety_boundary: "Provider smoke is executor evidence only; it never creates Outcome, Acceptance or production-write authority.".to_string(),
        },
        "pi-live-runtime" => EvidenceAcquisitionStep {
            axis_id: axis.id.clone(),
            external_required: true,
            acquisition: "Run authenticated E0 Pi smoke for the exact Morn build on the attested provider runner, then load the emitted RealRuntime claim and start the exact same pinned runtime route to obtain a fresh Healthy lease.".to_string(),
            workflow_ref: Some(".github/workflows/live-pi-evidence.yml".to_string()),
            artifact_or_input: "real-runtime-evidence-claim.json citing the exact MORN_BUILD_IDENTITY_REF + fresh Pi Healthy lease".to_string(),
            deployment_inputs: vec![
                "MORN_BUILD_IDENTITY_REF".to_string(),
                "MORN_EVIDENCE_CLAIMS_FILE".to_string(),
                "MORN_EVIDENCE_ISSUER".to_string(),
                "MORN_PI_MODE=real".to_string(),
                "MORN_EXECUTION_ATTESTATION_FILE".to_string(),
            ],
            safety_boundary: "Provider smoke is executor evidence only; it never creates Outcome, Acceptance or production-write authority.".to_string(),
        },
        "customer-real-site" => {
            let subject = site_ref
                .map(|site| format!("factory-customer:{site}"))
                .unwrap_or_else(|| "factory-customer:<exact-site>".to_string());
            EvidenceAcquisitionStep {
                axis_id: axis.id.clone(),
                external_required: true,
                acquisition: "Obtain deployment/customer-owned real-site evidence for the exact site and load it through the trusted evidence-claim file.".to_string(),
                workflow_ref: Some(".github/workflows/live-customer-evidence.yml".to_string()),
                artifact_or_input: format!(
                    "RealSite EvidenceClaim subject={subject} citing the exact MORN_BUILD_IDENTITY_REF"
                ),
                deployment_inputs: vec![
                    "MORN_DEPLOYMENT_SITE_REF".to_string(),
                    "MORN_BUILD_IDENTITY_REF".to_string(),
                    "MORN_EVIDENCE_CLAIMS_FILE".to_string(),
                    "MORN_EVIDENCE_ISSUER".to_string(),
                ],
                safety_boundary: "Morn cannot mint customer/site truth; repository fixtures and provider output are insufficient.".to_string(),
            }
        }
        "production-write" => {
            let subject = site_ref
                .map(|site| format!("factory-production-write:{site}"))
                .unwrap_or_else(|| "factory-production-write:<exact-site>".to_string());
            EvidenceAcquisitionStep {
                axis_id: axis.id.clone(),
                external_required: true,
                acquisition: "Load deployment-owned ProductionWrite evidence for the exact site; each real action still needs an independent Work/Profile/Authority/Permit decision.".to_string(),
                workflow_ref: Some(".github/workflows/live-customer-evidence.yml".to_string()),
                artifact_or_input: format!(
                    "ProductionWrite EvidenceClaim subject={subject} citing the exact MORN_BUILD_IDENTITY_REF"
                ),
                deployment_inputs: vec![
                    "MORN_DEPLOYMENT_SITE_REF".to_string(),
                    "MORN_BUILD_IDENTITY_REF".to_string(),
                    "MORN_EVIDENCE_CLAIMS_FILE".to_string(),
                    "MORN_EVIDENCE_ISSUER".to_string(),
                ],
                safety_boundary: "Readiness evidence cannot grant action-level authority or bypass ExternalAction enforcement.".to_string(),
            }
        }
        other => EvidenceAcquisitionStep {
            axis_id: other.to_string(),
            external_required: true,
            acquisition: "Provide deployment-owned evidence for this readiness axis.".to_string(),
            workflow_ref: None,
            artifact_or_input: format!("EvidenceClaim for {}", axis.subject),
            deployment_inputs: vec!["MORN_EVIDENCE_CLAIMS_FILE".to_string()],
            safety_boundary: "Unknown evidence classes fail closed and must not be inferred from adjacent proof.".to_string(),
        },
    }
}

fn missing_axis_acquisition_step(axis_id: &str, site_ref: Option<&str>) -> EvidenceAcquisitionStep {
    let (subject, evidence_class) = match axis_id {
        "architecture-baseline" => (
            "morn-v11.5-architecture".to_string(),
            EvidenceClass::DesignSpec,
        ),
        "local-reference-slice" => (
            "factory-readonly-wedge".to_string(),
            EvidenceClass::LocalFixture,
        ),
        "ci-conformance" => (
            "morn-v11.5-ci-conformance".to_string(),
            EvidenceClass::CiConformance,
        ),
        "deepseek-live-runtime" => ("deepseek-harness".to_string(), EvidenceClass::RealRuntime),
        "pi-live-runtime" => ("pi-harness".to_string(), EvidenceClass::RealRuntime),
        "customer-real-site" => (
            site_ref
                .map(|site| format!("factory-customer:{site}"))
                .unwrap_or_else(|| "factory-customer:<exact-site>".to_string()),
            EvidenceClass::RealSite,
        ),
        "production-write" => (
            site_ref
                .map(|site| format!("factory-production-write:{site}"))
                .unwrap_or_else(|| "factory-production-write:<exact-site>".to_string()),
            EvidenceClass::ProductionWrite,
        ),
        other => (other.to_string(), EvidenceClass::DesignSpec),
    };
    acquisition_step(
        &ReleaseReadinessAxis {
            id: axis_id.to_string(),
            subject,
            evidence_class,
            state: ReadinessState::MissingEvidence,
            reason: "required readiness axis is absent from the projection".to_string(),
            evidence_refs: Vec::new(),
        },
        site_ref,
    )
}

fn axis_identity_matches_target(
    axis: &ReleaseReadinessAxis,
    scope: DeploymentReadinessScope,
    site_ref: Option<&str>,
) -> bool {
    let Some(site) = site_ref else {
        return true;
    };
    match axis.id.as_str() {
        "customer-real-site" if scope.requires_site_target() => {
            axis.subject == format!("factory-customer:{site}")
        }
        "production-write" if scope.requires_production_write_authority() => {
            axis.subject == format!("factory-production-write:{site}")
        }
        _ => true,
    }
}

fn deployment_target_acquisition_step() -> EvidenceAcquisitionStep {
    EvidenceAcquisitionStep {
        axis_id: "deployment-target-site".to_string(),
        external_required: true,
        acquisition: "Select and attest one exact deployment/customer site before evaluating customer or production readiness.".to_string(),
        workflow_ref: Some(".github/workflows/live-customer-evidence.yml".to_string()),
        artifact_or_input: "MORN_DEPLOYMENT_SITE_REF=<exact-site>".to_string(),
        deployment_inputs: vec!["MORN_DEPLOYMENT_SITE_REF".to_string()],
        safety_boundary: "A site must never be inferred from another evidence claim or replaced with a default tenant/site.".to_string(),
    }
}

fn production_write_authority_acquisition_step(site_ref: Option<&str>) -> EvidenceAcquisitionStep {
    EvidenceAcquisitionStep {
        axis_id: "production-write-authority".to_string(),
        external_required: true,
        acquisition: "Evaluate the exact Work generation, action, resource and site through Profile/Authority, then issue the corresponding one-action ExternalActionPermit.".to_string(),
        workflow_ref: None,
        artifact_or_input: site_ref
            .map(|site| format!(
                "AuthorityDecision + ExternalActionPermit for the exact Work/action at site {site}"
            ))
            .unwrap_or_else(|| {
                "AuthorityDecision + ExternalActionPermit for the exact Work/action/site".to_string()
            }),
        deployment_inputs: vec!["MORN_DEPLOYMENT_SITE_REF".to_string()],
        safety_boundary: "ProductionWrite readiness evidence can never grant action-level authority; no permit means no write.".to_string(),
    }
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
    evaluate_deployment_readiness_for_target(axes, scope, production_write_authority, None)
}

pub fn evaluate_deployment_readiness_for_target(
    axes: &[ReleaseReadinessAxis],
    scope: DeploymentReadinessScope,
    production_write_authority: bool,
    site_ref: Option<&str>,
) -> DeploymentReadinessDecision {
    let site_ref = site_ref.map(str::trim).filter(|value| !value.is_empty());
    let mut blockers = Vec::new();
    if scope.requires_site_target() && site_ref.is_none() {
        blockers.push(
            "deployment-target-site=missing: customer/production readiness must name an exact site"
                .to_string(),
        );
    }
    for required in scope.required_axes() {
        match axes.iter().find(|axis| axis.id == *required) {
            Some(axis) if axis.state == ReadinessState::Proven => {}
            Some(axis) => blockers.push(format!("{}={:?}: {}", required, axis.state, axis.reason)),
            None => blockers.push(format!("{required}=missing-axis")),
        }
    }
    if let Some(site) = site_ref {
        if scope.requires_site_target() {
            let expected_customer = format!("factory-customer:{site}");
            match axes.iter().find(|axis| axis.id == "customer-real-site") {
                Some(axis) if axis.subject == expected_customer => {}
                Some(axis) => blockers.push(format!(
                    "customer-real-site=identity-mismatch: expected subject {expected_customer}, got {}",
                    axis.subject
                )),
                None => {}
            }
        }
        if scope.requires_production_write_authority() {
            let expected_write = format!("factory-production-write:{site}");
            match axes.iter().find(|axis| axis.id == "production-write") {
                Some(axis) if axis.subject == expected_write => {}
                Some(axis) => blockers.push(format!(
                    "production-write=identity-mismatch: expected subject {expected_write}, got {}",
                    axis.subject
                )),
                None => {}
            }
        }
    }
    if scope.requires_production_write_authority() && !production_write_authority {
        blockers.push(
            "production-write-authority=forbidden: evidence cannot grant Profile/Authority permission"
                .to_string(),
        );
    }

    let mut evidence_acquisition_plan = Vec::new();
    for required in scope.required_axes() {
        match axes.iter().find(|axis| axis.id == *required) {
            Some(axis)
                if axis.state == ReadinessState::Proven
                    && axis_identity_matches_target(axis, scope, site_ref) => {}
            Some(axis) => evidence_acquisition_plan.push(acquisition_step(axis, site_ref)),
            None => {
                evidence_acquisition_plan.push(missing_axis_acquisition_step(required, site_ref))
            }
        }
    }
    if scope.requires_site_target() && site_ref.is_none() {
        evidence_acquisition_plan.push(deployment_target_acquisition_step());
    }
    if scope.requires_production_write_authority() && !production_write_authority {
        evidence_acquisition_plan.push(production_write_authority_acquisition_step(site_ref));
    }

    DeploymentReadinessDecision {
        scope,
        site_ref: site_ref.map(str::to_string),
        ready: blockers.is_empty(),
        required_axes: scope
            .required_axes()
            .iter()
            .map(|axis| (*axis).to_string())
            .collect(),
        blockers,
        evidence_acquisition_plan,
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
                subject: "factory-customer:plant-a".to_string(),
                evidence_class: EvidenceClass::RealSite,
                state: ReadinessState::Proven,
                reason: "ok".to_string(),
                evidence_refs: vec!["site://1".to_string()],
            },
            ReleaseReadinessAxis {
                id: "production-write".to_string(),
                subject: "factory-production-write:plant-a".to_string(),
                evidence_class: EvidenceClass::ProductionWrite,
                state: ReadinessState::Proven,
                reason: "historical production evidence".to_string(),
                evidence_refs: vec!["write://1".to_string()],
            },
        ];

        let customer = evaluate_deployment_readiness_for_target(
            &axes,
            DeploymentReadinessScope::CustomerReadOnlyDeepSeek,
            false,
            Some("plant-a"),
        );
        assert!(customer.ready);

        let write_without_authority = evaluate_deployment_readiness_for_target(
            &axes,
            DeploymentReadinessScope::ProductionWriteDeepSeek,
            false,
            Some("plant-a"),
        );
        assert!(!write_without_authority.ready);
        assert!(write_without_authority
            .blockers
            .iter()
            .any(|blocker| blocker.contains("production-write-authority=forbidden")));

        let write_with_authority = evaluate_deployment_readiness_for_target(
            &axes,
            DeploymentReadinessScope::ProductionWriteDeepSeek,
            true,
            Some("plant-a"),
        );
        assert!(write_with_authority.ready);

        axes.iter_mut()
            .find(|axis| axis.id == "deepseek-live-runtime")
            .unwrap()
            .state = ReadinessState::RuntimeUnhealthy;
        let customer = evaluate_deployment_readiness_for_target(
            &axes,
            DeploymentReadinessScope::CustomerReadOnlyDeepSeek,
            false,
            Some("plant-a"),
        );
        assert!(!customer.ready);
    }

    #[test]
    fn future_dated_proof_cannot_make_a_release_axis_ready() {
        let mut ledger = reference_evidence_ledger();
        let mut proof = EvidenceClaim::proven(
            "morn-v11.5-ci-conformance",
            EvidenceClass::CiConformance,
            vec!["git://huangdi97/morn/future".to_string()],
            "deployment-attestor",
            "clock-skewed CI claim",
        )
        .unwrap();
        proof.observed_at = Timestamp::from_millis(Timestamp::now().millis() + 60_000);
        ledger.append(proof).unwrap();

        let axes = release_readiness_axes_for_build(
            &ledger,
            RuntimeReadinessEvidence::unavailable("not configured"),
            RuntimeReadinessEvidence::unavailable("not configured"),
            Some("git://huangdi97/morn/future"),
        );
        let ci = axes
            .iter()
            .find(|axis| axis.id == "ci-conformance")
            .unwrap();
        assert_eq!(ci.state, ReadinessState::NotYetValid);
    }

    #[test]
    fn customer_and_write_readiness_are_bound_to_exact_site_identity() {
        let mut ledger = reference_evidence_ledger();
        ledger
            .append(
                EvidenceClaim::proven(
                    "factory-customer:plant-a",
                    EvidenceClass::RealSite,
                    vec!["site://plant-a/review".to_string()],
                    "deployment-attestor",
                    "plant-a read-only evidence",
                )
                .unwrap(),
            )
            .unwrap();
        ledger
            .append(
                EvidenceClaim::proven(
                    "factory-production-write:plant-a",
                    EvidenceClass::ProductionWrite,
                    vec!["write://plant-a/change-1".to_string()],
                    "deployment-attestor",
                    "plant-a production-write evidence",
                )
                .unwrap(),
            )
            .unwrap();

        let axes = release_readiness_axes_for_deployment(
            &ledger,
            RuntimeReadinessEvidence::unavailable("not configured"),
            RuntimeReadinessEvidence::unavailable("not configured"),
            None,
            Some("plant-a"),
        );
        assert_eq!(
            axes.iter()
                .find(|axis| axis.id == "customer-real-site")
                .unwrap()
                .subject,
            "factory-customer:plant-a"
        );
        assert_eq!(
            axes.iter()
                .find(|axis| axis.id == "production-write")
                .unwrap()
                .subject,
            "factory-production-write:plant-a"
        );

        let no_target = evaluate_deployment_readiness(
            &axes,
            DeploymentReadinessScope::CustomerReadOnlyDeepSeek,
            false,
        );
        assert!(!no_target.ready);
        assert!(no_target
            .blockers
            .iter()
            .any(|blocker| blocker.contains("deployment-target-site=missing")));

        let wrong_target = evaluate_deployment_readiness_for_target(
            &axes,
            DeploymentReadinessScope::CustomerReadOnlyDeepSeek,
            false,
            Some("plant-b"),
        );
        assert!(!wrong_target.ready);
        assert!(wrong_target
            .blockers
            .iter()
            .any(|blocker| blocker.contains("identity-mismatch")));
    }

    #[test]
    fn every_non_ready_scope_exposes_remediation_for_identity_missing_axis_and_authority() {
        let mut axes = vec![
            ReleaseReadinessAxis {
                id: "architecture-baseline".to_string(),
                subject: "morn-v11.5-architecture".to_string(),
                evidence_class: EvidenceClass::DesignSpec,
                state: ReadinessState::Proven,
                reason: "ok".to_string(),
                evidence_refs: vec!["design://1".to_string()],
            },
            ReleaseReadinessAxis {
                id: "local-reference-slice".to_string(),
                subject: "factory-readonly-wedge".to_string(),
                evidence_class: EvidenceClass::LocalFixture,
                state: ReadinessState::Proven,
                reason: "ok".to_string(),
                evidence_refs: vec!["fixture://1".to_string()],
            },
            ReleaseReadinessAxis {
                id: "ci-conformance".to_string(),
                subject: "morn-v11.5-ci-conformance".to_string(),
                evidence_class: EvidenceClass::CiConformance,
                state: ReadinessState::Proven,
                reason: "ok".to_string(),
                evidence_refs: vec!["ci://1".to_string()],
            },
            ReleaseReadinessAxis {
                id: "deepseek-live-runtime".to_string(),
                subject: "deepseek-harness".to_string(),
                evidence_class: EvidenceClass::RealRuntime,
                state: ReadinessState::Proven,
                reason: "ok".to_string(),
                evidence_refs: vec!["runtime://1".to_string()],
            },
            ReleaseReadinessAxis {
                id: "customer-real-site".to_string(),
                subject: "factory-customer:plant-a".to_string(),
                evidence_class: EvidenceClass::RealSite,
                state: ReadinessState::Proven,
                reason: "ok".to_string(),
                evidence_refs: vec!["site://plant-a".to_string()],
            },
            ReleaseReadinessAxis {
                id: "production-write".to_string(),
                subject: "factory-production-write:plant-a".to_string(),
                evidence_class: EvidenceClass::ProductionWrite,
                state: ReadinessState::Proven,
                reason: "ok".to_string(),
                evidence_refs: vec!["write://plant-a".to_string()],
            },
        ];

        let wrong_site = evaluate_deployment_readiness_for_target(
            &axes,
            DeploymentReadinessScope::CustomerReadOnlyDeepSeek,
            false,
            Some("plant-b"),
        );
        assert!(!wrong_site.ready);
        assert!(wrong_site
            .evidence_acquisition_plan
            .iter()
            .any(|step| step.axis_id == "customer-real-site"
                && step.artifact_or_input.contains("factory-customer:plant-b")));

        let missing_target = evaluate_deployment_readiness(
            &axes,
            DeploymentReadinessScope::CustomerReadOnlyDeepSeek,
            false,
        );
        assert!(missing_target
            .evidence_acquisition_plan
            .iter()
            .any(|step| step.axis_id == "deployment-target-site"));

        let no_authority = evaluate_deployment_readiness_for_target(
            &axes,
            DeploymentReadinessScope::ProductionWriteDeepSeek,
            false,
            Some("plant-a"),
        );
        assert!(no_authority
            .evidence_acquisition_plan
            .iter()
            .any(|step| step.axis_id == "production-write-authority"
                && step.safety_boundary.contains("no permit means no write")));

        axes.retain(|axis| axis.id != "deepseek-live-runtime");
        let missing_runtime_axis = evaluate_deployment_readiness_for_target(
            &axes,
            DeploymentReadinessScope::CustomerReadOnlyDeepSeek,
            false,
            Some("plant-a"),
        );
        assert!(missing_runtime_axis
            .blockers
            .iter()
            .any(|blocker| blocker == "deepseek-live-runtime=missing-axis"));
        let runtime_plan = missing_runtime_axis
            .evidence_acquisition_plan
            .iter()
            .find(|step| step.axis_id == "deepseek-live-runtime")
            .expect("missing required axis must still expose its acquisition path");
        assert_eq!(
            runtime_plan.workflow_ref.as_deref(),
            Some(".github/workflows/live-dsh-evidence.yml")
        );
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
    fn blocked_customer_runtime_scope_exposes_exact_evidence_acquisition_plan() {
        let axes = release_readiness_axes_for_deployment(
            &reference_evidence_ledger(),
            RuntimeReadinessEvidence::unavailable("live DSH not configured"),
            RuntimeReadinessEvidence::unavailable("live Pi not configured"),
            Some("build://morn/test"),
            Some("plant-a"),
        );
        let decision = evaluate_deployment_readiness_for_target(
            &axes,
            DeploymentReadinessScope::CustomerReadOnlyDeepSeek,
            false,
            Some("plant-a"),
        );
        assert!(!decision.ready);
        let dsh = decision
            .evidence_acquisition_plan
            .iter()
            .find(|step| step.axis_id == "deepseek-live-runtime")
            .unwrap();
        assert!(dsh.external_required);
        assert_eq!(
            dsh.workflow_ref.as_deref(),
            Some(".github/workflows/live-dsh-evidence.yml")
        );
        assert!(dsh
            .deployment_inputs
            .contains(&"MORN_EVIDENCE_CLAIMS_FILE".to_string()));

        let site = decision
            .evidence_acquisition_plan
            .iter()
            .find(|step| step.axis_id == "customer-real-site")
            .unwrap();
        assert!(site.artifact_or_input.contains("factory-customer:plant-a"));
        assert!(site
            .safety_boundary
            .contains("cannot mint customer/site truth"));
    }

    #[test]
    fn site_and_write_proofs_cannot_be_reused_across_deployment_builds() {
        let old_build = "build://morn/old";
        let current_build = "build://morn/current";
        let mut ledger = reference_evidence_ledger();

        let mut site = EvidenceClaim::proven(
            "factory-customer:plant-a",
            EvidenceClass::RealSite,
            vec!["site://plant-a/pilot-1".to_string(), old_build.to_string()],
            "deployment-attestor",
            "plant-a pilot on the old build",
        )
        .unwrap();
        site.observed_at = Timestamp::from_millis(10);
        ledger.append(site).unwrap();

        let mut write = EvidenceClaim::proven(
            "factory-production-write:plant-a",
            EvidenceClass::ProductionWrite,
            vec![
                "write://plant-a/change-1".to_string(),
                old_build.to_string(),
            ],
            "deployment-attestor",
            "plant-a write evidence on the old build",
        )
        .unwrap();
        write.observed_at = Timestamp::from_millis(10);
        ledger.append(write).unwrap();

        let stale = release_readiness_axes_for_deployment(
            &ledger,
            RuntimeReadinessEvidence::unavailable("not configured"),
            RuntimeReadinessEvidence::unavailable("not configured"),
            Some(current_build),
            Some("plant-a"),
        );
        for id in ["customer-real-site", "production-write"] {
            let axis = stale.iter().find(|axis| axis.id == id).unwrap();
            assert_eq!(axis.state, ReadinessState::IdentityMismatch);
            assert!(axis.reason.contains(current_build));
        }

        let mut fresh_site = EvidenceClaim::proven(
            "factory-customer:plant-a",
            EvidenceClass::RealSite,
            vec![
                "site://plant-a/pilot-2".to_string(),
                current_build.to_string(),
            ],
            "deployment-attestor",
            "plant-a pilot repeated on the current build",
        )
        .unwrap();
        fresh_site.observed_at = Timestamp::from_millis(20);
        ledger.append(fresh_site).unwrap();

        let mut fresh_write = EvidenceClaim::proven(
            "factory-production-write:plant-a",
            EvidenceClass::ProductionWrite,
            vec![
                "write://plant-a/change-2".to_string(),
                current_build.to_string(),
            ],
            "deployment-attestor",
            "plant-a write evidence repeated on the current build",
        )
        .unwrap();
        fresh_write.observed_at = Timestamp::from_millis(20);
        ledger.append(fresh_write).unwrap();

        let fresh = release_readiness_axes_for_deployment(
            &ledger,
            RuntimeReadinessEvidence::unavailable("not configured"),
            RuntimeReadinessEvidence::unavailable("not configured"),
            Some(current_build),
            Some("plant-a"),
        );
        for id in ["customer-real-site", "production-write"] {
            assert_eq!(
                fresh.iter().find(|axis| axis.id == id).unwrap().state,
                ReadinessState::Proven
            );
        }
    }

    #[test]
    fn deployment_customer_axes_require_explicit_site_identity() {
        let build = "build://morn/current";
        let mut ledger = reference_evidence_ledger();
        for (subject, class, evidence) in [
            (
                "factory-customer",
                EvidenceClass::RealSite,
                "site://generic/pilot",
            ),
            (
                "factory-production-write",
                EvidenceClass::ProductionWrite,
                "write://generic/change",
            ),
        ] {
            let mut claim = EvidenceClaim::proven(
                subject,
                class,
                vec![evidence.to_string(), build.to_string()],
                "deployment-attestor",
                "generic deployment evidence",
            )
            .unwrap();
            claim.observed_at = Timestamp::from_millis(10);
            ledger.append(claim).unwrap();
        }

        let axes = release_readiness_axes_for_deployment(
            &ledger,
            RuntimeReadinessEvidence::unavailable("not configured"),
            RuntimeReadinessEvidence::unavailable("not configured"),
            Some(build),
            None,
        );
        for id in ["customer-real-site", "production-write"] {
            let axis = axes.iter().find(|axis| axis.id == id).unwrap();
            assert_eq!(axis.state, ReadinessState::MissingEvidence);
            assert!(axis.reason.contains("exact deployment site identity"));
            assert!(axis.evidence_refs.is_empty());
        }
    }

    #[test]
    fn deployment_runtime_axis_requires_explicit_running_build_identity() {
        let runtime = "runtime://deepseek-harness/v1#sha256:artifact";
        let mut ledger = reference_evidence_ledger();
        let mut proof = EvidenceClaim::proven(
            "deepseek-harness",
            EvidenceClass::RealRuntime,
            vec![runtime.to_string(), "build://morn/smoke".to_string()],
            "deployment-attestor",
            "authenticated provider smoke passed",
        )
        .unwrap();
        proof.observed_at = Timestamp::from_millis(10);
        ledger.append(proof).unwrap();

        let axes = release_readiness_axes_for_deployment(
            &ledger,
            RuntimeReadinessEvidence::healthy_for_identity(
                vec!["health://dsh/settled-turn".to_string()],
                runtime,
                "fresh provider health",
            ),
            RuntimeReadinessEvidence::unavailable("not configured"),
            None,
            None,
        );
        let dsh = axes
            .iter()
            .find(|axis| axis.id == "deepseek-live-runtime")
            .unwrap();
        assert_eq!(dsh.state, ReadinessState::MissingEvidence);
        assert!(dsh.reason.contains("no exact build identity configured"));
    }

    #[test]
    fn runtime_proof_cannot_be_reused_across_morn_builds() {
        let runtime = "runtime://deepseek-harness/v1#sha256:artifact";
        let old_build = "build://morn/old";
        let current_build = "build://morn/current";
        let mut ledger = reference_evidence_ledger();
        let mut old = EvidenceClaim::proven(
            "deepseek-harness",
            EvidenceClass::RealRuntime,
            vec![runtime.to_string(), old_build.to_string()],
            "deployment-attestor",
            "authenticated provider smoke passed on the old Morn build",
        )
        .unwrap();
        old.observed_at = Timestamp::from_millis(10);
        ledger.append(old).unwrap();

        let stale = release_readiness_axes_for_build(
            &ledger,
            RuntimeReadinessEvidence::healthy_for_identity(
                vec!["health://dsh/settled-turn".to_string()],
                runtime,
                "fresh provider health on the current deployment",
            ),
            RuntimeReadinessEvidence::unavailable("not configured"),
            Some(current_build),
        );
        let dsh = stale
            .iter()
            .find(|axis| axis.id == "deepseek-live-runtime")
            .unwrap();
        assert_eq!(dsh.state, ReadinessState::IdentityMismatch);
        assert!(dsh.reason.contains(current_build));

        let mut current = EvidenceClaim::proven(
            "deepseek-harness",
            EvidenceClass::RealRuntime,
            vec![runtime.to_string(), current_build.to_string()],
            "deployment-attestor",
            "authenticated provider smoke repeated on the current Morn build",
        )
        .unwrap();
        current.observed_at = Timestamp::from_millis(20);
        ledger.append(current).unwrap();

        let fresh = release_readiness_axes_for_build(
            &ledger,
            RuntimeReadinessEvidence::healthy_for_identity(
                vec!["health://dsh/settled-turn".to_string()],
                runtime,
                "fresh provider health on the current deployment",
            ),
            RuntimeReadinessEvidence::unavailable("not configured"),
            Some(current_build),
        );
        assert_eq!(
            fresh
                .iter()
                .find(|axis| axis.id == "deepseek-live-runtime")
                .unwrap()
                .state,
            ReadinessState::Proven
        );
    }

    #[test]
    fn runtime_proof_must_bind_the_exact_active_provider_artifact() {
        let mut ledger = reference_evidence_ledger();
        ledger
            .append(
                EvidenceClaim::proven(
                    "deepseek-harness",
                    EvidenceClass::RealRuntime,
                    vec![
                        "runtime://deepseek-harness/v1#sha256:old".to_string(),
                        "attestation://dsh/live-old".to_string(),
                    ],
                    "deployment-attestor",
                    "old authenticated runtime smoke",
                )
                .unwrap(),
            )
            .unwrap();

        let axes = release_readiness_axes(
            &ledger,
            RuntimeReadinessEvidence::healthy_for_identity(
                vec!["runtime://deepseek-harness/session/new/settled-turn".to_string()],
                "runtime://deepseek-harness/v2#sha256:new",
                "fresh health on replacement runtime",
            ),
            RuntimeReadinessEvidence::unavailable("not configured"),
        );
        let dsh = axes
            .iter()
            .find(|axis| axis.id == "deepseek-live-runtime")
            .unwrap();
        assert_eq!(dsh.state, ReadinessState::IdentityMismatch);
        assert!(dsh.reason.contains("active deployment identity"));
    }

    #[test]
    fn runtime_proof_must_bind_every_active_semantic_route_identity() {
        let runtime = "runtime://deepseek-harness/v1#sha256:artifact";
        let profile = format!("deepseek-harness-profile@v1#sha256:{}", "a".repeat(64));
        let route = format!("deepseek-harness-route@v1#sha256:{}", "b".repeat(64));
        let mut ledger = reference_evidence_ledger();
        ledger
            .append(
                EvidenceClaim::proven(
                    "deepseek-harness",
                    EvidenceClass::RealRuntime,
                    vec![runtime.to_string(), profile.clone()],
                    "deployment-attestor",
                    "runtime and profile proven, but route is stale",
                )
                .unwrap(),
            )
            .unwrap();

        let axes = release_readiness_axes(
            &ledger,
            RuntimeReadinessEvidence::healthy_for_identities(
                vec!["health://dsh/settled-turn".to_string()],
                runtime,
                vec![profile, route.clone()],
                "fresh health on current configured route",
            ),
            RuntimeReadinessEvidence::unavailable("not configured"),
        );
        let dsh = axes
            .iter()
            .find(|axis| axis.id == "deepseek-live-runtime")
            .unwrap();
        assert_eq!(dsh.state, ReadinessState::IdentityMismatch);
        assert!(dsh.reason.contains(&route));
    }

    #[test]
    fn runtime_proof_cannot_mix_health_with_another_execution_environment() {
        let runtime = "runtime://deepseek-harness/v1#sha256:artifact";
        let profile = format!("deepseek-harness-profile@v1#sha256:{}", "a".repeat(64));
        let route = format!("deepseek-harness-route@v1#sha256:{}", "b".repeat(64));
        let environment = "env://container/dsh-runtime-a";
        let mut ledger = reference_evidence_ledger();
        ledger
            .append(
                EvidenceClaim::proven(
                    "deepseek-harness",
                    EvidenceClass::RealRuntime,
                    vec![runtime.to_string(), profile.clone(), route.clone()],
                    "deployment-attestor",
                    "runtime route proven in an unspecified environment",
                )
                .unwrap(),
            )
            .unwrap();

        let axes = release_readiness_axes(
            &ledger,
            RuntimeReadinessEvidence::healthy_for_identities(
                vec!["health://dsh/settled-turn".to_string()],
                runtime,
                vec![environment.to_string(), profile, route],
                "fresh health from the configured execution environment",
            ),
            RuntimeReadinessEvidence::unavailable("not configured"),
        );
        let dsh = axes
            .iter()
            .find(|axis| axis.id == "deepseek-live-runtime")
            .unwrap();
        assert_eq!(dsh.state, ReadinessState::IdentityMismatch);
        assert!(dsh.reason.contains(environment));
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
        let runtime_identity = "runtime://deepseek-harness/v1#sha256:current";
        let mut ledger = reference_evidence_ledger();
        ledger
            .append(
                EvidenceClaim::proven(
                    "deepseek-harness",
                    EvidenceClass::RealRuntime,
                    vec![
                        "attestation://dsh/live-2".to_string(),
                        runtime_identity.to_string(),
                    ],
                    "deployment-attestor",
                    "authenticated official runtime observed on the exact artifact",
                )
                .unwrap(),
            )
            .unwrap();

        let axes = release_readiness_axes(
            &ledger,
            RuntimeReadinessEvidence::healthy_for_identity(
                vec!["health://dsh/lease-2".to_string()],
                runtime_identity,
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
    fn healthy_lease_without_exact_runtime_identity_cannot_prove_real_runtime() {
        let mut ledger = reference_evidence_ledger();
        ledger
            .append(
                EvidenceClaim::proven(
                    "deepseek-harness",
                    EvidenceClass::RealRuntime,
                    vec!["attestation://dsh/live".to_string()],
                    "deployment-attestor",
                    "live runtime claimed",
                )
                .unwrap(),
            )
            .unwrap();
        let axes = release_readiness_axes(
            &ledger,
            RuntimeReadinessEvidence::healthy(
                vec!["health://dsh/live".to_string()],
                "healthy but identity missing",
            ),
            RuntimeReadinessEvidence::unavailable("not configured"),
        );
        assert_eq!(axes[3].state, ReadinessState::IdentityMismatch);
        assert!(axes[3].reason.contains("no exact active runtime identity"));
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
