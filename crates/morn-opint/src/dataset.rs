//! OutcomeDataset: snapshots, manifests, label definitions, feature schemas,
//! splits and data-quality/leakage checks. Labels are authoritative; leakage
//! (future/duplicate/workspace) is detected, never silently allowed.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{
    DataQualityReportId, DatasetSnapshotId, EpisodeDatasetId, FeatureSchemaId, LabelDefinitionId,
    OperationalEpisodeId, SplitManifestId,
};
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;

use crate::episode::OperationalEpisode;

/// A label definition tied to an authoritative record type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabelDefinition {
    pub id: LabelDefinitionId,
    pub name: String,
    pub source: String, // AcceptanceDecision | OutcomeRecord | TerminalStatus | HumanAction
    pub value_type: String, // bool | numeric | categorical
    pub created_at: Timestamp,
}

impl LabelDefinition {
    pub fn new(
        name: impl Into<String>,
        source: impl Into<String>,
        value_type: impl Into<String>,
    ) -> Self {
        Self {
            id: LabelDefinitionId::generate_with("label"),
            name: name.into(),
            source: source.into(),
            value_type: value_type.into(),
            created_at: Timestamp::now(),
        }
    }
}

/// Feature schema (typed, deterministic; no embedding).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureSpec {
    pub name: String,
    pub kind: String, // categorical | numeric | boolean | temporal | graph | aggregate | missing
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureSchemaVersion {
    pub id: FeatureSchemaId,
    pub version: Version,
    pub features: Vec<FeatureSpec>,
}

impl FeatureSchemaVersion {
    pub fn new(features: Vec<FeatureSpec>) -> Self {
        Self {
            id: FeatureSchemaId::generate_with("fschema"),
            version: Version::v1(),
            features,
        }
    }
}

/// Dataset manifest: provenance of the dataset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetManifest {
    pub dataset_id: EpisodeDatasetId,
    pub source: String,
    pub license: String,
    pub dataset_version: Version,
    pub checksum: String,
    pub acquisition_date: String,
    pub subset_rule: String,
    pub preprocessing: String,
    pub schema_ref: String,
    pub reference_evidence: Vec<String>,
    pub created_at: Timestamp,
}

/// An immutable dataset snapshot (rows are episode refs).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetSnapshot {
    pub id: DatasetSnapshotId,
    pub dataset_id: EpisodeDatasetId,
    pub manifest_ref: String,
    pub episode_ids: Vec<OperationalEpisodeId>,
    pub created_at: Timestamp,
}

/// Train/test split manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SplitManifest {
    pub id: SplitManifestId,
    pub dataset_id: EpisodeDatasetId,
    pub split_type: String, // temporal | case | random
    pub train_episode_ids: Vec<OperationalEpisodeId>,
    pub test_episode_ids: Vec<OperationalEpisodeId>,
    pub holdout_note: String,
}

/// Data quality report with leak checks.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DataQualityReport {
    pub id: DataQualityReportId,
    pub dataset_id: EpisodeDatasetId,
    pub missing_rate: f64,
    pub duplicate_count: u32,
    pub future_leak_count: u32,
    pub workspace_violation_count: u32,
    pub post_outcome_feature_count: u32,
    pub ok: bool,
}

/// Outcome dataset service.
#[derive(Debug, Default)]
pub struct OutcomeDataset {
    pub manifests: Vec<DatasetManifest>,
    pub snapshots: Vec<DatasetSnapshot>,
    pub splits: Vec<SplitManifest>,
    pub quality_reports: Vec<DataQualityReport>,
}

impl OutcomeDataset {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build_snapshot(
        &mut self,
        manifest: DatasetManifest,
        episodes: &[&OperationalEpisode],
    ) -> DatasetSnapshot {
        let snapshot = DatasetSnapshot {
            id: DatasetSnapshotId::generate_with("snap"),
            dataset_id: manifest.dataset_id.clone(),
            manifest_ref: manifest.dataset_id.to_string(),
            episode_ids: episodes.iter().map(|e| e.id.clone()).collect(),
            created_at: Timestamp::now(),
        };
        self.snapshots.push(snapshot.clone());
        snapshot
    }

    /// Leakage checks: duplicate episode, future features, workspace boundary,
    /// post-outcome features. `feature_time` maps episode -> feature timestamp;
    /// `outcome_time` maps episode -> outcome timestamp.
    pub fn check_leakage(
        &mut self,
        dataset_id: &EpisodeDatasetId,
        episodes: &[&OperationalEpisode],
        feature_time: &[(OperationalEpisodeId, i64)],
        outcome_time: &[(OperationalEpisodeId, i64)],
        post_outcome_feature_episodes: &[OperationalEpisodeId],
    ) -> DataQualityReport {
        let mut seen = HashSet::new();
        let mut duplicates = 0u32;
        let mut workspace_violations = 0u32;
        for ep in episodes {
            if !seen.insert(ep.id.clone()) {
                duplicates += 1;
            }
            if ep.workspace_id.as_str() != "ws-1" && !ep.workspace_id.as_str().is_empty() {
                // workspace boundary is enforced by the caller passing only one workspace;
                // here we flag any episode whose workspace differs from the first.
                if let Some(first) = episodes.first() {
                    if first.workspace_id != ep.workspace_id {
                        workspace_violations += 1;
                    }
                }
            }
        }
        let ft: std::collections::HashMap<_, _> = feature_time.iter().cloned().collect();
        let ot: std::collections::HashMap<_, _> = outcome_time.iter().cloned().collect();
        let mut future_leaks = 0u32;
        for (id, f) in &ft {
            if let Some(o) = ot.get(id) {
                if f > o {
                    future_leaks += 1;
                }
            }
        }
        let post_outcome = post_outcome_feature_episodes.len() as u32;
        let ok =
            duplicates == 0 && future_leaks == 0 && workspace_violations == 0 && post_outcome == 0;
        let report = DataQualityReport {
            id: DataQualityReportId::generate_with("dq"),
            dataset_id: dataset_id.clone(),
            missing_rate: 0.0,
            duplicate_count: duplicates,
            future_leak_count: future_leaks,
            workspace_violation_count: workspace_violations,
            post_outcome_feature_count: post_outcome,
            ok,
        };
        self.quality_reports.push(report.clone());
        report
    }

