//! Work System as Code: manifest schema, version, validate, diff, export, import,
//! compatibility and dry-run compile. Never auto-activates production.

use serde_json::{json, Value};

use morn_kernel::error::{Error, Result};
use morn_kernel::version::Version;

use crate::compiler::SolutionCompiler;
use crate::problem_spec::SolutionRequest;
use crate::solution::{ProposedSolution, SolutionPackage};

/// Manifest service for SolutionPackage (Work System as Code).
#[derive(Debug, Default)]
pub struct ManifestService;

impl ManifestService {
    pub fn new() -> Self {
        Self
    }

    /// Validate a manifest against the v0.2 schema.
    pub fn validate(&self, manifest: &Value) -> Result<()> {
        let morn = manifest
            .get("morn")
            .ok_or_else(|| Error::validation("manifest missing 'morn' section"))?;
        if morn.get("domain").and_then(Value::as_str).is_none() {
            return Err(Error::validation("manifest 'morn.domain' must be a string"));
        }
        let problem = manifest
            .get("problem")
            .ok_or_else(|| Error::validation("manifest missing 'problem' section"))?;
        if problem.get("objective").and_then(Value::as_str).is_none() {
            return Err(Error::validation(
                "manifest 'problem.objective' must be a string",
            ));
        }
        let wps = manifest
            .get("work_packages")
            .and_then(Value::as_array)
            .ok_or_else(|| Error::validation("manifest 'work_packages' must be an array"))?;
        if wps.is_empty() {
            return Err(Error::validation(
                "manifest must contain at least one work package",
            ));
        }
        Ok(())
    }

    /// Next version (minor bump).
    pub fn next_version(&self, version: &Version) -> Version {
        version.next_minor()
    }

