//! tool — Defines executable tools and their component integration.
use crate::core::component::{Data, IOComponent};

pub trait Tool: IOComponent {
    fn execute(&mut self, input: Data) -> Result<Data, String>;
}

pub mod builtins;
pub mod code_exec;
pub mod file_ops;
pub mod launcher;
pub mod web_search;

pub use builtins::{CalcTool, GetTimeTool, HttpRequestTool, SendMsgTool};
pub use code_exec::ExecPythonTool;
pub use file_ops::{ReadFileTool, WriteFileTool};
pub use launcher::SearchLauncherTool;
pub use web_search::WebSearchTool;

pub fn create_default_tools() -> Vec<Box<dyn Tool>> {
    vec![
        Box::new(WebSearchTool::new()),
        Box::new(ReadFileTool::new()),
        Box::new(WriteFileTool::new()),
        Box::new(ExecPythonTool::new()),
        Box::new(GetTimeTool::new()),
        Box::new(CalcTool::new()),
        Box::new(SendMsgTool::new()),
        Box::new(HttpRequestTool::new()),
        Box::new(SearchLauncherTool::new()),
    ]
}

pub fn get_tool_by_name(name: &str) -> Option<Box<dyn Tool>> {
    match name {
        "web_search" => Some(Box::new(WebSearchTool::new())),
        "read_file" => Some(Box::new(ReadFileTool::new())),
        "write_file" => Some(Box::new(WriteFileTool::new())),
        "exec_python" => Some(Box::new(ExecPythonTool::new())),
        "get_time" => Some(Box::new(GetTimeTool::new())),
        "calc" => Some(Box::new(CalcTool::new())),
        "send_msg" => Some(Box::new(SendMsgTool::new())),
        "http_request" => Some(Box::new(HttpRequestTool::new())),
        "search_launcher" => Some(Box::new(SearchLauncherTool::new())),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_registry() {
        for name in &[
            "web_search",
            "read_file",
            "write_file",
            "exec_python",
            "get_time",
            "calc",
            "send_msg",
            "http_request",
            "search_launcher",
        ] {
            let tool = get_tool_by_name(name);
            assert!(tool.is_some(), "tool {} should exist", name);
        }
        assert!(get_tool_by_name("nonexistent").is_none());
    }

    #[test]
    fn test_tool_execution_basic() {
        let mut tool = WebSearchTool::new();
        let result = tool.execute(Data::text("rust language")).unwrap();
        assert!(result.content.as_str().unwrap().contains("web_search"));
        assert!(result.content.as_str().unwrap().contains("rust language"));

        let mut calc = CalcTool::new();
        let result = calc.execute(Data::text("2+2")).unwrap();
        assert!(result.content.as_str().unwrap().contains("calc"));
        assert!(result.content.as_str().unwrap().contains("2+2"));
    }

    #[test]
    fn test_tool_error_handling() {
        let unknown = get_tool_by_name("unknown_tool");
        assert!(unknown.is_none());

        let mut tool = WriteFileTool::new();
        let result = tool.send("invalid", Data::text("test"));
        assert!(result.is_err());
    }
}