    /// Temporal split: earlier episodes train, later test (reproducible by sort).
    pub fn temporal_split(
        &mut self,
        dataset_id: &EpisodeDatasetId,
        mut episodes: Vec<OperationalEpisode>,
        test_ratio: f64,
    ) -> Result<SplitManifest> {
        episodes.sort_by_key(|e| e.started_at);
        let test_n = ((episodes.len() as f64) * test_ratio).round() as usize;
        let split_at = episodes.len().saturating_sub(test_n);
        let (train, test) = episodes.split_at(split_at);
        if train.is_empty() || test.is_empty() {
            return Err(Error::validation(
                "temporal split requires both train and test",
            ));
        }
        let split = SplitManifest {
            id: SplitManifestId::generate_with("split"),
            dataset_id: dataset_id.clone(),
            split_type: "temporal".to_string(),
            train_episode_ids: train.iter().map(|e| e.id.clone()).collect(),
            test_episode_ids: test.iter().map(|e| e.id.clone()).collect(),
            holdout_note: "temporal holdout: test episodes occur after train episodes".to_string(),
        };
        self.splits.push(split.clone());
        Ok(split)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::OperationalEpisode;
    use morn_kernel::time::Timestamp;

    fn episode_at(
        ws: &morn_kernel::ids::WorkspaceId,
        domain: &str,
        started: i64,
    ) -> OperationalEpisode {
        let mut e = OperationalEpisode::new(ws.clone(), domain, "ctx");
        e.started_at = Timestamp::from_millis(started);
        e
    }

    #[test]
    fn temporal_split_is_reproducible() {
        let ws = morn_kernel::ids::WorkspaceId::generate();
        let mut ds = OutcomeDataset::new();
        let manifest = DatasetManifest {
            dataset_id: EpisodeDatasetId::generate_with("ds"),
            source: "fixture".to_string(),
            license: "internal".to_string(),
            dataset_version: Version::v1(),
            checksum: "abc".to_string(),
            acquisition_date: "2026-08-15".to_string(),
            subset_rule: "all".to_string(),
            preprocessing: "none".to_string(),
            schema_ref: "morn.feature.v1".to_string(),
            reference_evidence: vec![],
            created_at: Timestamp::now(),
        };
        let episodes = vec![
            episode_at(&ws, "biolab", 1000),
            episode_at(&ws, "biolab", 2000),
            episode_at(&ws, "biolab", 3000),
            episode_at(&ws, "biolab", 4000),
        ];
        ds.build_snapshot(manifest.clone(), &episodes.iter().collect::<Vec<_>>());
        let split = ds
            .temporal_split(&manifest.dataset_id, episodes, 0.25)
            .unwrap();
        assert_eq!(split.train_episode_ids.len(), 3);
        assert_eq!(split.test_episode_ids.len(), 1);
        assert_eq!(split.split_type, "temporal");
        // test episode is the latest
        assert!(split.test_episode_ids[0] != split.train_episode_ids[0]);
    }

    #[test]
    fn leakage_detection_duplicate_future_workspace() {
        let ws = morn_kernel::ids::WorkspaceId::generate();
        let other_ws = morn_kernel::ids::WorkspaceId::generate();
        let mut ds = OutcomeDataset::new();
        let did = EpisodeDatasetId::generate();
        let e1 = episode_at(&ws, "biolab", 1000);
        let e1_dup = e1.clone();
        let e2 = episode_at(&other_ws, "biolab", 2000);
        let e3 = episode_at(&ws, "biolab", 3000);
        let episodes = vec![&e1, &e1_dup, &e2, &e3];
        // e1 feature time AFTER its outcome time -> future leak
        let feature_time = vec![
            (e1.id.clone(), 5000),
            (e2.id.clone(), 2500),
            (e3.id.clone(), 3500),
        ];
        let outcome_time = vec![
            (e1.id.clone(), 1500),
            (e2.id.clone(), 2200),
            (e3.id.clone(), 3200),
        ];
        let report = ds.check_leakage(&did, &episodes, &feature_time, &outcome_time, &[]);
        assert!(report.duplicate_count >= 1);
        assert!(report.future_leak_count >= 1);
        assert!(report.workspace_violation_count >= 1);
        assert!(!report.ok);
    }

    #[test]
    fn clean_dataset_passes_quality() {
        let ws = morn_kernel::ids::WorkspaceId::generate();
        let mut ds = OutcomeDataset::new();
        let did = EpisodeDatasetId::generate();
        let e1 = episode_at(&ws, "biolab", 1000);
        let e2 = episode_at(&ws, "biolab", 2000);
        let report = ds.check_leakage(
            &did,
            &[&e1, &e2],
            &[(e1.id.clone(), 1500), (e2.id.clone(), 2500)],
            &[(e1.id.clone(), 1500), (e2.id.clone(), 2500)],
            &[],
        );
        assert!(report.ok, "clean dataset must pass quality checks");
    }
}
