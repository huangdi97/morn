//! Deterministic capability resolution against explicit requirements.

use serde::{Deserialize, Serialize};

use morn_kernel::ExecutionGuarantee;

use crate::manifest::{CapabilityManifestId, CapabilityRecord, CapabilityStage, IsolationLevel};
use crate::registry::CapabilityKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CapabilityRequest {
    pub required_provides: Vec<String>,
    pub allowed_kinds: Vec<CapabilityKind>,
    pub minimum_isolation: Option<IsolationLevel>,
    pub required_authority: Vec<String>,
    pub max_estimated_cost_micros: Option<u64>,
    pub max_latency_p95_ms: Option<u64>,
    pub maximum_effect: Option<crate::effect::EffectClass>,
    pub required_runtime_kinds: Vec<String>,
    pub required_harness_compatibility: Vec<String>,
    pub required_execution_guarantees: Vec<ExecutionGuarantee>,
    pub site_ref: Option<String>,
    pub profile_ref: Option<String>,
    pub unavailable_providers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedCapability {
    pub manifest_id: CapabilityManifestId,
    pub provider_ref: String,
    pub kind: CapabilityKind,
    pub estimated_cost_micros: Option<u64>,
    /// Effective execution floor after merging Work/Profile and capability
    /// requirements. This is a requirement for environment selection, not
    /// evidence that the capability provider already satisfies it.
    pub required_execution_class: IsolationLevel,
    pub required_execution_guarantees: Vec<ExecutionGuarantee>,
    pub score: u32,
    pub rationale: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorkcellRequest {
    pub required_provides: Vec<String>,
    pub allowed_kinds: Vec<CapabilityKind>,
    pub minimum_isolation: Option<IsolationLevel>,
    pub required_authority: Vec<String>,
    pub max_total_cost_micros: Option<u64>,
    pub max_latency_p95_ms: Option<u64>,
    pub maximum_effect: Option<crate::effect::EffectClass>,
    pub required_runtime_kinds: Vec<String>,
    pub required_harness_compatibility: Vec<String>,
    pub required_execution_guarantees: Vec<ExecutionGuarantee>,
    pub site_ref: Option<String>,
    pub profile_ref: Option<String>,
    pub unavailable_providers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkcellMember {
    pub capability: ResolvedCapability,
    pub covers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkcellPlan {
    pub members: Vec<WorkcellMember>,
    pub uncovered: Vec<String>,
    pub total_estimated_cost_micros: u64,
    pub rationale: Vec<String>,
}

impl WorkcellPlan {
    pub fn is_complete(&self) -> bool {
        self.uncovered.is_empty()
    }

    pub fn agent_count(&self) -> usize {
        self.members
            .iter()
            .filter(|member| matches!(member.capability.kind, CapabilityKind::Agent))
            .count()
    }
}

fn merge_execution_class(
    capability: IsolationLevel,
    requested: Option<IsolationLevel>,
) -> Option<IsolationLevel> {
    let Some(requested) = requested else {
        return Some(capability);
    };
    if capability.satisfies(requested) {
        return Some(capability);
    }
    if requested.satisfies(capability) {
        return Some(requested);
    }
    // Remote/physical versus local containment are intentionally incomparable
    // until a concrete environment provider supplies richer attested semantics.
    None
}

fn merge_execution_guarantees(
    capability: &[ExecutionGuarantee],
    requested: &[ExecutionGuarantee],
) -> Vec<ExecutionGuarantee> {
    let mut merged = capability.to_vec();
    for guarantee in requested {
        if !merged.contains(guarantee) {
            merged.push(*guarantee);
        }
    }
    merged.sort();
    merged
}

fn effect_rank(effect: crate::effect::EffectClass) -> u8 {
    use crate::effect::EffectClass::*;
    match effect {
        E0LifecycleReversible => 0,
        E1Transactional => 1,
        E2Compensatable => 2,
        E3Irreversible => 3,
    }
}

#[derive(Debug, Default)]
pub struct CapabilityResolver;

impl CapabilityResolver {
    pub fn resolve(
        &self,
        request: &CapabilityRequest,
        candidates: &[CapabilityRecord],
    ) -> Vec<ResolvedCapability> {
        let mut matches = Vec::new();
        for candidate in candidates {
            let manifest = &candidate.manifest;

            if !matches!(
                candidate.stage,
                CapabilityStage::Qualified | CapabilityStage::Admitted
            ) {
                continue;
            }
            if let Some(site) = &request.site_ref {
                if candidate.stage != CapabilityStage::Admitted
                    || !candidate.admitted_sites.iter().any(|value| value == site)
                {
                    continue;
                }
            }
            if let Some(profile) = &request.profile_ref {
                let site = request.site_ref.as_deref();
                if candidate.stage != CapabilityStage::Admitted
                    || !candidate.admission_refs.iter().any(|admission| {
                        admission.profile_ref == *profile
                            && site.is_none_or(|expected_site| admission.site_ref == expected_site)
                    })
                {
                    continue;
                }
            }
            if request
                .unavailable_providers
                .iter()
                .any(|provider| provider == &manifest.provider_ref)
            {
                continue;
            }
            if !request.allowed_kinds.is_empty() && !request.allowed_kinds.contains(&manifest.kind)
            {
                continue;
            }
            if !request
                .required_provides
                .iter()
                .all(|needed| manifest.provides.iter().any(|provided| provided == needed))
            {
                continue;
            }
            let Some(required_execution_class) = merge_execution_class(
                manifest.execution.minimum_isolation,
                request.minimum_isolation,
            ) else {
                continue;
            };
            let required_execution_guarantees = merge_execution_guarantees(
                &manifest.execution.required_guarantees,
                &request.required_execution_guarantees,
            );
            if !request.required_authority.iter().all(|needed| {
                manifest
                    .authority
                    .allow
                    .iter()
                    .any(|allowed| allowed == needed)
                    && !manifest
                        .authority
                        .deny
                        .iter()
                        .any(|denied| denied == needed)
            }) {
                continue;
            }
            if let (Some(maximum), Some(cost)) = (
                request.max_estimated_cost_micros,
                manifest.economics.estimated_cost_micros,
            ) {
                if cost > maximum {
                    continue;
                }
            }
            if let Some(maximum) = request.max_latency_p95_ms {
                match manifest.economics.latency_p95_ms {
                    Some(latency) if latency <= maximum => {}
                    _ => continue,
                }
            }
            if let Some(maximum_effect) = request.maximum_effect {
                if effect_rank(manifest.authority.maximum_effect) > effect_rank(maximum_effect) {
                    continue;
                }
            }
            if !request.required_runtime_kinds.iter().all(|required| {
                manifest
                    .execution
                    .runtime_kinds
                    .iter()
                    .any(|runtime| runtime == required)
            }) {
                continue;
            }
            if !request
                .required_harness_compatibility
                .iter()
                .all(|required| {
                    manifest
                        .execution
                        .harness_compatibility
                        .iter()
                        .any(|harness| harness == required)
                })
            {
                continue;
            }
            let mut rationale = vec!["semantic requirements satisfied".to_string()];
            let mut score = 50;
            if candidate.stage == CapabilityStage::Admitted {
                score += 40;
                rationale.push("site-admitted".to_string());
            }
            if !candidate.qualification_refs.is_empty() {
                score += 10;
                rationale.push("qualification evidence present".to_string());
            }
            matches.push(ResolvedCapability {
                manifest_id: manifest.id.clone(),
                provider_ref: manifest.provider_ref.clone(),
                kind: manifest.kind,
                estimated_cost_micros: manifest.economics.estimated_cost_micros,
                required_execution_class,
                required_execution_guarantees,
                score,
                rationale,
            });
        }

        matches.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| left.provider_ref.cmp(&right.provider_ref))
        });
        matches
    }

    /// Build a minimum-sufficient executor mix. Hard constraints are filtered
    /// first; then a deterministic greedy set-cover picks the candidate that
    /// closes the most still-uncovered capabilities. Ties prefer higher trust
    /// score, lower known cost, then provider name. This deliberately permits a
    /// complete zero-agent workcell when rules/programs/solvers/humans suffice.
    pub fn resolve_minimum_workcell(
        &self,
        request: &WorkcellRequest,
        candidates: &[CapabilityRecord],
    ) -> WorkcellPlan {
        let mut uncovered = request.required_provides.clone();
        uncovered.sort();
        uncovered.dedup();

        let base = CapabilityRequest {
            required_provides: Vec::new(),
            allowed_kinds: request.allowed_kinds.clone(),
            minimum_isolation: request.minimum_isolation,
            required_authority: request.required_authority.clone(),
            max_estimated_cost_micros: None,
            max_latency_p95_ms: request.max_latency_p95_ms,
            maximum_effect: request.maximum_effect,
            required_runtime_kinds: request.required_runtime_kinds.clone(),
            required_harness_compatibility: request.required_harness_compatibility.clone(),
            required_execution_guarantees: request.required_execution_guarantees.clone(),
            site_ref: request.site_ref.clone(),
            profile_ref: request.profile_ref.clone(),
            unavailable_providers: request.unavailable_providers.clone(),
        };

        let eligible = self.resolve(&base, candidates);
        let mut selected: Vec<WorkcellMember> = Vec::new();
        let mut used: Vec<CapabilityManifestId> = Vec::new();
        let mut total_cost = 0_u64;

        loop {
            if uncovered.is_empty() {
                break;
            }

            let mut best: Option<(ResolvedCapability, Vec<String>)> = None;
            for resolved in &eligible {
                if used.iter().any(|id| id == &resolved.manifest_id) {
                    continue;
                }
                let Some(record) = candidates
                    .iter()
                    .find(|candidate| candidate.manifest.id == resolved.manifest_id)
                else {
                    continue;
                };
                let mut covers: Vec<String> = uncovered
                    .iter()
                    .filter(|needed| record.manifest.provides.iter().any(|p| p == *needed))
                    .cloned()
                    .collect();
                covers.sort();
                covers.dedup();
                if covers.is_empty() {
                    continue;
                }

                let candidate_cost = match (
                    request.max_total_cost_micros,
                    resolved.estimated_cost_micros,
                ) {
                    (Some(_), None) => continue,
                    (_, Some(cost)) => cost,
                    (None, None) => 0,
                };
                if request
                    .max_total_cost_micros
                    .is_some_and(|max| total_cost.saturating_add(candidate_cost) > max)
                {
                    continue;
                }

                let should_replace = best.as_ref().is_none_or(|(current, current_covers)| {
                    covers.len() > current_covers.len()
                        || (covers.len() == current_covers.len()
                            && (resolved.score > current.score
                                || (resolved.score == current.score
                                    && (resolved.estimated_cost_micros.unwrap_or(u64::MAX)
                                        < current.estimated_cost_micros.unwrap_or(u64::MAX)
                                        || (resolved.estimated_cost_micros
                                            == current.estimated_cost_micros
                                            && resolved.provider_ref < current.provider_ref)))))
                });
                if should_replace {
                    best = Some((resolved.clone(), covers));
                }
            }

            let Some((capability, covers)) = best else {
                break;
            };
            total_cost = total_cost.saturating_add(capability.estimated_cost_micros.unwrap_or(0));
            uncovered.retain(|needed| !covers.iter().any(|covered| covered == needed));
            used.push(capability.manifest_id.clone());
            selected.push(WorkcellMember { capability, covers });
        }

        let mut rationale = vec![format!(
            "selected {} executor(s) using hard-filter + minimum-cover planning",
            selected.len()
        )];
        if selected
            .iter()
            .all(|member| !matches!(member.capability.kind, CapabilityKind::Agent))
        {
            rationale.push("zero-agent plan: no model/LLM executor required".to_string());
        }
        if !uncovered.is_empty() {
            rationale.push(format!("uncovered requirements: {}", uncovered.join(", ")));
        }

        WorkcellPlan {
            members: selected,
            uncovered,
            total_estimated_cost_micros: total_cost,
            rationale,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::EffectClass;
    use crate::manifest::{CapabilityManifest, CapabilityStage};
    use morn_kernel::ids::CapabilityId;

    fn admitted(
        name: &str,
        provider: &str,
        kind: CapabilityKind,
        provides: &[&str],
        cost: u64,
    ) -> CapabilityRecord {
        let mut manifest = CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            name,
            provider,
            kind,
            EffectClass::E0LifecycleReversible,
        );
        manifest.provides = provides.iter().map(|value| value.to_string()).collect();
        manifest.economics.estimated_cost_micros = Some(cost);
        manifest.execution.minimum_isolation = IsolationLevel::Container;
        let mut record = CapabilityRecord::new(manifest);
        record.stage = CapabilityStage::Admitted;
        record.qualification_refs.push(format!("qual:{name}"));
        record.admitted_sites.push("plant-a".to_string());
        record
    }

    #[test]
    fn minimum_workcell_can_be_zero_agent() {
        let candidates = vec![
            admitted(
                "event-rule",
                "rule-provider",
                CapabilityKind::Rule,
                &["alarm.classify"],
                1,
            ),
            admitted(
                "capacity-solver",
                "solver-provider",
                CapabilityKind::Solver,
                &["capacity.optimize"],
                20,
            ),
            admitted(
                "production-owner",
                "human-provider",
                CapabilityKind::Human,
                &["decision.accept"],
                100,
            ),
            admitted(
                "general-agent",
                "dsh",
                CapabilityKind::Agent,
                &["alarm.classify"],
                50,
            ),
        ];
        let plan = CapabilityResolver.resolve_minimum_workcell(
            &WorkcellRequest {
                required_provides: vec![
                    "alarm.classify".into(),
                    "capacity.optimize".into(),
                    "decision.accept".into(),
                ],
                minimum_isolation: Some(IsolationLevel::Container),
                site_ref: Some("plant-a".into()),
                ..Default::default()
            },
            &candidates,
        );
        assert!(plan.is_complete());
        assert_eq!(plan.members.len(), 3);
        assert_eq!(plan.agent_count(), 0);
    }

    #[test]
    fn workcell_fails_closed_on_budget_or_unavailable_provider() {
        let candidates = vec![
            admitted(
                "cheap-rule",
                "rule-provider",
                CapabilityKind::Rule,
                &["observe"],
                10,
            ),
            admitted(
                "solver",
                "solver-provider",
                CapabilityKind::Solver,
                &["optimize"],
                90,
            ),
        ];
        let plan = CapabilityResolver.resolve_minimum_workcell(
            &WorkcellRequest {
                required_provides: vec!["observe".into(), "optimize".into()],
                max_total_cost_micros: Some(50),
                site_ref: Some("plant-a".into()),
                ..Default::default()
            },
            &candidates,
        );
        assert!(!plan.is_complete());
        assert_eq!(plan.uncovered, vec!["optimize".to_string()]);

        let unavailable = CapabilityResolver.resolve_minimum_workcell(
            &WorkcellRequest {
                required_provides: vec!["observe".into()],
                site_ref: Some("plant-a".into()),
                unavailable_providers: vec!["rule-provider".into()],
                ..Default::default()
            },
            &candidates,
        );
        assert!(!unavailable.is_complete());
    }

    #[test]
    fn profile_execution_requirements_are_merged_not_mistaken_for_capability_evidence() {
        let mut record = admitted(
            "reader",
            "program-provider",
            CapabilityKind::Program,
            &["read"],
            1,
        );
        record.manifest.execution.minimum_isolation = IsolationLevel::Process;
        record.manifest.execution.required_guarantees =
            vec![ExecutionGuarantee::FilesystemReadPolicy];

        let resolved = CapabilityResolver.resolve(
            &CapabilityRequest {
                required_provides: vec!["read".to_string()],
                minimum_isolation: Some(IsolationLevel::Container),
                required_execution_guarantees: vec![
                    ExecutionGuarantee::NetworkEgressPolicy,
                    ExecutionGuarantee::SecretIndirection,
                ],
                site_ref: Some("plant-a".to_string()),
                ..Default::default()
            },
            &[record],
        );
        assert_eq!(resolved.len(), 1);
        assert_eq!(
            resolved[0].required_execution_class,
            IsolationLevel::Container
        );
        assert!(resolved[0]
            .required_execution_guarantees
            .contains(&ExecutionGuarantee::FilesystemReadPolicy));
        assert!(resolved[0]
            .required_execution_guarantees
            .contains(&ExecutionGuarantee::NetworkEgressPolicy));
        assert!(resolved[0]
            .required_execution_guarantees
            .contains(&ExecutionGuarantee::SecretIndirection));
    }

    #[test]
    fn incompatible_execution_topologies_fail_closed_during_resolution() {
        let mut remote = admitted(
            "remote-only",
            "remote-provider",
            CapabilityKind::Service,
            &["execute"],
            1,
        );
        remote.manifest.execution.minimum_isolation = IsolationLevel::Remote;

        let blocked = CapabilityResolver.resolve(
            &CapabilityRequest {
                required_provides: vec!["execute".to_string()],
                minimum_isolation: Some(IsolationLevel::Container),
                site_ref: Some("plant-a".to_string()),
                ..Default::default()
            },
            &[remote],
        );
        assert!(blocked.is_empty());
    }

    #[test]
    fn remote_is_not_treated_as_stronger_than_microvm() {
        let mut remote = admitted(
            "remote-service",
            "remote-provider",
            CapabilityKind::Service,
            &["execute"],
            10,
        );
        remote.manifest.execution.minimum_isolation = IsolationLevel::Remote;

        let blocked = CapabilityResolver.resolve(
            &CapabilityRequest {
                required_provides: vec!["execute".to_string()],
                minimum_isolation: Some(IsolationLevel::MicroVm),
                site_ref: Some("plant-a".to_string()),
                ..Default::default()
            },
            &[remote.clone()],
        );
        assert!(blocked.is_empty());

        let explicit_remote = CapabilityResolver.resolve(
            &CapabilityRequest {
                required_provides: vec!["execute".to_string()],
                minimum_isolation: Some(IsolationLevel::Remote),
                site_ref: Some("plant-a".to_string()),
                ..Default::default()
            },
            &[remote],
        );
        assert_eq!(explicit_remote.len(), 1);
    }

    #[test]
    fn resolver_merges_execution_requirements_for_environment_selection() {
        let mut record = admitted(
            "sandboxed-program",
            "program-provider",
            CapabilityKind::Program,
            &["observe"],
            10,
        );
        record.manifest.execution.required_guarantees = vec![
            ExecutionGuarantee::FilesystemWritePolicy,
            ExecutionGuarantee::NetworkEgressPolicy,
        ];

        let resolved = CapabilityResolver.resolve(
            &CapabilityRequest {
                required_provides: vec!["observe".to_string()],
                required_execution_guarantees: vec![ExecutionGuarantee::RuntimeAttestation],
                site_ref: Some("plant-a".to_string()),
                ..Default::default()
            },
            &[record],
        );
        assert_eq!(resolved.len(), 1);
        assert!(resolved[0]
            .required_execution_guarantees
            .contains(&ExecutionGuarantee::FilesystemWritePolicy));
        assert!(resolved[0]
            .required_execution_guarantees
            .contains(&ExecutionGuarantee::NetworkEgressPolicy));
        assert!(resolved[0]
            .required_execution_guarantees
            .contains(&ExecutionGuarantee::RuntimeAttestation));
    }

    #[test]
    fn resolver_enforces_effect_latency_runtime_and_harness_constraints() {
        let mut record = admitted(
            "safe-program",
            "program-provider",
            CapabilityKind::Program,
            &["observe"],
            10,
        );
        record.manifest.economics.latency_p95_ms = Some(20);
        record.manifest.execution.runtime_kinds = vec!["rust".to_string()];
        record.manifest.execution.harness_compatibility = vec!["none".to_string()];
        record.manifest.authority.maximum_effect =
            crate::effect::EffectClass::E0LifecycleReversible;

        let ok = CapabilityResolver.resolve(
            &CapabilityRequest {
                required_provides: vec!["observe".to_string()],
                max_latency_p95_ms: Some(50),
                maximum_effect: Some(crate::effect::EffectClass::E0LifecycleReversible),
                required_runtime_kinds: vec!["rust".to_string()],
                required_harness_compatibility: vec!["none".to_string()],
                site_ref: Some("plant-a".to_string()),
                ..Default::default()
            },
            &[record.clone()],
        );
        assert_eq!(ok.len(), 1);

        let blocked = CapabilityResolver.resolve(
            &CapabilityRequest {
                required_provides: vec!["observe".to_string()],
                max_latency_p95_ms: Some(5),
                site_ref: Some("plant-a".to_string()),
                ..Default::default()
            },
            &[record],
        );
        assert!(blocked.is_empty());
    }

    #[test]
    fn resolver_rejects_site_admission_from_wrong_profile() {
        let mut record = admitted(
            "investigator",
            "provider",
            CapabilityKind::Program,
            &["investigate"],
            10,
        );
        record
            .admission_refs
            .push(crate::manifest::CapabilityAdmissionRef {
                admission_ref: "admission:1".to_string(),
                site_ref: "plant-a".to_string(),
                profile_ref: "morn.factory.readonly@1.0.0".to_string(),
            });

        let accepted = CapabilityResolver.resolve(
            &CapabilityRequest {
                required_provides: vec!["investigate".to_string()],
                site_ref: Some("plant-a".to_string()),
                profile_ref: Some("morn.factory.readonly@1.0.0".to_string()),
                ..Default::default()
            },
            &[record.clone()],
        );
        assert_eq!(accepted.len(), 1);

        let rejected = CapabilityResolver.resolve(
            &CapabilityRequest {
                required_provides: vec!["investigate".to_string()],
                site_ref: Some("plant-a".to_string()),
                profile_ref: Some("morn.factory.readonly@2.0.0".to_string()),
                ..Default::default()
            },
            &[record],
        );
        assert!(rejected.is_empty());
    }

    #[test]
    fn resolver_requires_qualification_and_site_admission() {
        let mut manifest = CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            "investigator",
            "dsh",
            CapabilityKind::Llm,
            EffectClass::E0LifecycleReversible,
        );
        manifest.provides = vec!["equipment.anomaly.investigate".into()];
        manifest.authority.allow = vec!["historian.read".into()];
        manifest.execution.minimum_isolation = IsolationLevel::Container;

        let mut record = CapabilityRecord::new(manifest);
        record.stage = CapabilityStage::Qualified;
        record.qualification_refs = vec!["qual-1".into()];

        let request = CapabilityRequest {
            required_provides: vec!["equipment.anomaly.investigate".into()],
            allowed_kinds: vec![CapabilityKind::Llm],
            minimum_isolation: Some(IsolationLevel::Container),
            required_authority: vec!["historian.read".into()],
            site_ref: Some("plant-a".into()),
            ..Default::default()
        };

        let resolver = CapabilityResolver;
        assert!(resolver.resolve(&request, &[record.clone()]).is_empty());

        record.stage = CapabilityStage::Admitted;
        record.admitted_sites.push("plant-a".into());
        assert_eq!(resolver.resolve(&request, &[record]).len(), 1);
    }
}