    /// Structural diff between two manifests (top-level sections + work packages).
    pub fn diff(&self, base: &Value, next: &Value) -> Vec<String> {
        let mut changes = Vec::new();
        let base_domain = base
            .pointer("/morn/domain")
            .and_then(Value::as_str)
            .unwrap_or("");
        let next_domain = next
            .pointer("/morn/domain")
            .and_then(Value::as_str)
            .unwrap_or("");
        if base_domain != next_domain {
            changes.push(format!("domain changed: {base_domain} -> {next_domain}"));
        }
        let base_wps: Vec<String> = base
            .get("work_packages")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();
        let next_wps: Vec<String> = next
            .get("work_packages")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(Value::as_str)
                    .map(String::from)
                    .collect()
            })
            .unwrap_or_default();
        for wp in &next_wps {
            if !base_wps.contains(wp) {
                changes.push(format!("work package added: {wp}"));
            }
        }
        for wp in &base_wps {
            if !next_wps.contains(wp) {
                changes.push(format!("work package removed: {wp}"));
            }
        }
        changes
    }

    /// Compatibility: same domain and compatible major version (major must match).
    pub fn compatibility(&self, base: &SolutionPackage, next: &SolutionPackage) -> Result<()> {
        let base_domain = base
            .manifest
            .pointer("/morn/domain")
            .and_then(Value::as_str)
            .unwrap_or("");
        let next_domain = next
            .manifest
            .pointer("/morn/domain")
            .and_then(Value::as_str)
            .unwrap_or("");
        if base_domain != next_domain {
            return Err(Error::validation(format!(
                "domain mismatch: {base_domain} vs {next_domain}"
            )));
        }
        if base.version.major != next.version.major {
            return Err(Error::validation(format!(
                "major version incompatibility: {} vs {}",
                base.version, next.version
            )));
        }
        Ok(())
    }

    /// Export manifest to a JSON string.
    pub fn export(&self, pkg: &SolutionPackage) -> Result<String> {
        let mut doc = pkg.manifest.clone();
        doc["id"] = json!(pkg.id.to_string());
        doc["version"] = json!(pkg.version.to_string());
        serde_json::to_string_pretty(&doc).map_err(|e| Error::internal(e.to_string()))
    }

    /// Import/load a manifest JSON string back into a SolutionPackage.
    pub fn import(
        &self,
        json_str: &str,
        proposed_solution_id: &morn_kernel::ids::ProposedSolutionId,
    ) -> Result<SolutionPackage> {
        let manifest: Value = serde_json::from_str(json_str)
            .map_err(|e| Error::validation(format!("invalid manifest json: {e}")))?;
        self.validate(&manifest)?;
        let version = manifest
            .get("version")
            .and_then(Value::as_str)
            .unwrap_or("1.0.0")
            .parse::<Version>()?;
        let name = manifest
            .pointer("/problem/objective")
            .and_then(Value::as_str)
            .unwrap_or("solution")
            .to_string();
        Ok(SolutionPackage {
            id: morn_kernel::ids::SolutionPackageId::generate_with("solpkg"),
            name,
            version,
            manifest,
            proposed_solution_id: proposed_solution_id.clone(),
            approved_solution_id: None,
            created_at: morn_kernel::time::Timestamp::now(),
        })
    }

    /// Dry-run compile: analyze + propose + validate with no persistence or side effects.
    pub fn dry_run_compile(
        &self,
        request: &SolutionRequest,
    ) -> Result<(ProposedSolution, Vec<String>)> {
        let mut compiler = SolutionCompiler::new();
        let (problem, graph) = compiler.analyze(request)?;
        let proposed = compiler.propose(&problem, &graph, request)?;
        let report = compiler.validate(&proposed, &graph, request)?;
        let issues: Vec<String> = report.issues.iter().map(|i| i.message.clone()).collect();
        Ok((proposed, issues))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compiler::SolutionCompiler;
    use crate::problem_spec::SolutionRequest;
    use morn_kernel::ids::{ProposedSolutionId, WorkspaceId};

    fn request() -> SolutionRequest {
        let ws = WorkspaceId::generate();
        let mut req = SolutionRequest::new(ws, "Dataset to Reviewed Scientific Claim", "biolab");
        req.available_capabilities.push("*".to_string());
        req.available_harnesses.push("morn-native".to_string());
        req
    }

    fn package() -> (SolutionPackage, SolutionPackage) {
        let mut compiler = SolutionCompiler::new();
        let req = request();
        let (problem, graph) = compiler.analyze(&req).unwrap();
        let proposed = compiler.propose(&problem, &graph, &req).unwrap();
        let approved = compiler.approve(proposed.id.clone(), "pi");
        let v1 = compiler.compile(&approved, &proposed, &problem).unwrap();
        let v2 = SolutionPackage {
            version: v1.version.next_minor(),
            ..v1.clone()
        };
        (v1, v2)
    }

    #[test]
    fn manifest_validate_export_import_roundtrip() {
        let svc = ManifestService::new();
        let (pkg, _) = package();
        svc.validate(&pkg.manifest).unwrap();
        let exported = svc.export(&pkg).unwrap();
        let imported = svc.import(&exported, &pkg.proposed_solution_id).unwrap();
        assert_eq!(imported.version, pkg.version);
        assert_eq!(imported.name, pkg.name);
    }

    #[test]
    fn diff_detects_changes_and_compatibility_enforced() {
        let svc = ManifestService::new();
        let (v1, v2) = package();
        assert!(svc.compatibility(&v1, &v2).is_ok());
        // incompatible domain
        let mut bad = v2.clone();
        bad.manifest["morn"]["domain"] = json!("pharma");
        assert!(svc.compatibility(&v1, &bad).is_err());
        // diff detects removed/added work packages
        let mut next = v2.manifest.clone();
        next["work_packages"] = json!(["new-wp"]);
        let changes = svc.diff(&v1.manifest, &next);
        assert!(changes.iter().any(|c| c.contains("added")));
        assert!(changes.iter().any(|c| c.contains("removed")));
    }

    #[test]
    fn invalid_manifest_rejected() {
        let svc = ManifestService::new();
        assert!(svc.validate(&json!({"morn": {}})).is_err());
    }

    #[test]
    fn dry_run_compile_no_side_effects() {
        let svc = ManifestService::new();
        let req = request();
        let (proposed, issues) = svc.dry_run_compile(&req).unwrap();
        assert!(!proposed.work_packages.is_empty());
        let _ = ProposedSolutionId::new("x");
        assert!(issues.is_empty() || issues.iter().all(|i| !i.contains("no work packages")));
    }
}
