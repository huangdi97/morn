//! ArtifactService: governed artifact lifecycle with immutable versions.

use std::collections::HashMap;

use serde_json::Value;

use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{ArtifactId, ArtifactVersionId, PrincipalId, WorkspaceId};
use morn_kernel::status::ArtifactStatus;

use crate::approval::{ArtifactApproval, ArtifactApprovalDecision};
use crate::artifact::{Artifact, ArtifactVersion};
use crate::provenance::{ProvRelation, ProvenanceGraph};
use crate::review::{Review, ReviewDecision};

/// In-memory artifact store. Repository adapters persist externally.
#[derive(Debug, Default)]
pub struct ArtifactService {
    artifacts: HashMap<ArtifactId, Artifact>,
    versions: Vec<ArtifactVersion>,
    reviews: Vec<Review>,
    approvals: Vec<ArtifactApproval>,
}

impl ArtifactService {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new artifact with its first (draft) version.
    #[allow(clippy::too_many_arguments)]
    pub fn create(
        &mut self,
        artifact_type: &str,
        schema_version: &str,
        workspace_id: WorkspaceId,
        created_by: PrincipalId,
        content_ref: &str,
        structured_content: Value,
        checksum: &str,
    ) -> Result<(Artifact, ArtifactVersion)> {
        let artifact = Artifact::new(
            artifact_type,
            schema_version,
            workspace_id,
            created_by.clone(),
        );
        let version = ArtifactVersion::new(
            artifact.id.clone(),
            1,
            content_ref,
            structured_content,
            checksum,
            None,
            None,
            created_by,
        );
        self.artifacts.insert(artifact.id.clone(), artifact.clone());
        self.versions.push(version.clone());
        Ok((artifact, version))
    }

    pub fn artifact(&self, id: &ArtifactId) -> Option<&Artifact> {
        self.artifacts.get(id)
    }

    pub fn version(&self, id: &ArtifactVersionId) -> Option<&ArtifactVersion> {
        self.versions.iter().find(|v| v.id == *id)
    }

    pub fn restore_artifacts(&mut self, artifacts: Vec<Artifact>) {
        for a in artifacts {
            self.artifacts.insert(a.id.clone(), a);
        }
    }

    pub fn restore_versions(&mut self, versions: Vec<ArtifactVersion>) {
        self.versions = versions;
    }

    pub fn all_artifacts(&self) -> Vec<&Artifact> {
        self.artifacts.values().collect()
    }

    pub fn all_version_count(&self) -> usize {
        self.versions.len()
    }

    pub fn versions_of(&self, artifact_id: &ArtifactId) -> Vec<&ArtifactVersion> {
        self.versions
            .iter()
            .filter(|v| v.artifact_id == *artifact_id)
            .collect()
    }

    pub fn latest_version(&self, artifact_id: &ArtifactId) -> Option<&ArtifactVersion> {
        self.versions_of(artifact_id)
            .into_iter()
            .max_by_key(|v| v.version_no)
    }

    /// Submit a draft for review.
    pub fn submit(&mut self, version_id: &ArtifactVersionId) -> Result<()> {
        self.transition(version_id, ArtifactStatus::Submitted)?;
        Ok(())
    }

    pub fn mark_in_review(&mut self, version_id: &ArtifactVersionId) -> Result<()> {
        self.transition(version_id, ArtifactStatus::InReview)?;
        Ok(())
    }

    pub fn request_changes(&mut self, version_id: &ArtifactVersionId) -> Result<()> {
        self.transition(version_id, ArtifactStatus::ChangesRequested)?;
        Ok(())
    }

    /// Record a review decision on a version.
    pub fn review(&mut self, review: Review) -> Result<()> {
        let version = self
            .version_mut(&review.artifact_version_id)
            .ok_or_else(|| Error::not_found(format!("version {}", review.artifact_version_id)))?;
        if !matches!(
            version.status,
            ArtifactStatus::Submitted | ArtifactStatus::InReview
        ) {
            return Err(Error::invalid_state(format!(
                "version {} is not under review (status {:?})",
                version.id, version.status
            )));
        }
        version.status = match review.decision {
            ReviewDecision::Approve => ArtifactStatus::InReview,
            ReviewDecision::RequestChanges => ArtifactStatus::ChangesRequested,
            ReviewDecision::Reject => ArtifactStatus::ChangesRequested,
        };
        self.reviews.push(review);
        Ok(())
    }

    /// Approve a version (requires prior review to have moved it to InReview).
    pub fn approve(&mut self, approval: ArtifactApproval) -> Result<()> {
        let version = self
            .version_mut(&approval.artifact_version_id)
            .ok_or_else(|| Error::not_found(format!("version {}", approval.artifact_version_id)))?;
        if version.status != ArtifactStatus::InReview {
            return Err(Error::invalid_state(format!(
                "version {} must be InReview before approval (status {:?})",
                version.id, version.status
            )));
        }
        if approval.decision == ArtifactApprovalDecision::Approved {
            version.status = ArtifactStatus::Approved;
        }
        self.approvals.push(approval);
        Ok(())
    }

    pub fn lock(&mut self, version_id: &ArtifactVersionId) -> Result<()> {
        self.transition(version_id, ArtifactStatus::Locked)?;
        Ok(())
    }

