//! fetch — Retrieves community template registries from remote sources.
use super::{CommunityTemplateRegistry, RemoteTemplate};

impl CommunityTemplateRegistry {
    pub fn fetch_registry(&mut self, url: Option<&str>) -> Result<Vec<RemoteTemplate>, String> {
        let fetch_url = url.unwrap_or(&self.registry_url);

        let response = reqwest::blocking::get(fetch_url)
            .map_err(|e| format!("Failed to fetch registry from '{}': {}", fetch_url, e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Registry returned HTTP {} from '{}'",
                response.status(),
                fetch_url
            ));
        }

        let templates: Vec<RemoteTemplate> = response
            .json()
            .map_err(|e| format!("Failed to parse registry response: {}", e))?;

        for t in &templates {
            self.cache.insert(t.id.clone(), t.clone());
        }

        Ok(templates)
    }

    pub fn check_updates(
        &mut self,
        url: Option<&str>,
    ) -> Result<Vec<(String, String, String)>, String> {
        let fetch_url = url.unwrap_or(&self.registry_url).to_string();

        let remote = self.fetch_registry(Some(&fetch_url))?;

        let mut updates = Vec::new();
        for t in &remote {
            if let Some(local) = self.installed.get(&t.id) {
                if local.version != t.version {
                    updates.push((t.id.clone(), local.version.clone(), t.version.clone()));
                }
            }
        }

        Ok(updates)
    }
}
