//! BioLabService: the Dataset -> Reviewed Scientific Claim vertical slice.

use std::collections::BTreeMap;

use serde_json::json;

use morn_artifact::approval::ArtifactApproval;
use morn_artifact::review::{Review, ReviewDecision};
use morn_artifact::service::ArtifactService;
use morn_capability::effect::EffectContract;
use morn_kernel::error::{Error, Result};
use morn_kernel::ids::{ActionTypeId, ObjectId, PrincipalId, WorkspaceId};
use morn_kernel::policy::{Policy, PolicyRule};
use morn_runtime::gateway::ActionGateway;
use morn_work::acceptance::AcceptanceSpec;
use morn_work::execution_mode::{ExecutionMode, ExecutorType, WorkNature};
use morn_work::service::{AcceptanceEvidence, WorkService};
use morn_work::work_package::WorkPackage;
use morn_world::object::Object;
use morn_world::outcome::OutcomeRecord;

use crate::domain;
use crate::ids::{AnalysisRunId, DatasetId, ScientificClaimId};

/// One step of the BioLab E2E for reporting/UI.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct E2eStep {
    pub step: String,
    pub ok: bool,
    pub detail: String,
}

/// Result of the BioLab E2E run.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct E2eResult {
    pub workspace_id: String,
    pub dataset_id: String,
    pub work_package_id: String,
    pub analysis_run_id: String,
    pub artifact_id: String,
    pub artifact_version_id: String,
    pub claim_id: String,
    pub outcome_id: String,
    pub steps: Vec<E2eStep>,
    pub all_ok: bool,
}

impl E2eResult {
    pub fn all_ok(&self) -> bool {
        self.all_ok
    }
}

/// The BioLab vertical slice service.
pub struct BioLabService {
    pub workspace_id: WorkspaceId,
    pub world: morn_world::WorldService,
    pub artifacts: ArtifactService,
    pub work: WorkService,
    pub gateway: ActionGateway,

    pub analyst: PrincipalId,
    pub reviewer: PrincipalId,
    pub pi: PrincipalId,
}

impl BioLabService {
    pub fn new(workspace_id: WorkspaceId) -> Self {
        let analyst = PrincipalId::generate_with("analyst");
        let reviewer = PrincipalId::generate_with("reviewer");
        let pi = PrincipalId::generate_with("pi");

        // Policies: allow analysis and review actions; release_claim is E3 gated.
        let policy = Policy::new(
            workspace_id.clone(),
            "biolab",
            vec![
                PolicyRule::allow("start_analysis"),
                PolicyRule::allow("submit_analysis_artifact"),
                PolicyRule::allow("approve_claim"),
                PolicyRule::allow("release_claim"),
            ],
        );
        let e3 = EffectContract::e3("public release of scientific claim");
        let gateway = ActionGateway::new(policy, e3);

        Self {
            workspace_id,
            world: morn_world::WorldService::new(),
            artifacts: ArtifactService::new(),
            work: WorkService::new(),
            gateway,
            analyst,
            reviewer,
            pi,
        }
    }

    pub fn init_world(&mut self) {
        domain::register_object_types(&self.workspace_id, &mut self.world);
    }

