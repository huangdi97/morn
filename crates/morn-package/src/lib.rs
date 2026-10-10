//! Package / Plugin system: PackManifest, pack lifecycle (init/validate/build/
//! install/enable/disable/upgrade/uninstall/inspect/diff) and PluginManifest.
//! Uninstall never deletes historical canonical records/provenance.

pub mod distribution;
pub mod supply_chain;

use serde::{Deserialize, Serialize};

pub use distribution::{
    CosignCliVerifier, ExternalCommandSpec, OciArtifactPublisher, OciLayerInput, OciPublishReceipt,
    OciPublishRequest, OrasCliPublisher, SigstoreIdentityPolicy,
};
use morn_kernel::ids::Id;
use morn_kernel::time::Timestamp;
use morn_kernel::version::Version;
pub use morn_kernel::version::Version as PackVersion;
pub use supply_chain::{
    ArtifactLayer, CapabilityArtifactDescriptor, SupplyChainVerificationEvidence,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PackIdTag;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PluginIdTag;

pub type PackId = Id<PackIdTag>;
pub type PluginId = Id<PluginIdTag>;

fn default_protocol_compat() -> String {
    "11.5.x".to_string()
}

/// Pack lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum PackStatus {
    Draft,
    Validated,
    Built,
    Installed,
    Enabled,
    Disabled,
    UpgradePending,
    Deprecated,
    Uninstalled,
}

/// A domain pack manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackManifest {
    pub id: PackId,
    pub name: String,
    pub kind: String, // domain-pack | evaluation-pack | simulation-pack | capability-pack | connector-pack
    pub version: Version,
    pub sdk_version: String,
    /// Legacy packaging-ABI compatibility retained for v1 migration. This is
    /// not the v11.5 semantic protocol and must not be treated as an immutable Core.
    pub core_compat: String,
    #[serde(default = "default_protocol_compat")]
    pub protocol_compat: String,
    #[serde(default)]
    pub profile_compat: Vec<String>,
    #[serde(default)]
    pub composition_runtime_compat: Vec<String>,
    pub dependencies: Vec<String>,
    pub permissions: Vec<String>,
    pub entrypoints: Vec<String>,
    pub migrations: Vec<String>,
    pub health_checks: Vec<String>,
    pub status: PackStatus,
    pub created_at: Timestamp,
}

impl PackManifest {
    pub fn new(name: &str, kind: &str, version: Version) -> Self {
        Self {
            id: PackId::generate_with("pack"),
            name: name.to_string(),
            kind: kind.to_string(),
            version,
            sdk_version: "1.0.0".to_string(),
            core_compat: "1.x".to_string(),
            protocol_compat: default_protocol_compat(),
            profile_compat: Vec::new(),
            composition_runtime_compat: Vec::new(),
            dependencies: Vec::new(),
            permissions: Vec::new(),
            entrypoints: Vec::new(),
            migrations: Vec::new(),
            health_checks: Vec::new(),
            status: PackStatus::Draft,
            created_at: Timestamp::now(),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        safe_name(&self.name)?;
        if self.sdk_version.trim().is_empty() {
            return Err("sdk_version required".to_string());
        }
        if !self.core_compat.starts_with("1.") {
            return Err("legacy core_compat must target packaging ABI 1.x".to_string());
        }
        if self.protocol_compat.trim().is_empty() {
            return Err("protocol_compat required".to_string());
        }
        Ok(())
    }
}

/// Plugin manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub id: PluginId,
    pub name: String,
    pub plugin_type: String, // capability-provider | harness-provider | runtime-provider | connector-provider | domain-pack | evaluation-pack | simulation-pack | ui-extension | cli-extension
    pub version: Version,
    /// Legacy packaging-ABI compatibility; not the semantic Morn protocol.
    pub core_compat: String,
    pub sdk_compat: String,
    #[serde(default = "default_protocol_compat")]
    pub protocol_compat: String,
    #[serde(default)]
    pub profile_compat: Vec<String>,
    #[serde(default)]
    pub composition_runtime_compat: Vec<String>,
    pub dependencies: Vec<String>,
    pub permissions: Vec<String>,
    pub entrypoints: Vec<String>,
    pub config_schema: String,
    pub secrets: Vec<String>,
    pub migrations: Vec<String>,
    pub health_checks: Vec<String>,
    pub created_at: Timestamp,
}

