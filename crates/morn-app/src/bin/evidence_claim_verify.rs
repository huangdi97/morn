use std::fs;

use morn_assurance::{reference_evidence_ledger, EvidenceClaim, EvidenceClaimState, EvidenceClass};
use morn_kernel::error::{Error, Result};
use morn_kernel::time::Timestamp;
use serde_json::json;

fn expected_class(value: &str) -> Result<EvidenceClass> {
    match value.trim() {
        "real-site" => Ok(EvidenceClass::RealSite),
        "production-write" => Ok(EvidenceClass::ProductionWrite),
        other => Err(Error::validation(format!(
            "unsupported expected evidence class {other:?}; expected real-site or production-write"
        ))),
    }
}

fn main() -> Result<()> {
    let path = std::env::var("MORN_EVIDENCE_CLAIM_FILE")
        .map_err(|_| Error::validation("MORN_EVIDENCE_CLAIM_FILE is required"))?;
    let issuer = std::env::var("MORN_EVIDENCE_ISSUER")
        .map_err(|_| Error::validation("MORN_EVIDENCE_ISSUER is required"))?;
    let expected_subject = std::env::var("MORN_EVIDENCE_EXPECTED_SUBJECT")
        .map_err(|_| Error::validation("MORN_EVIDENCE_EXPECTED_SUBJECT is required"))?;
    let expected_class = expected_class(
        &std::env::var("MORN_EVIDENCE_EXPECTED_CLASS")
            .map_err(|_| Error::validation("MORN_EVIDENCE_EXPECTED_CLASS is required"))?,
    )?;

    if path.trim().is_empty() || issuer.trim().is_empty() || expected_subject.trim().is_empty() {
        return Err(Error::validation(
            "evidence file, issuer and expected subject must be non-empty",
        ));
    }

    let raw = fs::read_to_string(&path).map_err(|error| {
        Error::external(format!("cannot read evidence claim file {path:?}: {error}"))
    })?;
    let claim: EvidenceClaim = serde_json::from_str(&raw).map_err(|error| {
        Error::validation(format!("invalid EvidenceClaim JSON in {path:?}: {error}"))
    })?;

    if claim.subject != expected_subject.trim() {
        return Err(Error::validation(format!(
            "claim subject {:?} does not match exact expected subject {:?}",
            claim.subject,
            expected_subject.trim()
        )));
    }
    if claim.class != expected_class {
        return Err(Error::validation(format!(
            "claim class {} does not match expected class {}",
            claim.class.key(),
            expected_class.key()
        )));
    }
    if claim.state != EvidenceClaimState::Proven {
        return Err(Error::validation(
            "customer/deployment evidence gate accepts only explicit Proven claims",
        ));
    }
    if claim.observed_at > Timestamp::now() {
        return Err(Error::validation(
            "future-dated customer/deployment evidence cannot be admitted",
        ));
    }

    // Match normal deployment startup exactly: external claims are appended
    // after repository/reference baseline claims, so duplicate ids or history
    // conflicts cannot pass this handoff and fail only later at server boot.
    let mut ledger = reference_evidence_ledger();
    ledger.append_from_trusted_issuer(claim.clone(), issuer.trim())?;

    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "gate": "MORN_DEPLOYMENT_EVIDENCE_CLAIM_VERIFY",
            "status": "PASS",
            "claim": claim,
            "claims": {
                "schema_and_semantics_validated": true,
                "issuer_identity_pinned": true,
                "external_evidence_cryptographically_verified_by_morn": false,
                "customer_fact_minted_by_morn": false,
                "production_write_authority_granted": false
            },
            "note": "This gate validates the protected deployment envelope and exact subject/class only. The external governance system remains responsible for the evidence referenced by the claim."
        }))
        .map_err(|error| Error::internal(error.to_string()))?
    );
    Ok(())
}
