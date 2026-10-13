//! Deployment-owned provenance contract for a real BioLab dataset.
//!
//! The reference BioLab fixture may use synthetic metadata, but a real pilot
//! must never be presented as customer/scientific evidence unless the dataset
//! itself is pinned to lawful, reviewable provenance.

use morn_kernel::error::{Error, Result};
use morn_kernel::time::Timestamp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BioLabDatasetProvenance {
    pub provenance_id: String,
    pub dataset_ref: String,
    pub content_digest: String,
    pub schema_ref: String,
    pub source_system_ref: String,
    pub governance_ref: String,
    pub license_or_consent_ref: String,
    pub collection_provenance_refs: Vec<String>,
    pub issuer: String,
    pub observed_at: Timestamp,
}

impl BioLabDatasetProvenance {
    pub fn validate(&self, now: Timestamp) -> Result<()> {
        for (name, value) in [
            ("provenance_id", self.provenance_id.as_str()),
            ("dataset_ref", self.dataset_ref.as_str()),
            ("schema_ref", self.schema_ref.as_str()),
            ("source_system_ref", self.source_system_ref.as_str()),
            ("governance_ref", self.governance_ref.as_str()),
            ("license_or_consent_ref", self.license_or_consent_ref.as_str()),
            ("issuer", self.issuer.as_str()),
        ] {
            if value.trim().is_empty() {
                return Err(Error::validation(format!(
                    "BioLab dataset provenance requires non-empty {name}"
                )));
            }
            if value.chars().any(char::is_control) {
                return Err(Error::validation(format!(
                    "BioLab dataset provenance {name} must not contain control characters"
                )));
            }
        }

        let digest = self.content_digest.trim();
        let canonical_digest = digest.len() == 71
            && digest.starts_with("sha256:")
            && digest[7..]
                .chars()
                .all(|ch| ch.is_ascii_hexdigit() && !ch.is_ascii_uppercase());
        if !canonical_digest {
            return Err(Error::validation(
                "BioLab dataset content_digest must be canonical sha256:<64-lowercase-hex>",
            ));
        }

        if self.collection_provenance_refs.is_empty()
            || self
                .collection_provenance_refs
                .iter()
                .any(|reference| reference.trim().is_empty())
        {
            return Err(Error::validation(
                "BioLab real dataset requires explicit non-empty collection provenance references",
            ));
        }
        if self.observed_at > now {
            return Err(Error::validation(
                "future-dated BioLab dataset provenance cannot be admitted",
            ));
        }
        Ok(())
    }

    pub fn canonical_identity(&self) -> String {
        format!(
            "{}#{}",
            self.dataset_ref.trim(),
            self.content_digest.trim()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> BioLabDatasetProvenance {
        BioLabDatasetProvenance {
            provenance_id: "biolab-provenance-1".to_string(),
            dataset_ref: "dataset://customer-lab/aging-pilot-v1".to_string(),
            content_digest: format!("sha256:{}", "a".repeat(64)),
            schema_ref: "schema://biolab/aging-pilot@1".to_string(),
            source_system_ref: "lims://customer-lab/export-42".to_string(),
            governance_ref: "governance://customer-lab/research-approval-42".to_string(),
            license_or_consent_ref: "consent://protocol/42".to_string(),
            collection_provenance_refs: vec![
                "provenance://collection/batch-42".to_string(),
                "provenance://deidentification/review-42".to_string(),
            ],
            issuer: "customer-data-governance".to_string(),
            observed_at: Timestamp::from_millis(10),
        }
    }

    #[test]
    fn real_dataset_provenance_requires_lawful_source_and_exact_digest() {
        let provenance = fixture();
        provenance.validate(Timestamp::from_millis(11)).unwrap();
        assert_eq!(
            provenance.canonical_identity(),
            format!(
                "dataset://customer-lab/aging-pilot-v1#sha256:{}",
                "a".repeat(64)
            )
        );
    }

    #[test]
    fn real_dataset_provenance_rejects_missing_consent_bad_digest_and_future_time() {
        let mut provenance = fixture();
        provenance.license_or_consent_ref.clear();
        assert!(provenance.validate(Timestamp::from_millis(11)).is_err());

        let mut provenance = fixture();
        provenance.content_digest = "sha256:NOT-A-DIGEST".to_string();
        assert!(provenance.validate(Timestamp::from_millis(11)).is_err());

        let mut provenance = fixture();
        provenance.observed_at = Timestamp::from_millis(12);
        assert!(provenance.validate(Timestamp::from_millis(11)).is_err());
    }
}
