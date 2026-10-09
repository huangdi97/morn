//! Evidence-class ledger for non-claim discipline.
//!
//! A passing fixture or CI suite proves only its own evidence class. It never
//! auto-promotes a real-runtime, real-site or production-write claim.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::Id;
use morn_kernel::time::Timestamp;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceClass {
    DesignSpec,
    LocalFixture,
    CiConformance,
    RealRuntime,
    RealSite,
    ProductionWrite,
}

impl EvidenceClass {
    pub const fn key(self) -> &'static str {
        match self {
            Self::DesignSpec => "design-spec",
            Self::LocalFixture => "local-fixture",
            Self::CiConformance => "ci-conformance",
            Self::RealRuntime => "real-runtime",
            Self::RealSite => "real-site",
            Self::ProductionWrite => "production-write",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceClaimState {
    Proven,
    BlockedExternal,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct EvidenceClaimTag;
pub type EvidenceClaimId = Id<EvidenceClaimTag>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceClaim {
    pub id: EvidenceClaimId,
    pub subject: String,
    pub class: EvidenceClass,
    pub state: EvidenceClaimState,
    pub evidence_refs: Vec<String>,
    pub issuer: String,
    pub reason: String,
    pub observed_at: Timestamp,
}

impl EvidenceClaim {
    pub fn proven(
        subject: impl Into<String>,
        class: EvidenceClass,
        evidence_refs: Vec<String>,
        issuer: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<Self> {
        if evidence_refs.is_empty()
            || evidence_refs
                .iter()
                .any(|reference| reference.trim().is_empty())
        {
            return Err(Error::validation(
                "a proven evidence claim requires explicit non-empty evidence references",
            ));
        }
        let subject = subject.into();
        let issuer = issuer.into();
        let reason = reason.into();
        if subject.trim().is_empty() {
            return Err(Error::validation("evidence claim subject is required"));
        }
        if issuer.trim().is_empty() {
            return Err(Error::validation("evidence claim issuer is required"));
        }
        if reason.trim().is_empty() {
            return Err(Error::validation("proven evidence claim reason is required"));
        }
        Ok(Self {
            id: EvidenceClaimId::generate_with("evidence-claim"),
            subject,
            class,
            state: EvidenceClaimState::Proven,
            evidence_refs,
            issuer,
            reason,
            observed_at: Timestamp::now(),
        })
    }

    pub fn blocked_external(
        subject: impl Into<String>,
        class: EvidenceClass,
        issuer: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<Self> {
        let reason = reason.into();
        if reason.trim().is_empty() {
            return Err(Error::validation(
                "external blocker requires an explicit reason",
            ));
        }
        Ok(Self {
            id: EvidenceClaimId::generate_with("evidence-claim"),
            subject: subject.into(),
            class,
            state: EvidenceClaimState::BlockedExternal,
            evidence_refs: Vec::new(),
            issuer: issuer.into(),
            reason,
            observed_at: Timestamp::now(),
        })
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EvidenceLedger {
    claims: Vec<EvidenceClaim>,
}

impl EvidenceLedger {
    pub fn append(&mut self, claim: EvidenceClaim) -> Result<()> {
        if claim.subject.trim().is_empty() {
            return Err(Error::validation("evidence claim subject is required"));
        }
        if claim.issuer.trim().is_empty() {
            return Err(Error::validation("evidence claim issuer is required"));
        }
        if claim.state == EvidenceClaimState::Proven
            && (claim.evidence_refs.is_empty()
                || claim
                    .evidence_refs
                    .iter()
                    .any(|reference| reference.trim().is_empty())
                || claim.reason.trim().is_empty())
        {
            return Err(Error::validation(
                "a proven evidence claim requires non-empty evidence references and reason",
            ));
        }
        if claim.state == EvidenceClaimState::BlockedExternal && claim.reason.trim().is_empty() {
            return Err(Error::validation(
                "an external blocker requires an explicit reason",
            ));
        }
        if self.claims.iter().any(|existing| existing.id == claim.id) {
            return Err(Error::conflict(format!(
                "evidence claim {} already exists",
                claim.id
            )));
        }
        self.claims.push(claim);
        Ok(())
    }

    pub fn claims(&self) -> &[EvidenceClaim] {
        &self.claims
    }

    pub fn proven_for(&self, subject: &str) -> Vec<&EvidenceClaim> {
        self.claims
            .iter()
            .filter(|claim| claim.subject == subject && claim.state == EvidenceClaimState::Proven)
            .collect()
    }

    /// Evidence classes are categorical, not a trust ladder. CI evidence does
    /// not semantically include local-fixture evidence, and real-site evidence
    /// does not silently include a production-write claim.
    pub fn satisfies(&self, subject: &str, required: EvidenceClass) -> bool {
        self.proven_for(subject)
            .into_iter()
            .any(|claim| claim.class == required)
    }

    pub fn proven_classes(&self, subject: &str) -> std::collections::BTreeSet<EvidenceClass> {
        self.proven_for(subject)
            .into_iter()
            .map(|claim| claim.class)
            .collect()
    }

    /// This deliberately does not synthesize one evidence class from another.
    /// A caller must append an explicit claim with evidence from each required
    /// class.
    pub fn require(&self, subject: &str, required: EvidenceClass) -> Result<()> {
        if self.satisfies(subject, required) {
            Ok(())
        } else {
            Err(Error::invalid_state(format!(
                "subject {subject} lacks explicit evidence class {}; evidence classes do not auto-promote one another",
                required.key()
            )))
        }
    }

    pub fn require_all(&self, subject: &str, required: &[EvidenceClass]) -> Result<()> {
        for class in required {
            self.require(subject, *class)?;
        }
        Ok(())
    }
}

/// Repository/reference claims only. These entries deliberately stop at the
/// evidence actually available without external runtime/site credentials.
pub fn reference_evidence_ledger() -> EvidenceLedger {
    let mut ledger = EvidenceLedger::default();
    ledger
        .append(
            EvidenceClaim::proven(
                "morn-v11.5-architecture",
                EvidenceClass::DesignSpec,
                vec!["docs/architecture-v11.5.md".to_string()],
                "morn-reference-runtime",
                "versioned architecture and ADR baseline exists",
            )
            .expect("static design claim valid"),
        )
        .expect("static design claim append");

    ledger
        .append(
            EvidenceClaim::proven(
                "factory-readonly-wedge",
                EvidenceClass::LocalFixture,
                vec!["crates/morn-control-plane/tests/factory_readonly_slice.rs".to_string()],
                "morn-reference-runtime",
                "deterministic Factory read-only control-plane slice exists",
            )
            .expect("static fixture claim valid"),
        )
        .expect("static fixture claim append");

    for (subject, reason) in [
        (
            "deepseek-harness",
            "real DSH adapter exists; authenticated runtime/model/credential and attested environment evidence remain external to deterministic contract tests",
        ),
        (
            "pi-harness",
            "real Pi adapter exists; installed runtime/model/credential and attested environment evidence remain external to deterministic contract tests",
        ),
    ] {
        ledger
            .append(
                EvidenceClaim::blocked_external(
                    subject,
                    EvidenceClass::RealRuntime,
                    "morn-reference-runtime",
                    reason,
                )
                .expect("static blocker valid"),
            )
            .expect("static blocker append");
    }

    ledger
        .append(
            EvidenceClaim::blocked_external(
                "factory-customer",
                EvidenceClass::RealSite,
                "morn-reference-runtime",
                "lawful customer/site data, system access and authority are not present",
            )
            .expect("static site blocker valid"),
        )
        .expect("static site blocker append");

    ledger
        .append(
            EvidenceClaim::blocked_external(
                "factory-production-write",
                EvidenceClass::ProductionWrite,
                "morn-reference-runtime",
                "production write and physical control are explicitly not entered",
            )
            .expect("static write blocker valid"),
        )
        .expect("static write blocker append");

    ledger
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proven_claim_requires_auditable_reason_and_non_empty_references() {
        assert!(EvidenceClaim::proven(
            "subject",
            EvidenceClass::RealRuntime,
            vec!["runtime://evidence".to_string()],
            "deployment",
            "",
        )
        .is_err());
        assert!(EvidenceClaim::proven(
            "subject",
            EvidenceClass::RealRuntime,
            vec![" ".to_string()],
            "deployment",
            "runtime attested",
        )
        .is_err());
    }

    #[test]
    fn deserialized_proven_claim_cannot_bypass_evidence_requirements() {
        let claim: EvidenceClaim = serde_json::from_value(serde_json::json!({
            "id": "evidence-claim:deployment-forged",
            "subject": "value:work:g1:outcome",
            "class": "real-site",
            "state": "proven",
            "evidence_refs": [],
            "issuer": "deployment-loader",
            "reason": "missing actual witness",
            "observed_at": "1970-01-01T00:00:00Z"
        }))
        .unwrap();
        let mut ledger = EvidenceLedger::default();
        assert!(ledger.append(claim).is_err());
    }

    #[test]
    fn reference_ledger_never_upgrades_external_blockers() {
        let ledger = reference_evidence_ledger();
        assert!(ledger.satisfies("factory-readonly-wedge", EvidenceClass::LocalFixture));
        assert!(!ledger.satisfies("deepseek-harness", EvidenceClass::RealRuntime));
        assert!(!ledger.satisfies("factory-customer", EvidenceClass::RealSite));
        assert!(!ledger.satisfies("factory-production-write", EvidenceClass::ProductionWrite));
    }

    #[test]
    fn evidence_classes_are_categories_not_an_ordinal_ladder() {
        let mut ledger = EvidenceLedger::default();
        ledger
            .append(
                EvidenceClaim::proven(
                    "subject",
                    EvidenceClass::RealSite,
                    vec!["site://read-only".to_string()],
                    "site-evaluator",
                    "real site observation",
                )
                .unwrap(),
            )
            .unwrap();

        assert!(ledger.satisfies("subject", EvidenceClass::RealSite));
        assert!(!ledger.satisfies("subject", EvidenceClass::CiConformance));
        assert!(!ledger.satisfies("subject", EvidenceClass::ProductionWrite));
    }

    #[test]
    fn fixture_and_ci_evidence_cannot_claim_real_runtime() {
        let mut ledger = EvidenceLedger::default();
        ledger
            .append(
                EvidenceClaim::proven(
                    "deepseek-harness",
                    EvidenceClass::LocalFixture,
                    vec!["test://dsh-contract".to_string()],
                    "morn-ci",
                    "fixture provider contract passed",
                )
                .unwrap(),
            )
            .unwrap();
        ledger
            .append(
                EvidenceClaim::proven(
                    "deepseek-harness",
                    EvidenceClass::CiConformance,
                    vec!["github-actions://run/1".to_string()],
                    "github-actions",
                    "conformance gate passed",
                )
                .unwrap(),
            )
            .unwrap();

        assert!(ledger.satisfies("deepseek-harness", EvidenceClass::CiConformance));
        assert!(ledger.satisfies("deepseek-harness", EvidenceClass::LocalFixture));
        assert!(!ledger.satisfies("deepseek-harness", EvidenceClass::RealRuntime));
        assert!(ledger
            .require("deepseek-harness", EvidenceClass::RealRuntime)
            .is_err());
    }

    #[test]
    fn external_blocker_is_visible_but_not_proof() {
        let mut ledger = EvidenceLedger::default();
        ledger
            .append(
                EvidenceClaim::blocked_external(
                    "factory-customer",
                    EvidenceClass::RealSite,
                    "morn-engineering",
                    "customer data and site authorization not available",
                )
                .unwrap(),
            )
            .unwrap();

        assert!(!ledger.satisfies("factory-customer", EvidenceClass::RealSite));
        assert_eq!(
            ledger.claims()[0].state,
            EvidenceClaimState::BlockedExternal
        );
    }

    #[test]
    fn production_write_requires_explicit_production_write_evidence() {
        let mut ledger = EvidenceLedger::default();
        ledger
            .append(
                EvidenceClaim::proven(
                    "factory-write",
                    EvidenceClass::RealSite,
                    vec!["site://pilot/read-only".to_string()],
                    "site-evaluator",
                    "real read-only observation",
                )
                .unwrap(),
            )
            .unwrap();

        assert!(!ledger.satisfies("factory-write", EvidenceClass::ProductionWrite));
    }
}
