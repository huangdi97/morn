//! types — Defines persona data structures, prompt layers, and tuning parameters.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersonaParameters {
    pub temperature: f64,
    pub style: String,
    pub verbosity: f64,
    pub proactiveness: f64,
}

impl Default for PersonaParameters {
    fn default() -> Self {
        PersonaParameters {
            temperature: 0.6,
            style: "professional".into(),
            verbosity: 0.5,
            proactiveness: 0.3,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PromptLayers {
    pub l1_core_identity: String,
    pub l2_skill_instructions: Option<String>,
    pub l3_format_template: Option<String>,
    pub l4_constraints: Option<String>,
    pub l5_conversation_style: Option<String>,
}

impl Default for PromptLayers {
    fn default() -> Self {
        PromptLayers {
            l1_core_identity: "You are a helpful AI assistant.".into(),
            l2_skill_instructions: None,
            l3_format_template: None,
            l4_constraints: None,
            l5_conversation_style: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Persona {
    pub id: String,
    pub name: String,
    pub core_principles: Vec<String>,
    pub decision_framework: Vec<String>,
    pub anti_patterns: Vec<String>,
    pub parameters: PersonaParameters,
    pub prompt_layers: PromptLayers,
    pub communication_style: String,
}

impl Persona {
    pub fn new(id: &str, name: &str) -> Self {
        Persona {
            id: id.to_string(),
            name: name.to_string(),
            core_principles: vec![],
            decision_framework: vec![],
            anti_patterns: vec![],
            parameters: PersonaParameters::default(),
            prompt_layers: PromptLayers::default(),
            communication_style: "professional".into(),
        }
    }

    pub fn build_system_prompt(&self) -> String {
        let mut prompt = self.prompt_layers.l1_core_identity.clone();
        if let Some(ref skills) = self.prompt_layers.l2_skill_instructions {
            prompt.push_str("\n\n");
            prompt.push_str(skills);
        }
        if let Some(ref fmt) = self.prompt_layers.l3_format_template {
            prompt.push_str("\n\n");
            prompt.push_str(fmt);
        }
        if let Some(ref constraints) = self.prompt_layers.l4_constraints {
            prompt.push_str("\n\n");
            prompt.push_str(constraints);
        }
        if let Some(ref style) = self.prompt_layers.l5_conversation_style {
            prompt.push_str("\n\n");
            prompt.push_str(style);
        }
        if !self.core_principles.is_empty() {
            prompt.push_str("\n\nCore Principles:\n");
            for p in &self.core_principles {
                prompt.push_str(&format!("- {}\n", p));
            }
        }
        if !self.anti_patterns.is_empty() {
            prompt.push_str("\n\nAvoid:\n");
            for a in &self.anti_patterns {
                prompt.push_str(&format!("- {}\n", a));
            }
        }
        prompt
    }
}