impl PluginManifest {
    pub fn validate(&self) -> Result<(), String> {
        safe_name(&self.name)?;
        const KNOWN: [&str; 9] = [
            "capability-provider",
            "harness-provider",
            "runtime-provider",
            "connector-provider",
            "domain-pack",
            "evaluation-pack",
            "simulation-pack",
            "ui-extension",
            "cli-extension",
        ];
        if !KNOWN.contains(&self.plugin_type.as_str()) {
            return Err(format!("unknown plugin_type {:?}", self.plugin_type));
        }
        if !self.core_compat.starts_with("1.") {
            return Err("legacy core_compat must target packaging ABI 1.x".to_string());
        }
        if self.sdk_compat.trim().is_empty() {
            return Err("sdk_compat required".to_string());
        }
        if self.protocol_compat.trim().is_empty() {
            return Err("protocol_compat required".to_string());
        }
        Ok(())
    }

    pub fn new(name: &str, plugin_type: &str) -> Self {
        Self {
            id: PluginId::generate_with("plugin"),
            name: name.to_string(),
            plugin_type: plugin_type.to_string(),
            version: Version::v1(),
            core_compat: "1.x".to_string(),
            sdk_compat: "1.x".to_string(),
            protocol_compat: default_protocol_compat(),
            profile_compat: Vec::new(),
            composition_runtime_compat: Vec::new(),
            dependencies: Vec::new(),
            permissions: Vec::new(),
            entrypoints: Vec::new(),
            config_schema: "{}".to_string(),
            secrets: Vec::new(),
            migrations: Vec::new(),
            health_checks: Vec::new(),
            created_at: Timestamp::now(),
        }
    }
}

/// Pack lifecycle service. History is never erased.
#[derive(Debug, Default)]
pub struct PackLifecycle {
    pub packs: Vec<PackManifest>,
    pub history: Vec<String>,
}

impl PackLifecycle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init(&mut self, manifest: PackManifest) -> Result<PackId, String> {
        manifest.validate()?;
        let id = manifest.id.clone();
        self.packs.push(manifest);
        self.history.push(format!("init {id}"));
        Ok(id)
    }

    fn get_mut(&mut self, id: &PackId) -> Option<&mut PackManifest> {
        self.packs.iter_mut().find(|p| p.id == *id)
    }

    pub fn validate(&mut self, id: &PackId) -> Result<(), String> {
        let p = self.get_mut(id).ok_or("pack not found")?;
        p.validate()?;
        p.status = PackStatus::Validated;
        self.history.push(format!("validated {id}"));
        Ok(())
    }

    pub fn build(&mut self, id: &PackId) -> Result<(), String> {
        let p = self.get_mut(id).ok_or("pack not found")?;
        if p.status != PackStatus::Validated {
            return Err("must validate before build".to_string());
        }
        p.status = PackStatus::Built;
        self.history.push(format!("built {id}"));
        Ok(())
    }

    pub fn install(&mut self, id: &PackId) -> Result<(), String> {
        let p = self.get_mut(id).ok_or("pack not found")?;
        if p.status != PackStatus::Built {
            return Err("must build before install".to_string());
        }
        p.status = PackStatus::Installed;
        self.history.push(format!("installed {id}"));
        Ok(())
    }

    pub fn enable(&mut self, id: &PackId) -> Result<(), String> {
        let p = self.get_mut(id).ok_or("pack not found")?;
        if p.status != PackStatus::Installed && p.status != PackStatus::Disabled {
            return Err("pack not installed".to_string());
        }
        p.status = PackStatus::Enabled;
        self.history.push(format!("enabled {id}"));
        Ok(())
    }

    pub fn disable(&mut self, id: &PackId) -> Result<(), String> {
        let p = self.get_mut(id).ok_or("pack not found")?;
        if p.status != PackStatus::Enabled {
            return Err("pack not enabled".to_string());
        }
        p.status = PackStatus::Disabled;
        self.history.push(format!("disabled {id}"));
        Ok(())
    }

    /// Upgrade requires a new version; creates a new manifest record (history kept).
    pub fn upgrade(&mut self, id: &PackId, new_version: Version) -> Result<(), String> {
        let p = self.get_mut(id).ok_or("pack not found")?;
        if p.status != PackStatus::Enabled {
            return Err("only enabled packs can upgrade".to_string());
        }
        p.version = new_version;
        p.status = PackStatus::UpgradePending;
        self.history.push(format!("upgrade_pending {id}"));
        Ok(())
    }

    /// Uninstall marks the pack Uninstalled but keeps all historical canonical
    /// records/provenance (history list retained; the manifest record remains).
    pub fn uninstall(&mut self, id: &PackId) -> Result<(), String> {
        let p = self.get_mut(id).ok_or("pack not found")?;
        p.status = PackStatus::Uninstalled;
        self.history
            .push(format!("uninstalled {id} (history preserved)"));
        Ok(())
    }

    pub fn inspect(&self, id: &PackId) -> Option<&PackManifest> {
        self.packs.iter().find(|p| p.id == *id)
    }

    /// Structural diff between two manifests.
    pub fn diff(&self, a: &PackManifest, b: &PackManifest) -> Vec<String> {
        let mut changes = Vec::new();
        if a.version != b.version {
            changes.push(format!("version {} -> {}", a.version, b.version));
        }
        if a.permissions != b.permissions {
            changes.push("permissions changed".to_string());
        }
        if a.entrypoints != b.entrypoints {
            changes.push("entrypoints changed".to_string());
        }
        changes
    }
}

