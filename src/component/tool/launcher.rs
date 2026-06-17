//! launcher — SearchLauncherTool for app/file/command/skill search.
use super::Tool;
use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};
use crate::core::error::MornError;

#[allow(dead_code)] /* 预留：launcher 搜索工具注册入口 */
pub struct SearchLauncherTool {
    id: String,
    name: String,
    launcher: crate::core::search_launcher::SearchLauncher,
}

impl SearchLauncherTool {
    pub fn new() -> Self {
        let mut launcher = crate::core::search_launcher::SearchLauncher::new();
        launcher.register_command(
            "search",
            "Search",
            "Search apps, files, commands, and skills",
        );
        launcher.register_command("settings", "Settings", "Open Morn settings");
        launcher.register_agent_skill(
            "self-evolution",
            "Self Evolution",
            "Run project improvement scans",
        );
        SearchLauncherTool {
            id: "tool-search-launcher".into(),
            name: "Search Launcher".into(),
            launcher,
        }
    }
}

impl Default for SearchLauncherTool {
    fn default() -> Self {
        Self::new()
    }
}

impl Component for SearchLauncherTool {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "tool"
    }
    fn init(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn run(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn pause(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), MornError> {
        Ok(())
    }
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl IOComponent for SearchLauncherTool {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "launcher query".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "launcher results".into(),
            },
        ]
    }
    fn send(&mut self, port: &str, _data: Data) -> Result<(), MornError> {
        Err(MornError::Internal(format!(
            "SearchLauncherTool cannot receive on port {}",
            port
        )))
    }
    fn recv(&mut self, port: &str) -> Result<Option<Data>, MornError> {
        if port == "output" {
            Ok(Some(Data::text("")))
        } else {
            Ok(None)
        }
    }
}

impl SecureComponent for SearchLauncherTool {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![]
    }
}

impl Tool for SearchLauncherTool {
    fn execute(&mut self, input: Data) -> Result<Data, MornError> {
        let query = input.content.as_str().unwrap_or("").to_string();
        let results = self.launcher.search(&query);
        let summary = results
            .into_iter()
            .take(5)
            .map(|(score, item)| {
                format!(
                    "{}:{}:{:.2}:{}",
                    item.category.as_str(),
                    item.name,
                    score,
                    item.description
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        Ok(Data::text(&format!("[search_launcher]\n{}", summary)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execute_with_valid_query() {
        let mut tool = SearchLauncherTool::new();
        let result = tool.execute(Data::text("search")).unwrap();
        let output = result.content.as_str().unwrap_or("");
        assert!(output.starts_with("[search_launcher]"));
    }

    #[test]
    fn execute_with_empty_query_does_not_panic() {
        let mut tool = SearchLauncherTool::new();
        let result = tool.execute(Data::text("")).unwrap();
        let output = result.content.as_str().unwrap_or("");
        assert!(output.starts_with("[search_launcher]"));
    }
}
