//! Standards-oriented discovery projections.
//!
//! These adapters export declared metadata from the canonical Morn
//! CapabilityManifest. They intentionally omit qualification, release, site
//! admission, authority and accepted-outcome truth. Importing external
//! discovery metadata likewise creates metadata only; it never creates or
//! mutates a CapabilityRecord.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};

use crate::{CapabilityKind, CapabilityManifest, CapabilityRecord};

pub const DISCOVERY_PROJECTION_VERSION: &str = "morn.discovery-projection/v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiscoveryProvenance {
    pub adapter_version: String,
    pub canonical_manifest_ref: String,
    pub canonical_manifest_digest: Option<String>,
    pub metadata_class: String,
}

impl DiscoveryProvenance {
    fn from_manifest(manifest: &CapabilityManifest) -> Self {
        Self {
            adapter_version: DISCOVERY_PROJECTION_VERSION.to_string(),
            canonical_manifest_ref: manifest.id.to_string(),
            canonical_manifest_digest: manifest.digest.clone(),
            metadata_class: "declared".to_string(),
        }
    }

    pub fn is_declared_only(&self) -> bool {
        self.metadata_class == "declared"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct XRegistryResourceProjection {
    pub group: String,
    pub resource_id: String,
    pub version: String,
    pub name: String,
    pub capability_kind: String,
    pub provider_ref: String,
    pub provides: Vec<String>,
    pub provenance: DiscoveryProvenance,
}

pub fn project_xregistry(record: &CapabilityRecord) -> Result<XRegistryResourceProjection> {
    record.manifest.validate_governance()?;
    Ok(XRegistryResourceProjection {
        group: "morn-capabilities".to_string(),
        resource_id: record.manifest.definition_id.to_string(),
        version: record.manifest.version.to_string(),
        name: record.manifest.name.clone(),
        capability_kind: record.manifest.kind.as_str().to_string(),
        provider_ref: record.manifest.provider_ref.clone(),
        provides: record.manifest.provides.clone(),
        provenance: DiscoveryProvenance::from_manifest(&record.manifest),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct A2aSkillProjection {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct A2aAgentCardProjection {
    pub name: String,
    pub card_url: String,
    pub protocol_version: String,
    pub skills: Vec<A2aSkillProjection>,
    pub provenance: DiscoveryProvenance,
}

fn validate_major_minor(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    parts.len() == 2
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.chars().all(|ch| ch.is_ascii_digit()))
}

pub fn project_a2a_agent_card(
    record: &CapabilityRecord,
    card_url: impl Into<String>,
    protocol_version: impl Into<String>,
) -> Result<A2aAgentCardProjection> {
    record.manifest.validate_governance()?;
    if record.manifest.kind != CapabilityKind::Agent {
        return Err(Error::validation(
            "A2A Agent Card projection is only valid for Agent-kind capabilities",
        ));
    }
    let card_url = card_url.into();
    let protocol_version = protocol_version.into();
    if card_url.trim().is_empty() || !validate_major_minor(&protocol_version) {
        return Err(Error::validation(
            "A2A projection requires a card URL and Major.Minor protocol version",
        ));
    }
    Ok(A2aAgentCardProjection {
        name: record.manifest.name.clone(),
        card_url,
        protocol_version,
        skills: record
            .manifest
            .provides
            .iter()
            .map(|skill| A2aSkillProjection {
                id: skill.clone(),
                name: skill.clone(),
            })
            .collect(),
        provenance: DiscoveryProvenance::from_manifest(&record.manifest),
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OasfDiscoveryProjection {
    pub name: String,
    pub taxonomy_version: String,
    pub skills: Vec<String>,
    pub domains: Vec<String>,
    pub provenance: DiscoveryProvenance,
}

pub fn project_oasf(
    record: &CapabilityRecord,
    taxonomy_version: impl Into<String>,
    domains: Vec<String>,
) -> Result<OasfDiscoveryProjection> {
    record.manifest.validate_governance()?;
    if record.manifest.kind != CapabilityKind::Agent {
        return Err(Error::validation(
            "OASF agentic discovery projection is only valid for Agent-kind capabilities",
        ));
    }
    let taxonomy_version = taxonomy_version.into();
    if taxonomy_version.trim().is_empty()
        || domains.iter().any(|domain| domain.trim().is_empty())
    {
        return Err(Error::validation(
            "OASF projection requires taxonomy version and non-empty domain labels",
        ));
    }
    Ok(OasfDiscoveryProjection {
        name: record.manifest.name.clone(),
        taxonomy_version,
        skills: record.manifest.provides.clone(),
        domains,
        provenance: DiscoveryProvenance::from_manifest(&record.manifest),
    })
}

/// Metadata imported from an external discovery registry. The type contains no
/// qualification/admission/authority fields by design, so callers cannot
/// deserialize registry declarations into governed Morn truth.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportedDiscoveryMetadata {
    pub source_kind: String,
    pub subject_ref: String,
    pub name: String,
    pub skills: Vec<String>,
    pub domains: Vec<String>,
    pub source_provenance_ref: String,
    pub metadata_class: String,
}

impl ImportedDiscoveryMetadata {
    pub fn declared(
        source_kind: impl Into<String>,
        subject_ref: impl Into<String>,
        name: impl Into<String>,
        skills: Vec<String>,
        domains: Vec<String>,
        source_provenance_ref: impl Into<String>,
    ) -> Result<Self> {
        let source_kind = source_kind.into();
        let subject_ref = subject_ref.into();
        let name = name.into();
        let source_provenance_ref = source_provenance_ref.into();
        if source_kind.trim().is_empty()
            || subject_ref.trim().is_empty()
            || name.trim().is_empty()
            || source_provenance_ref.trim().is_empty()
        {
            return Err(Error::validation(
                "imported discovery metadata requires source, subject, name and provenance",
            ));
        }
        Ok(Self {
            source_kind,
            subject_ref,
            name,
            skills,
            domains,
            source_provenance_ref,
            metadata_class: "declared".to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CapabilityManifest, CapabilityStage, EffectClass};
    use morn_kernel::ids::CapabilityId;

    fn agent_record() -> CapabilityRecord {
        let mut manifest = CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            "planner",
            "agent-provider",
            CapabilityKind::Agent,
            EffectClass::E0LifecycleReversible,
        );
        manifest.provides = vec!["plan".to_string(), "summarize".to_string()];
        manifest.provenance.source_ref = "repo://planner".to_string();
        manifest.digest = Some("sha256:abc".to_string());
        let mut record = CapabilityRecord::new(manifest);
        // Even a qualified/admitted canonical record is exported only as
        // declared discovery metadata.
        record.stage = CapabilityStage::Qualified;
        record.qualification_refs.push("qualification://q1".to_string());
        record.admitted_sites.push("site-a".to_string());
        record
    }

    #[test]
    fn xregistry_projection_never_exports_governance_truth() {
        let projected = project_xregistry(&agent_record()).unwrap();
        assert!(projected.provenance.is_declared_only());
        let json = serde_json::to_value(projected).unwrap();
        for forbidden in [
            "qualification_refs",
            "release_refs",
            "admission_refs",
            "admitted_sites",
            "authority",
            "stage",
        ] {
            assert!(json.get(forbidden).is_none(), "{forbidden} leaked");
        }
    }

    #[test]
    fn a2a_and_oasf_are_agent_only_declared_metadata() {
        let record = agent_record();
        let card = project_a2a_agent_card(
            &record,
            "https://agent.example/.well-known/agent-card.json",
            "1.0",
        )
        .unwrap();
        assert_eq!(card.skills.len(), 2);
        assert!(card.provenance.is_declared_only());
        assert!(project_a2a_agent_card(&record, "https://agent.example/card", "1.0.0").is_err());

        let oasf = project_oasf(&record, "0.9", vec!["operations".to_string()]).unwrap();
        assert_eq!(oasf.skills, record.manifest.provides);
        assert!(oasf.provenance.is_declared_only());

        let mut non_agent = record;
        non_agent.manifest.kind = CapabilityKind::Program;
        assert!(project_a2a_agent_card(&non_agent, "https://example/card", "1.0").is_err());
        assert!(project_oasf(&non_agent, "0.9", vec![]).is_err());
    }

    #[test]
    fn imported_registry_metadata_cannot_claim_qualification() {
        let imported = ImportedDiscoveryMetadata::declared(
            "a2a-agent-card",
            "a2a://planner",
            "planner",
            vec!["plan".to_string()],
            vec!["operations".to_string()],
            "https://agent.example/.well-known/agent-card.json",
        )
        .unwrap();
        let json = serde_json::to_value(imported).unwrap();
        assert_eq!(json["metadata_class"], "declared");
        assert!(json.get("qualified").is_none());
        assert!(json.get("admitted").is_none());
        assert!(json.get("authority").is_none());
    }
}