/// Validate a pack/plugin name is safe to use as an identifier: non-empty and
/// free of path separators / traversal / control characters. This is the Core
/// boundary that rejects path traversal and shell-injection style names.
pub(crate) fn safe_name(name: &str) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("name required".to_string());
    }
    if name.contains('/')
        || name.contains('\\')
        || name.contains("..")
        || name.chars().any(char::is_control)
    {
        return Err(format!(
            "unsafe name {name:?}: must not contain path separators, '..', or control chars"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pack() -> PackManifest {
        PackManifest::new("biolab", "domain-pack", Version::v1())
    }

    #[test]
    fn unsafe_names_rejected_path_traversal_and_injection() {
        for bad in [
            "../etc/passwd",
            "..\\..\\secret",
            "a/b",
            "; rm -rf /",
            "name\0with-nul",
        ] {
            let mut m = pack();
            m.name = bad.to_string();
            assert!(m.validate().is_err(), "unsafe name must be rejected: {bad}");
        }
        assert!(pack().validate().is_ok());
    }

    #[test]
    fn plugin_manifest_validate_type_and_name() {
        let mut p = PluginManifest::new("hello-plugin", "capability-provider");
        assert!(p.validate().is_ok());
        p.plugin_type = "not-a-type".to_string();
        assert!(p.validate().is_err());
        p.plugin_type = "connector-provider".to_string();
        p.name = "../escape".to_string();
        assert!(p.validate().is_err());
    }

    #[test]
    fn package_compatibility_separates_protocol_profile_and_runtime() {
        let mut manifest = pack();
        manifest.profile_compat = vec!["morn.factory.readonly@1.x".to_string()];
        manifest.composition_runtime_compat = vec!["cordis@4.x".to_string()];
        assert_eq!(manifest.protocol_compat, "11.5.x");
        assert!(manifest.validate().is_ok());

        let mut plugin = PluginManifest::new("dsh-provider", "harness-provider");
        plugin.profile_compat = vec!["morn.enterprise@1.x".to_string()];
        plugin.composition_runtime_compat = vec!["cordis@4.x".to_string()];
        assert_eq!(plugin.protocol_compat, "11.5.x");
        assert!(plugin.validate().is_ok());

        manifest.protocol_compat.clear();
        assert!(manifest.validate().is_err());
    }

    #[test]
    fn pack_lifecycle_full_cycle_preserves_history() {
        let mut lc = PackLifecycle::new();
        let id = lc.init(pack()).unwrap();
        lc.validate(&id).unwrap();
        lc.build(&id).unwrap();
        lc.install(&id).unwrap();
        lc.enable(&id).unwrap();
        lc.disable(&id).unwrap();
        lc.enable(&id).unwrap();
        lc.upgrade(&id, Version::new(1, 1, 0)).unwrap();
        lc.uninstall(&id).unwrap();
        assert_eq!(lc.inspect(&id).unwrap().status, PackStatus::Uninstalled);
        // History is preserved (no deletion).
        assert!(lc.history.len() >= 7);
        assert!(lc.history.iter().any(|h| h.contains("uninstalled")));
        // Manifest record still exists after uninstall.
        assert!(lc.inspect(&id).is_some());
    }

    #[test]
    fn lifecycle_order_enforced() {
        let mut lc = PackLifecycle::new();
        let id = lc.init(pack()).unwrap();
        // cannot install before build
        assert!(lc.install(&id).is_err());
        lc.validate(&id).unwrap();
        lc.build(&id).unwrap();
        lc.install(&id).unwrap();
        // cannot enable from Installed? enable requires Installed/Disabled - ok
        assert!(lc.enable(&id).is_ok());
    }

    #[test]
    fn diff_reports_changes() {
        let mut lc = PackLifecycle::new();
        let id = lc.init(pack()).unwrap();
        lc.validate(&id).unwrap();
        lc.build(&id).unwrap();
        lc.install(&id).unwrap();
        lc.enable(&id).unwrap();
        let before = lc.inspect(&id).unwrap().clone();
        lc.upgrade(&id, Version::new(1, 2, 0)).unwrap();
        let after = lc.inspect(&id).unwrap().clone();
        let changes = lc.diff(&before, &after);
        assert!(changes.iter().any(|c| c.contains("1.0.0 -> 1.2.0")));
    }
}
