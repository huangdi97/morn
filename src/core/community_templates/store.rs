//! store — Stores and queries community template registry metadata.
use super::{CommunityTemplateRegistry, RegistryInfo, RemoteTemplate};

impl CommunityTemplateRegistry {
    pub fn install_templates(&mut self, ids: &[String]) -> Result<Vec<String>, String> {
        if self.cache.is_empty() {
            self.fetch_registry(None)?;
        }

        let mut installed = Vec::new();
        for id in ids {
            if self.installed.contains_key(id) {
                return Err(format!("Template '{}' is already installed", id));
            }

            let template = self
                .cache
                .get(id)
                .ok_or_else(|| format!("Template '{}' not found in registry", id))?
                .clone();

            self.installed.insert(id.clone(), template);
            installed.push(id.clone());
        }

        Ok(installed)
    }

    pub fn uninstall(&mut self, id: &str) -> Result<RemoteTemplate, String> {
        self.installed
            .remove(id)
            .ok_or_else(|| format!("Template '{}' is not installed", id))
    }

    pub fn is_installed(&self, id: &str) -> bool {
        self.installed.contains_key(id)
    }

    pub fn list_installed(&self) -> Vec<&RemoteTemplate> {
        let mut list: Vec<_> = self.installed.values().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn list_cached(&self) -> Vec<&RemoteTemplate> {
        let mut list: Vec<_> = self.cache.values().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn search(&self, query: &str) -> Vec<&RemoteTemplate> {
        let q = query.to_lowercase();
        self.cache
            .values()
            .filter(|t| {
                t.name.to_lowercase().contains(&q)
                    || t.description.to_lowercase().contains(&q)
                    || t.tags.iter().any(|tag| tag.to_lowercase().contains(&q))
            })
            .collect()
    }

    pub fn add_registry(&mut self, registry: RegistryInfo) {
        self.registries.push(registry);
    }

    pub fn list_registries(&self) -> &[RegistryInfo] {
        &self.registries
    }

    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}
