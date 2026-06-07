use super::{compare_versions, TemplateManifest, TemplateStore};

impl TemplateStore {
    pub fn install(&mut self, manifest: TemplateManifest) -> Result<(), String> {
        if self.templates.contains_key(&manifest.id) {
            return Err(format!(
                "Template '{}' is already installed (version {}). Uninstall first or use update.",
                manifest.id, manifest.current_version
            ));
        }
        self.templates.insert(manifest.id.clone(), manifest);
        Ok(())
    }

    pub fn uninstall(&mut self, id: &str) -> Result<TemplateManifest, String> {
        self.templates
            .remove(id)
            .ok_or_else(|| format!("Template '{}' is not installed", id))
    }

    pub fn update(&mut self, manifest: TemplateManifest) -> Result<(), String> {
        let id = manifest.id.clone();
        if !self.templates.contains_key(&id) {
            return Err(format!(
                "Template '{}' is not installed. Use install() first.",
                id
            ));
        }
        self.templates.insert(id, manifest);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&TemplateManifest> {
        self.templates.get(id)
    }

    pub fn list(&self) -> Vec<&TemplateManifest> {
        let mut list: Vec<_> = self.templates.values().collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn list_by_category(&self, category: &str) -> Vec<&TemplateManifest> {
        let mut list: Vec<_> = self
            .templates
            .values()
            .filter(|t| t.category == category)
            .collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn list_by_tag(&self, tag: &str) -> Vec<&TemplateManifest> {
        let mut list: Vec<_> = self
            .templates
            .values()
            .filter(|t| t.tags.iter().any(|t| t == tag))
            .collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }

    pub fn is_installed(&self, id: &str) -> bool {
        self.templates.contains_key(id)
    }

    pub fn count(&self) -> usize {
        self.templates.len()
    }

    pub fn is_empty(&self) -> bool {
        self.templates.is_empty()
    }

    pub fn get_version(&self, id: &str) -> Option<&str> {
        self.templates.get(id).map(|t| t.current_version.as_str())
    }

    pub fn has_update(&self, id: &str, remote_version: &str) -> Result<bool, String> {
        let local = self
            .templates
            .get(id)
            .ok_or_else(|| format!("Template '{}' not found", id))?;
        Ok(compare_versions(&local.current_version, remote_version).is_lt())
    }
}
