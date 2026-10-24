//! Optional WIT/Wasm Component ABI contract.
//!
//! This module does not embed a Wasm runtime and does not replace Cordis.
//! It defines the provider-neutral checks a future local component executor
//! must satisfy before a component can be bound to a Morn capability.

use serde::{Deserialize, Serialize};

use morn_kernel::error::{Error, Result};

use crate::CapabilityManifest;

pub const WIT_COMPONENT_PROTOCOL: &str = "wit-component";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WasmComponentDescriptor {
    pub component_ref: String,
    pub component_digest: String,
    pub wit_world: String,
    pub wit_package: String,
    pub wit_version: String,
    pub wasi_version: Option<String>,
    pub imports: Vec<String>,
    pub exports: Vec<String>,
}

impl WasmComponentDescriptor {
    pub fn validate(&self) -> Result<()> {
        if self.component_ref.trim().is_empty()
            || self.component_digest.trim().is_empty()
            || self.wit_world.trim().is_empty()
            || self.wit_package.trim().is_empty()
            || self.wit_version.trim().is_empty()
        {
            return Err(Error::validation(
                "Wasm component requires immutable component and WIT identity",
            ));
        }
        if !self.component_digest.starts_with("sha256:") {
            return Err(Error::validation(
                "Wasm component must use a content-addressed sha256 digest",
            ));
        }
        Ok(())
    }
}

/// Explicit host capabilities. Empty fields mean no ambient grant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WasmHostGrant {
    pub readable_paths: Vec<String>,
    pub writable_paths: Vec<String>,
    pub network_hosts: Vec<String>,
    pub secret_handles: Vec<String>,
    pub allow_time: bool,
    pub allow_random: bool,
}

fn all_allowed(requested: &[String], allowed: &[String]) -> bool {
    requested.iter().all(|item| allowed.contains(item))
}

pub fn validate_wasm_component_binding(
    manifest: &CapabilityManifest,
    component: &WasmComponentDescriptor,
    grant: &WasmHostGrant,
) -> Result<()> {
    manifest.validate_governance()?;
    component.validate()?;

    if !manifest
        .interfaces
        .iter()
        .any(|interface| interface.protocol == WIT_COMPONENT_PROTOCOL)
    {
        return Err(Error::validation(
            "CapabilityManifest must explicitly declare a wit-component interface",
        ));
    }
    if !all_allowed(&component.imports, &manifest.requires) {
        return Err(Error::validation(
            "WIT component imports exceed the CapabilityManifest requires set",
        ));
    }
    if !all_allowed(&manifest.provides, &component.exports) {
        return Err(Error::validation(
            "WIT component exports do not satisfy the CapabilityManifest provides set",
        ));
    }
    if !all_allowed(&grant.writable_paths, &manifest.execution.writable_paths)
        || !all_allowed(&grant.network_hosts, &manifest.execution.network_allowlist)
        || !all_allowed(&grant.secret_handles, &manifest.execution.secret_refs)
    {
        return Err(Error::not_authorized(
            "Wasm host grant exceeds execution requirements",
        ));
    }

    let allows = |permission: &str| {
        manifest
            .authority
            .allow
            .iter()
            .any(|allowed| allowed == permission)
    };
    if !grant.writable_paths.is_empty() && !allows("filesystem.write") {
        return Err(Error::not_authorized(
            "Wasm filesystem writes require filesystem.write authority",
        ));
    }
    if !grant.network_hosts.is_empty() && !allows("network.egress") {
        return Err(Error::not_authorized(
            "Wasm network access requires network.egress authority",
        ));
    }
    if !grant.secret_handles.is_empty() && !allows("secret.read") {
        return Err(Error::not_authorized(
            "Wasm secret access requires secret.read authority",
        ));
    }
    if grant.allow_time && !allows("clock.read") {
        return Err(Error::not_authorized(
            "Wasm clock access requires clock.read authority",
        ));
    }
    if grant.allow_random && !allows("random.read") {
        return Err(Error::not_authorized(
            "Wasm randomness requires random.read authority",
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CapabilityInterface, CapabilityKind, CapabilityManifest, EffectClass};
    use morn_kernel::ids::CapabilityId;

    fn manifest() -> CapabilityManifest {
        let mut manifest = CapabilityManifest::new(
            CapabilityId::generate_with("cap"),
            "portable-transform",
            "wasm-provider",
            CapabilityKind::Program,
            EffectClass::E0LifecycleReversible,
        );
        manifest.provenance.source_ref = "oci://example/portable-transform".to_string();
        manifest.provides.push("transform.run".to_string());
        manifest.requires.push("config.read".to_string());
        manifest.interfaces.push(CapabilityInterface {
            protocol: WIT_COMPONENT_PROTOCOL.to_string(),
            input_schema_ref: "wit://example:transform/run@1.0.0".to_string(),
            output_schema_ref: "wit://example:transform/result@1.0.0".to_string(),
        });
        manifest
    }

    fn component() -> WasmComponentDescriptor {
        WasmComponentDescriptor {
            component_ref: "oci://example/portable-transform@sha256:abc".to_string(),
            component_digest: "sha256:abc".to_string(),
            wit_world: "transform".to_string(),
            wit_package: "example:transform".to_string(),
            wit_version: "1.0.0".to_string(),
            wasi_version: Some("0.3".to_string()),
            imports: vec!["config.read".to_string()],
            exports: vec!["transform.run".to_string()],
        }
    }

    #[test]
    fn least_capability_component_has_no_ambient_host_access() {
        validate_wasm_component_binding(&manifest(), &component(), &WasmHostGrant::default())
            .unwrap();
    }

    #[test]
    fn undeclared_import_or_host_permission_fails_closed() {
        let mut bad_component = component();
        bad_component.imports.push("ambient.fs".to_string());
        assert!(validate_wasm_component_binding(
            &manifest(),
            &bad_component,
            &WasmHostGrant::default()
        )
        .is_err());

        let mut m = manifest();
        m.execution
            .writable_paths
            .push("/workspace/out".to_string());
        let grant = WasmHostGrant {
            writable_paths: vec!["/workspace/out".to_string()],
            ..Default::default()
        };
        assert!(validate_wasm_component_binding(&m, &component(), &grant).is_err());
        m.authority.allow.push("filesystem.write".to_string());
        validate_wasm_component_binding(&m, &component(), &grant).unwrap();

        let escape = WasmHostGrant {
            writable_paths: vec!["/etc".to_string()],
            ..Default::default()
        };
        assert!(validate_wasm_component_binding(&m, &component(), &escape).is_err());
    }

    #[test]
    fn component_cannot_exist_without_explicit_wit_interface() {
        let mut m = manifest();
        m.interfaces.clear();
        assert!(
            validate_wasm_component_binding(&m, &component(), &WasmHostGrant::default()).is_err()
        );
    }
}
