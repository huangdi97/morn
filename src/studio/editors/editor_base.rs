macro_rules! make_editor {
    ($name:ident) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            pub id: String,
            pub name: String,
        }

        impl $name {
            pub fn new(id: &str, name: &str) -> Self {
                $name {
                    id: id.to_string(),
                    name: name.to_string(),
                }
            }

            pub fn load() -> Self {
                $name {
                    id: "default".into(),
                    name: format!("Default {}", stringify!($name).trim_end_matches("Editor")),
                }
            }

            pub fn save(&self) -> Result<(), String> {
                Ok(())
            }
        }
    };
}

pub(crate) use make_editor;