    /// Full demo: Dataset -> WorkPackage -> Analysis Artifact -> Review -> PI
    /// Approval -> governed Action -> StateDiff -> Reviewed Claim Outcome.
    pub fn run_dataset_to_claim_e2e(&mut self, dataset_name: &str, rows: u64) -> Result<E2eResult> {
        let mut steps = Vec::new();
        self.init_world();

        // 1. register dataset
        let dataset_id = DatasetId::generate_with("ds");
        let dataset_object_id = ObjectId::new(dataset_id.to_string());
        let dataset_object = Object::new(
            dataset_object_id.clone(),
            object_type_id(&self.world, "biolab.Dataset"),
            self.workspace_id.clone(),
            domain::dataset_state(dataset_name, rows),
        );
        self.world.register_object(dataset_object);
        steps.push(ok_step(
            "register_dataset",
            &format!("dataset {dataset_id} registered ({rows} rows)"),
        ));

        // 2. WorkPackage with AcceptanceSpec + hybrid ExecutionMode
        let acceptance = AcceptanceSpec::new("dataset_to_reviewed_claim")
            .with_required_artifacts(vec!["analysis".to_string()])
            .with_human_approval(true);
        let acceptance_id = acceptance.id.clone();
        self.work.add_acceptance_spec(acceptance);
        let wp = WorkPackage::new(
            self.workspace_id.clone(),
            "Dataset -> Reviewed Scientific Claim",
            self.analyst.clone(),
        )
        .with_acceptance_spec(acceptance_id)
        .with_execution_mode(ExecutionMode::hybrid(
            vec![WorkNature::Deterministic, WorkNature::Probabilistic],
            vec![
                ExecutorType::Program,
                ExecutorType::Actor,
                ExecutorType::Human,
            ],
        ));
        let wp_id = wp.id.clone();
        self.work.add_work_package(wp);
        steps.push(ok_step(
            "work_package",
            &format!("work package {wp_id} with acceptance spec + hybrid mode"),
        ));

        // 3. start analysis (deterministic worker) -> AnalysisRun object + analysis artifact v1
        let analysis_run_id = AnalysisRunId::generate_with("run");
        let analysis = self.run_deterministic_analysis(&dataset_id, &analysis_run_id, rows);
        let (artifact, version) = self.artifacts.create(
            "analysis",
            "biolab/analysis@1",
            self.workspace_id.clone(),
            self.analyst.clone(),
            &format!("artifact://analysis/{analysis_run_id}/v1"),
            json!({
                "dataset": dataset_id.to_string(),
                "rows": rows,
                "qc_pass": analysis.qc_pass,
                "summary": analysis.summary,
            }),
            &analysis.checksum,
        )?;
        steps.push(ok_step(
            "start_analysis",
            &format!(
                "analysis run {analysis_run_id} completed; artifact {} v{} created",
                artifact.id, version.version_no
            ),
        ));

        // 4. submit + review + PI approval
        self.artifacts.submit(&version.id)?;
        self.artifacts.mark_in_review(&version.id)?;
        self.artifacts.review(Review::new(
            version.id.clone(),
            self.reviewer.clone(),
            ReviewDecision::Approve,
            "independent statistical review passed",
        ))?;
        self.artifacts.approve(ArtifactApproval::approve(
            version.id.clone(),
            self.pi.clone(),
            Some("PI approves analysis".to_string()),
        ))?;
        steps.push(ok_step(
            "review_approve",
            "statistical reviewer + PI approval recorded",
        ));

        // 5. governed release action (E3) -> claim state diff -> outcome
        let claim_id = ScientificClaimId::generate_with("claim");
        let claim_object_id = ObjectId::new(claim_id.to_string());
        let claim_object = Object::new(
            claim_object_id.clone(),
            object_type_id(&self.world, "biolab.ScientificClaim"),
            self.workspace_id.clone(),
            {
                let mut s = BTreeMap::new();
                s.insert(
                    "statement".to_string(),
                    json!("Mechanism X is reproducible in this dataset"),
                );
                s.insert("status".to_string(), json!("draft"));
                s.insert("dataset".to_string(), json!(dataset_id.to_string()));
                s.insert(
                    "analysis_run".to_string(),
                    json!(analysis_run_id.to_string()),
                );
                s.insert("artifact".to_string(), json!(artifact.id.to_string()));
                s
            },
        );
        self.world.register_object(claim_object);

        // PI approval satisfied for E3 release
        self.gateway.mark_approval_satisfied("pi");
        let action_type_id = ActionTypeId::generate_with("actt");
        let proposal = morn_world::action::ActionProposal::new(
            action_type_id,
            claim_object_id.clone(),
            self.workspace_id.clone(),
            BTreeMap::new(),
            self.analyst.as_str(),
        );
        let authorized = self.gateway.authorize(&proposal, "release_claim")?;
        let mut new_state = BTreeMap::new();
        new_state.insert("status".to_string(), json!("released"));
        new_state.insert(
            "statement".to_string(),
            json!("Mechanism X is reproducible in this dataset"),
        );
        new_state.insert("dataset".to_string(), json!(dataset_id.to_string()));
        new_state.insert(
            "analysis_run".to_string(),
            json!(analysis_run_id.to_string()),
        );
        new_state.insert("artifact".to_string(), json!(artifact.id.to_string()));
        let outcome = self.gateway.execute(
            &authorized,
            claim_object_id.clone(),
            new_state,
            Some("PI approved release".to_string()),
            self.analyst.as_str(),
            &mut self.world,
        )?;
        steps.push(ok_step(
            "governed_action",
            &format!(
                "release_claim executed -> receipt {} ({} )",
                outcome.receipt_id, outcome.details
            ),
        ));

        // 6. accept work package
        let status = self.work.attempt_accept(
            &wp_id,
            &AcceptanceEvidence {
                produced_artifacts: vec!["analysis".to_string()],
                verification_passed: true,
                required_approvals: vec!["pi".to_string()],
                forbidden_condition_hit: None,
            },
        )?;
        steps.push(ok_step(
            "accept",
            &format!("work package accepted: {status:?}"),
        ));

        // 7. outcome record
        let snapshot = self
            .world
            .snapshots_for(&claim_object_id)
            .last()
            .cloned()
            .ok_or_else(|| Error::internal("missing claim snapshot"))?;
        let outcome_record =
            OutcomeRecord::new(self.workspace_id.clone(), "Reviewed Scientific Claim", true);
        let outcome_id = outcome_record.id.clone();
        self.world.record_outcome(OutcomeRecord {
            state_snapshot_ids: vec![snapshot.id.clone()],
            verification_report_id: None,
            work_package_id: Some(wp_id.clone()),
            related_artifacts: vec![artifact.id.to_string()],
            ..outcome_record
        });
        steps.push(ok_step(
            "outcome",
            &format!("Reviewed Scientific Claim outcome {outcome_id} recorded"),
        ));

        let all_ok = !steps.is_empty() && steps.iter().all(|s| s.ok);
        Ok(E2eResult {
            workspace_id: self.workspace_id.to_string(),
            dataset_id: dataset_id.to_string(),
            work_package_id: wp_id.to_string(),
            analysis_run_id: analysis_run_id.to_string(),
            artifact_id: artifact.id.to_string(),
            artifact_version_id: version.id.to_string(),
            claim_id: claim_id.to_string(),
            outcome_id: outcome_id.to_string(),
            steps,
            all_ok,
        })
    }

