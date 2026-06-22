use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityDef {
    pub id: String,
    pub category: String,
    pub name: String,
    pub description: String,
    pub config_schema: serde_json::Value,
    pub entry: String,
    pub version: String,
    pub author: String,
}
