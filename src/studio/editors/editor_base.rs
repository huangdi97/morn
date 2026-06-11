//! editor_base — Base editor types including EditorPosition, NodeEditorFields, and the make_editor! macro.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct EditorPosition {
    pub x: f64,
    pub y: f64,
}

impl Default for EditorPosition {
    fn default() -> Self {
        EditorPosition { x: 0.0, y: 0.0 }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeEditorFields {
    pub id: String,
    pub name: String,
    pub node_type: String,
    pub description: String,
    pub position: EditorPosition,
    pub metadata: serde_json::Value,
}

impl NodeEditorFields {
    pub fn new(id: &str, name: &str, node_type: &str) -> Self {
        NodeEditorFields {
            id: id.to_string(),
            name: name.to_string(),
            node_type: node_type.to_string(),
            description: String::new(),
            position: EditorPosition::default(),
            metadata: serde_json::json!({}),
        }
    }

    pub fn to_config(&self) -> serde_json::Value {
        serde_json::json!({
            "id": self.id,
            "name": self.name,
            "node_type": self.node_type,
            "description": self.description,
            "position": self.position,
            "metadata": self.metadata,
        })
    }
}

macro_rules! make_editor {
    ($name:ident) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            pub id: String,
            pub name: String,
            pub node_type: String,
            pub description: String,
            pub position: crate::studio::editors::editor_base::EditorPosition,
            pub metadata: serde_json::Value,
        }

        impl $name {
            pub fn new(id: &str, name: &str) -> Self {
                $name {
                    id: id.to_string(),
                    name: name.to_string(),
                    node_type: stringify!($name)
                        .trim_end_matches("Editor")
                        .to_ascii_lowercase(),
                    description: String::new(),
                    position: crate::studio::editors::editor_base::EditorPosition::default(),
                    metadata: serde_json::json!({}),
                }
            }

            pub fn load() -> Self {
                $name {
                    id: "default".into(),
                    name: format!("Default {}", stringify!($name).trim_end_matches("Editor")),
                    node_type: stringify!($name)
                        .trim_end_matches("Editor")
                        .to_ascii_lowercase(),
                    description: String::new(),
                    position: crate::studio::editors::editor_base::EditorPosition::default(),
                    metadata: serde_json::json!({}),
                }
            }

            pub fn save(&self) -> Result<(), String> {
                Ok(())
            }

            pub fn set_position(&mut self, x: f64, y: f64) {
                self.position = crate::studio::editors::editor_base::EditorPosition { x, y };
            }

            pub fn to_config(&self) -> serde_json::Value {
                serde_json::json!({
                    "id": self.id,
                    "name": self.name,
                    "node_type": self.node_type,
                    "description": self.description,
                    "position": self.position,
                    "metadata": self.metadata,
                })
            }
        }
    };
}

pub(crate) use make_editor;