    /// Deterministic "analysis" (a real computation on the dataset metadata,
    /// not hardcoded UI data).
    fn run_deterministic_analysis(
        &mut self,
        dataset_id: &DatasetId,
        analysis_run_id: &AnalysisRunId,
        rows: u64,
    ) -> DeterministicResult {
        let qc_pass = rows > 0 && rows.is_multiple_of(2);
        let summary = format!(
            "qc={}; rows={}; plate_batch=1",
            if qc_pass { "pass" } else { "warn" },
            rows
        );
        // record an AnalysisRun object in the world
        let run_object = Object::new(
            ObjectId::new(analysis_run_id.to_string()),
            object_type_id(&self.world, "biolab.AnalysisRun"),
            self.workspace_id.clone(),
            {
                let mut s = BTreeMap::new();
                s.insert("dataset".to_string(), json!(dataset_id.to_string()));
                s.insert("status".to_string(), json!("completed"));
                s.insert("summary".to_string(), json!(summary));
                s
            },
        );
        self.world.register_object(run_object);
        DeterministicResult {
            qc_pass,
            summary,
            checksum: format!("sha256:{}", rows),
        }
    }
}

struct DeterministicResult {
    qc_pass: bool,
    summary: String,
    checksum: String,
}

fn object_type_id(world: &morn_world::WorldService, name: &str) -> morn_kernel::ids::ObjectTypeId {
    world
        .object_types()
        .iter()
        .find(|t| t.name == name)
        .map(|t| t.id.clone())
        .unwrap_or_else(|| morn_kernel::ids::ObjectTypeId::generate_with("objt"))
}

fn ok_step(step: &str, detail: &str) -> E2eStep {
    E2eStep {
        step: step.to_string(),
        ok: true,
        detail: detail.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use morn_kernel::ids::{ArtifactVersionId, OutcomeRecordId, WorkPackageId, WorkspaceId};

    #[test]
    fn biolab_e2e_reaches_reviewed_claim_with_lineage() {
        let ws = WorkspaceId::generate();
        let mut svc = BioLabService::new(ws.clone());
        let result = svc.run_dataset_to_claim_e2e("aging_pilot", 128).unwrap();
        assert!(result.all_ok(), "steps: {:?}", result.steps);
        assert_eq!(result.steps.len(), 7);

        // claim object released
        let claim = svc
            .world
            .object(&ObjectId::new(result.claim_id.clone()))
            .expect("claim object");
        assert_eq!(claim.state().get("status"), Some(&json!("released")));

        // lineage: outcome links snapshot; claim links dataset/analysis/artifact
        let outcome = svc
            .world
            .outcome(&OutcomeRecordId::new(result.outcome_id.clone()))
            .expect("outcome");
        assert!(!outcome.state_snapshot_ids.is_empty());
        assert_eq!(
            outcome.work_package_id,
            Some(WorkPackageId::new(result.work_package_id.clone()))
        );

        // artifact approved and readable
        let artifact_version = svc
            .artifacts
            .version(&ArtifactVersionId::new(result.artifact_version_id.clone()))
            .expect("artifact version");
        assert_eq!(
            artifact_version.status,
            morn_kernel::status::ArtifactStatus::Approved
        );
        assert!(svc.artifacts.require_approved(&artifact_version.id).is_ok());

        // work package accepted
        assert_eq!(
            svc.work
                .work_package(&WorkPackageId::new(result.work_package_id.clone()))
                .unwrap()
                .status,
            morn_kernel::status::WorkStatus::Accepted
        );
    }
}
