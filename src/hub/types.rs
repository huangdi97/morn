#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ListingType {
    Tool,
    Knowledge,
    Skill,
    Persona,
    Agent,
    Workflow,
    TeamTemplate,
    ComponentTypeDef,
}

impl ListingType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ListingType::Tool => "tool",
            ListingType::Knowledge => "knowledge",
            ListingType::Skill => "skill",
            ListingType::Persona => "persona",
            ListingType::Agent => "agent",
            ListingType::Workflow => "workflow",
            ListingType::TeamTemplate => "team_template",
            ListingType::ComponentTypeDef => "component_type_def",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Listing {
    pub id: String,
    pub item_type: String,
    pub name: String,
    pub description: String,
    pub price: Option<f64>,
    #[serde(default)]
    pub price_model: String,
    pub author: String,
    pub rating: f64,
    pub downloads: u64,
    pub created_at: String,
    pub version: String,
    pub screenshots: String,
    pub category: String,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub verified: bool,
    #[serde(default)]
    pub updated_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Transaction {
    pub id: String,
    pub listing_id: String,
    pub buyer: String,
    pub amount: f64,
    pub timestamp: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct License {
    pub id: String,
    pub listing_id: String,
    pub user_id: String,
    pub granted_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentVersion {
    pub id: String,
    pub listing_id: String,
    pub version: String,
    pub data_json: String,
    pub changelog: String,
    pub created_at: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Review {
    pub id: String,
    pub listing_id: String,
    pub user_id: String,
    pub rating: u8,
    pub comment: String,
    pub created_at: String,
}
