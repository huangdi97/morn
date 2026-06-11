//! Component type definition and trait implementations.
pub struct ComponentTypeDef {
    pub type_name: String,
    pub interfaces: Vec<String>,
    pub config_schema: serde_json::Value,
    pub implements: Vec<String>,
    pub author: String,
    pub version: String,
}
