//! Real BioLab Pilot v1: adapter/manifest/provenance/validation for a lawful
//! real dataset. No fabricated "real data": if no lawful real dataset is
//! registered, the FULL pilot is BLOCKED while CORE (adapter/contract) stands.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{DatasetSnapshotId, PilotManifestId};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

/// Pilot manifest: provenance of a real dataset (never fabricated).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PilotManifest {
    pub id: PilotManifestId,
    pub source: String,
    pub license: String,
    pub dataset_id: String,
    pub dataset_version: Version,
    pub checksum: String,
    pub acquisition_date: String,
    pub subset_rule: String,
    pub preprocessing: String,
    pub schema: String,
    pub reference_evidence: Vec<String>,
    pub created_at: Timestamp,
}

impl PilotManifest {
    pub fn new(
        source: impl Into<String>,
        license: impl Into<String>,
        dataset_id: impl Into<String>,
    ) -> Self {
        Self {
            id: PilotManifestId::generate_with("pilot"),
            source: source.into(),
            license: license.into(),
            dataset_id: dataset_id.into(),
            dataset_version: Version::v1(),
            checksum: String::new(),
            acquisition_date: String::new(),
            subset_rule: "all".to_string(),
            preprocessing: "none".to_string(),
            schema: "biolab.dataset.v1".to_string(),
            reference_evidence: Vec::new(),
            created_at: Timestamp::now(),
        }
    }
}

/// A pilot pipeline run record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PilotRunRecord {
    pub snapshot_ref: DatasetSnapshotId,
    pub predictions_before_run: u32,
    pub delivery_accepted: bool,
    pub actual_metrics_ref: String,
    pub prediction_error_ref: String,
    pub calibration_ref: String,
    pub ran_at: Timestamp,
}

/// Real pilot service.
#[derive(Debug, Default)]
pub struct RealPilotService {
    pub manifest: Option<PilotManifest>,
    pub runs: Vec<PilotRunRecord>,
}

impl RealPilotService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a real dataset manifest. Validation rejects fabricated data:
    /// source/license must be non-empty and a checksum must be present.
    pub fn register_manifest(&mut self, manifest: PilotManifest) -> Result<()> {
        if manifest.source.trim().is_empty() {
            return Err(Error::validation("pilot source must be a real data source"));
        }
        if manifest.license.trim().is_empty() {
            return Err(Error::validation("pilot license must be declared"));
        }
        if manifest.source.eq_ignore_ascii_case("fixture")
            || manifest.source.eq_ignore_ascii_case("synthetic")
        {
            return Err(Error::validation(
                "synthetic/fixture data cannot be registered as a real pilot",
            ));
        }
        if manifest.checksum.trim().is_empty() {
            return Err(Error::validation("pilot dataset checksum must be recorded"));
        }
        self.manifest = Some(manifest);
        Ok(())
    }

    /// The full pilot pipeline (real data -> predictions -> managed work ->
    /// actual -> error -> calibration). Requires a registered real manifest.
    pub fn run_pipeline(
        &mut self,
        snapshot_ref: DatasetSnapshotId,
        predictions_before_run: u32,
        delivery_accepted: bool,
        actual_metrics_ref: &str,
        prediction_error_ref: &str,
        calibration_ref: &str,
    ) -> Result<PilotRunRecord> {
        if self.manifest.is_none() {
            return Err(Error::external(
                "no lawful real BioLab dataset registered; FULL pilot is BLOCKED (adapter/contract CORE is complete)",
            ));
        }
        let record = PilotRunRecord {
            snapshot_ref,
            predictions_before_run,
            delivery_accepted,
            actual_metrics_ref: actual_metrics_ref.to_string(),
            prediction_error_ref: prediction_error_ref.to_string(),
            calibration_ref: calibration_ref.to_string(),
            ran_at: Timestamp::now(),
        };
        self.runs.push(record.clone());
        Ok(record)
    }

    pub fn full_pilot_available(&self) -> bool {
        self.manifest.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fabricated_manifest_rejected() {
        let mut svc = RealPilotService::new();
        let mut m = PilotManifest::new("synthetic", "internal", "ds-1");
        m.checksum = "abc".to_string();
        assert!(
            svc.register_manifest(m).is_err(),
            "synthetic source must be rejected"
        );
    }

    #[test]
    fn real_manifest_registers_and_pipeline_blocks_without_data() {
        let mut svc = RealPilotService::new();
        // No manifest -> FULL pilot blocked.
        assert!(svc
            .run_pipeline(DatasetSnapshotId::generate(), 3, true, "m", "e", "c")
            .is_err());
        let mut m = PilotManifest::new("public-geo-dataset", "CC0", "GSE12345");
        m.checksum = "sha256:deadbeef".to_string();
        svc.register_manifest(m).unwrap();
        assert!(svc.full_pilot_available());
        let record = svc
            .run_pipeline(DatasetSnapshotId::generate(), 3, true, "m", "e", "c")
            .unwrap();
        assert!(record.delivery_accepted);
        assert_eq!(svc.runs.len(), 1);
    }
}
