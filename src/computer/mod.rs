pub mod fs_ops;
pub mod app_ops;
pub mod sys_ops;
pub mod desktop_ops;
pub mod browser_ops;
pub mod perception;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum SecurityLevel {
    L1Sandbox,
    L2Local,
    L3System,
}

impl SecurityLevel {
    pub fn as_str(&self) -> &'static str {
        match self {
            SecurityLevel::L1Sandbox => "sandbox",
            SecurityLevel::L2Local => "local",
            SecurityLevel::L3System => "system",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ComputerOpResult {
    pub success: bool,
    pub data: String,
    pub security_level: String,
    pub approval_required: bool,
}
