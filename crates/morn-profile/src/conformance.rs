//! Profile conformance evaluation.
//!
//! Profiles describe guarantees, not provider names. A composition is conformant
//! only when every required semantic is satisfied and the execution guarantees
//! meet the profile floor.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{DomainProfile, RequirementLevel};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConformanceEvidence {
    pub satisfied_semantics: BTreeSet<String>,
    pub forbidden_semantics_present: BTreeSet<String>,
    pub isolation: String,
    pub durable_work_state: bool,
    pub source_of_truth_bound: bool,
    pub provenance_ready: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConformanceReport {
    pub profile_ref: String,
    pub passed: bool,
    pub missing: Vec<String>,
    pub violations: Vec<String>,
}

fn isolation_rank(value: &str) -> u8 {
    match value.to_ascii_lowercase().as_str() {
        "none" | "noisolation" => 0,
        "process" => 1,
        "container" => 2,
        "microvm" | "micro_vm" => 3,
        "fullvm" | "full_vm" | "vm" => 4,
        "remote" => 5,
        "physical" => 6,
        _ => 0,
    }
}

pub fn evaluate_profile(
    profile: &DomainProfile,
    evidence: &ConformanceEvidence,
) -> ConformanceReport {
    let mut missing = Vec::new();
    let mut violations = Vec::new();

    for requirement in &profile.requirements {
        match requirement.level {
            RequirementLevel::Required => {
                if !evidence.satisfied_semantics.contains(&requirement.semantic) {
                    missing.push(requirement.semantic.clone());
                }
            }
            RequirementLevel::Forbidden => {
                if evidence.satisfied_semantics.contains(&requirement.semantic)
                    || evidence
                        .forbidden_semantics_present
                        .contains(&requirement.semantic)
                {
                    violations.push(requirement.semantic.clone());
                }
            }
            RequirementLevel::Optional => {}
        }
    }

    if profile.durable_work_state_required && !evidence.durable_work_state {
        missing.push("DurableWorkState".to_string());
    }
    if profile.source_of_truth_binding_required && !evidence.source_of_truth_bound {
        missing.push("SourceOfTruthBinding".to_string());
    }
    if profile.provenance_required && !evidence.provenance_ready {
        missing.push("Provenance".to_string());
    }
    if isolation_rank(&evidence.isolation) < isolation_rank(&profile.minimum_isolation) {
        missing.push(format!(
            "Isolation>={}",
            profile.minimum_isolation
        ));
    }

    missing.sort();
    missing.dedup();
    violations.sort();
    violations.dedup();

    ConformanceReport {
        profile_ref: format!(
            "{}@{}.{}.{}",
            profile.id, profile.version.major, profile.version.minor, profile.version.patch
        ),
        passed: missing.is_empty() && violations.is_empty(),
        missing,
        violations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_profile_fails_closed_when_a_required_guarantee_is_missing() {
        let profile = DomainProfile::factory_readonly_v1();
        let mut evidence = ConformanceEvidence {
            isolation: "container".to_string(),
            durable_work_state: true,
            source_of_truth_bound: true,
            provenance_ready: true,
            ..Default::default()
        };
        for requirement in &profile.requirements {
            if requirement.semantic != "IndependentAcceptance" {
                evidence
                    .satisfied_semantics
                    .insert(requirement.semantic.clone());
            }
        }

        let report = evaluate_profile(&profile, &evidence);
        assert!(!report.passed);
        assert!(report.missing.iter().any(|item| item == "IndependentAcceptance"));
    }

    #[test]
    fn provider_names_are_irrelevant_to_profile_conformance() {
        let profile = DomainProfile::factory_readonly_v1();
        let mut evidence = ConformanceEvidence {
            isolation: "microvm".to_string(),
            durable_work_state: true,
            source_of_truth_bound: true,
            provenance_ready: true,
            ..Default::default()
        };
        evidence.satisfied_semantics.extend(
            profile
                .requirements
                .iter()
                .filter(|requirement| requirement.level == RequirementLevel::Required)
                .map(|requirement| requirement.semantic.clone()),
        );

        assert!(evaluate_profile(&profile, &evidence).passed);
    }
}
