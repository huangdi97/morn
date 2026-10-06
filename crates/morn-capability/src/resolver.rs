//! Deterministic capability resolution against explicit requirements.

use serde::{Deserialize, Serialize};

use crate::manifest::{CapabilityManifestId, CapabilityRecord, CapabilityStage, IsolationLevel};
use crate::registry::CapabilityKind;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct CapabilityRequest {
    pub required_provides: Vec<String>,
    pub allowed_kinds: Vec<CapabilityKind>,
    pub minimum_isolation: Option<IsolationLevel>,
    pub required_authority: Vec<String>,
    pub max_estimated_cost_micros: Option<u64>,
    pub site_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolvedCapability {
    pub manifest_id: CapabilityManifestId,
    pub provider_ref: String,
    pub score: u32,
    pub rationale: Vec<String>,
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
            if !request.allowed_kinds.is_empty()
                && !request
                    .allowed_kinds
                    .iter()
                    .any(|kind| *kind == manifest.kind)
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
            if let Some(minimum) = request.minimum_isolation {
                if manifest.execution.minimum_isolation < minimum {
                    continue;
                }
            }
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::EffectClass;
    use crate::manifest::{CapabilityManifest, CapabilityStage};
    use morn_kernel::ids::CapabilityId;

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