    /// Supersede an existing version with a new one; the old version remains readable.
    pub fn supersede(
        &mut self,
        old_version_id: &ArtifactVersionId,
        content_ref: &str,
        structured_content: Value,
        checksum: &str,
        created_by: PrincipalId,
    ) -> Result<ArtifactVersion> {
        let old = self
            .version(old_version_id)
            .cloned()
            .ok_or_else(|| Error::not_found(format!("version {old_version_id}")))?;
        let next_no = old.version_no + 1;
        let new_version = ArtifactVersion::new(
            old.artifact_id.clone(),
            next_no,
            content_ref,
            structured_content,
            checksum,
            Some(old_version_id.clone()),
            Some(old_version_id.clone()),
            created_by,
        );
        // Mark old as superseded.
        let old_mut = self
            .version_mut(old_version_id)
            .ok_or_else(|| Error::not_found(format!("version {old_version_id}")))?;
        old_mut.status = ArtifactStatus::Superseded;

        self.versions.push(new_version.clone());
        if let Some(artifact) = self.artifacts.get_mut(&old.artifact_id) {
            artifact.current_version = next_no;
            if artifact.status == ArtifactStatus::Superseded {
                artifact.status = ArtifactStatus::Draft;
            }
        }
        Ok(new_version)
    }

    /// The downstream gate: consumers that require approved content reject drafts.
    pub fn require_approved(&self, version_id: &ArtifactVersionId) -> Result<()> {
        let version = self
            .version(version_id)
            .ok_or_else(|| Error::not_found(format!("version {version_id}")))?;
        if version.status != ArtifactStatus::Approved {
            return Err(Error::forbidden(format!(
                "artifact version {} is not approved (status {:?})",
                version.id, version.status
            )));
        }
        Ok(())
    }

    /// Build the lineage graph of all versions.
    pub fn lineage(&self, artifact_id: &ArtifactId) -> ProvenanceGraph {
        let mut graph = ProvenanceGraph::default();
        for v in self.versions_of(artifact_id) {
            if let Some(derived) = &v.derived_from {
                graph.add(
                    derived.to_string(),
                    v.id.to_string(),
                    ProvRelation::WasDerivedFrom,
                );
            }
        }
        graph
    }

    pub fn reviews(&self) -> &[Review] {
        &self.reviews
    }

    pub fn approvals(&self) -> &[ArtifactApproval] {
        &self.approvals
    }

    fn transition(&mut self, version_id: &ArtifactVersionId, to: ArtifactStatus) -> Result<()> {
        let version = self
            .version_mut(version_id)
            .ok_or_else(|| Error::not_found(format!("version {version_id}")))?;
        if !version.status.can_transition_to(to) {
            return Err(Error::invalid_state(format!(
                "cannot transition artifact version {} from {:?} to {:?}",
                version.id, version.status, to
            )));
        }
        version.status = to;
        Ok(())
    }

    fn version_mut(&mut self, id: &ArtifactVersionId) -> Option<&mut ArtifactVersion> {
        self.versions.iter_mut().find(|v| v.id == *id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::approval::ArtifactApproval;
    use crate::review::{Review, ReviewDecision};
    use morn_kernel::ids::{PrincipalId, WorkspaceId};
    use serde_json::json;

    fn setup() -> (
        ArtifactService,
        ArtifactId,
        ArtifactVersionId,
        PrincipalId,
        WorkspaceId,
    ) {
        let mut svc = ArtifactService::new();
        let ws = WorkspaceId::generate();
        let author = PrincipalId::generate_with("analyst");
        let (artifact, version) = svc
            .create(
                "analysis",
                "biolab/analysis@1",
                ws.clone(),
                author.clone(),
                "ref://v1",
                json!({"x": 1}),
                "c1",
            )
            .unwrap();
        (svc, artifact.id.clone(), version.id.clone(), author, ws)
    }

    #[test]
    fn old_version_remains_readable_after_supersede() {
        let (mut svc, artifact_id, v1_id, author, _ws) = setup();
        let _ = &author;
        let v2 = svc
            .supersede(&v1_id, "ref://v2", json!({"x": 2}), "c2", author.clone())
            .unwrap();
        assert_eq!(v2.version_no, 2);
        // old version still readable
        let v1 = svc.version(&v1_id).unwrap();
        assert_eq!(v1.version_no, 1);
        assert_eq!(v1.status, ArtifactStatus::Superseded);
        assert_eq!(svc.latest_version(&artifact_id).unwrap().version_no, 2);
    }

    #[test]
    fn downstream_rejects_draft_artifact() {
        let (svc, _artifact, v1_id, _author, _ws) = setup();
        assert!(svc.require_approved(&v1_id).is_err());
    }

    #[test]
    fn approval_flow_reaches_approved_and_gate_passes() {
        let (mut svc, _artifact, v1_id, _author, _ws) = setup();
        let reviewer = PrincipalId::generate_with("reviewer");
        let pi = PrincipalId::generate_with("pi");
        svc.submit(&v1_id).unwrap();
        svc.review(Review::new(
            v1_id.clone(),
            reviewer,
            ReviewDecision::Approve,
            "ok",
        ))
        .unwrap();
        svc.approve(ArtifactApproval::approve(v1_id.clone(), pi, None))
            .unwrap();
        assert_eq!(
            svc.version(&v1_id).unwrap().status,
            ArtifactStatus::Approved
        );
        assert!(svc.require_approved(&v1_id).is_ok());
    }

    #[test]
    fn lineage_links_derived_versions() {
        let (mut svc, artifact_id, v1_id, author, _ws) = setup();
        svc.supersede(&v1_id, "ref://v2", json!({"x": 2}), "c2", author)
            .unwrap();
        let graph = svc.lineage(&artifact_id);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.edges[0].relation, ProvRelation::WasDerivedFrom);
    }
}
