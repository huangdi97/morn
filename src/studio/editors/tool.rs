#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PortDef {
    pub name: String,
    pub direction: String,
    pub data_type: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ToolEditor {
    pub name: String,
    pub ports: Vec<PortDef>,
    pub implementation_code: String,
    pub permissions: Vec<String>,
}

impl ToolEditor {
    pub fn new(name: &str) -> Self {
        ToolEditor {
            name: name.to_string(),
            ports: vec![
                PortDef { name: "input".into(), direction: "in".into(), data_type: "any".into() },
                PortDef { name: "output".into(), direction: "out".into(), data_type: "any".into() },
            ],
            implementation_code: String::new(),
            permissions: vec!["read".to_string()],
        }
    }

    pub fn to_config(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "tool",
            "name": self.name,
            "ports": self.ports,
            "implementation": self.implementation_code,
            "permissions": self.permissions,
        })
    }
}
