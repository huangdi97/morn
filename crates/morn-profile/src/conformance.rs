//! Profile conformance evaluation.
//!
//! Profiles describe guarantees, not provider names. A composition is conformant
//! only when every required semantic is satisfied and the execution guarantees
//! meet the profile floor.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use morn_kernel::ExecutionGuarantee;

use crate::{DomainProfile, RequirementLevel};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConformanceEvidence {
    pub satisfied_semantics: BTreeSet<String>,
    pub forbidden_semantics_present: BTreeSet<String>,
    pub isolation: String,
    #[serde(default)]
    pub execution_guarantees: BTreeSet<ExecutionGuarantee>,
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

fn normalized_isolation(value: &str) -> &str {
    match value {
        "none" | "noisolation" => "none",
        "process" => "process",
        "container" => "container",
        "microvm" | "micro_vm" => "microvm",
        "fullvm" | "full_vm" | "vm" => "fullvm",
        "remote" => "remote",
        "physical" => "physical",
        _ => "unknown",
    }
}

fn isolation_satisfies(actual: &str, required: &str) -> bool {
    let actual = actual.to_ascii_lowercase();
    let required = required.to_ascii_lowercase();
    match normalized_isolation(&required) {
        "none" => true,
        "process" => matches!(
            normalized_isolation(&actual),
            "process" | "container" | "microvm" | "fullvm"
        ),
        "container" => matches!(
            normalized_isolation(&actual),
            "container" | "microvm" | "fullvm"
        ),
        "microvm" => matches!(normalized_isolation(&actual), "microvm" | "fullvm"),
        "fullvm" => normalized_isolation(&actual) == "fullvm",
        // Remote and physical are executor/topology classes. They are not
        // ordinal security levels above a VM.
        "remote" => normalized_isolation(&actual) == "remote",
        "physical" => normalized_isolation(&actual) == "physical",
        _ => false,
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
    if !isolation_satisfies(&evidence.isolation, &profile.minimum_isolation) {
        missing.push(format!("IsolationSatisfies({})", profile.minimum_isolation));
    }
    for required in &profile.required_execution_guarantees {
        if !evidence.execution_guarantees.contains(required) {
            missing.push(format!("ExecutionGuarantee({})", required.key()));
        }
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
            execution_guarantees: profile
                .required_execution_guarantees
                .iter()
                .copied()
                .collect(),
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
        assert!(report
            .missing
            .iter()
            .any(|item| item == "IndependentAcceptance"));
    }

    #[test]
    fn remote_execution_does_not_satisfy_container_by_rank() {
        let profile = DomainProfile::factory_readonly_v1();
        let mut evidence = ConformanceEvidence {
            isolation: "remote".to_string(),
            execution_guarantees: profile
                .required_execution_guarantees
                .iter()
                .copied()
                .collect(),
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
        let report = evaluate_profile(&profile, &evidence);
        assert!(!report.passed);
        assert!(report
            .missing
            .iter()
            .any(|item| item == "IsolationSatisfies(container)"));
    }

    #[test]
    fn missing_network_egress_guarantee_fails_factory_profile() {
        let profile = DomainProfile::factory_readonly_v1();
        let mut evidence = ConformanceEvidence {
            isolation: "container".to_string(),
            execution_guarantees: profile
                .required_execution_guarantees
                .iter()
                .copied()
                .filter(|guarantee| *guarantee != ExecutionGuarantee::NetworkEgressPolicy)
                .collect(),
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

        let report = evaluate_profile(&profile, &evidence);
        assert!(!report.passed);
        assert!(report
            .missing
            .iter()
            .any(|item| item == "ExecutionGuarantee(network-egress-policy)"));
    }

    #[test]
    fn provider_names_are_irrelevant_to_profile_conformance() {
        let profile = DomainProfile::factory_readonly_v1();
        let mut evidence = ConformanceEvidence {
            isolation: "microvm".to_string(),
            execution_guarantees: profile
                .required_execution_guarantees
                .iter()
                .copied()
                .collect(),
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
