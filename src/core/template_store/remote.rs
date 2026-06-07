//! remote — Retrieves workflow templates and manifests from remote stores.
use super::{TemplateManifest, TemplateStore};

impl TemplateStore {
    pub fn fetch_remote_registry(url: &str) -> Result<Vec<TemplateManifest>, String> {
        let response = reqwest::blocking::get(url)
            .map_err(|e| format!("Failed to fetch registry from '{}': {}", url, e))?;
        if !response.status().is_success() {
            return Err(format!(
                "Registry returned HTTP {} from '{}'",
                response.status(),
                url
            ));
        }
        let manifests: Vec<TemplateManifest> = response
            .json()
            .map_err(|e| format!("Failed to parse registry response: {}", e))?;
        Ok(manifests)
    }

    pub fn install_from_registry(&mut self, url: &str, template_id: &str) -> Result<(), String> {
        let manifests = Self::fetch_remote_registry(url)?;
        let manifest = manifests
            .into_iter()
            .find(|m| m.id == template_id)
            .ok_or_else(|| format!("Template '{}' not found in registry '{}'", template_id, url))?;
        self.install(manifest)
    }

    pub fn update_from_registry(&mut self, url: &str, template_id: &str) -> Result<(), String> {
        let manifests = Self::fetch_remote_registry(url)?;
        let manifest = manifests
            .into_iter()
            .find(|m| m.id == template_id)
            .ok_or_else(|| format!("Template '{}' not found in registry '{}'", template_id, url))?;
        self.update(manifest)
    }

    pub fn bulk_install_from_registry(&mut self, url: &str) -> Result<Vec<String>, String> {
        let manifests = Self::fetch_remote_registry(url)?;
        let mut installed = Vec::new();
        for manifest in manifests {
            let id = manifest.id.clone();
            if !self.is_installed(&id) {
                if let Err(e) = self.install(manifest) {
                    eprintln!("[TemplateStore] Skipped '{}': {}", id, e);
                } else {
                    installed.push(id);
                }
            }
        }
        Ok(installed)
    }
}
