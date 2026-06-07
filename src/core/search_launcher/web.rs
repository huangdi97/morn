use super::{SearchCategory, SearchItem, SearchLauncher};

impl SearchLauncher {
    pub fn register_agent_skill(
        &mut self,
        id: impl Into<String>,
        name: impl Into<String>,
        description: impl Into<String>,
    ) {
        let item = SearchItem::new(id, name, description, SearchCategory::AgentSkill);
        self.index.add(item);
    }
}
