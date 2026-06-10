//! types — Defines persona data structures, prompt layers, and tuning parameters.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersonaParameters {
    pub temperature: f64,
    pub style: String,
    pub verbosity: f64,
    pub proactiveness: f64,
}

pub type PersonaRef = String;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
pub enum MergeStrategy {
    MajorityVote,
    WeightedAverage,
    Sequential,
    Debate,
}

impl MergeStrategy {
    pub fn label(&self) -> &'static str {
        match self {
            MergeStrategy::MajorityVote => "majority_vote",
            MergeStrategy::WeightedAverage => "weighted_average",
            MergeStrategy::Sequential => "sequential",
            MergeStrategy::Debate => "debate",
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CompositePersona {
    pub primary: PersonaRef,
    pub secondary: Option<PersonaRef>,
    pub blend_ratio: f32,
    pub persona_ids: Vec<String>,
    pub merge_strategy: MergeStrategy,
}

impl Default for CompositePersona {
    fn default() -> Self {
        CompositePersona {
            primary: String::new(),
            secondary: None,
            blend_ratio: 0.5,
            persona_ids: Vec::new(),
            merge_strategy: MergeStrategy::WeightedAverage,
        }
    }
}

impl CompositePersona {
    pub fn new(persona_ids: Vec<String>, merge_strategy: MergeStrategy) -> Self {
        let primary = persona_ids.first().cloned().unwrap_or_default();
        let secondary = persona_ids.get(1).cloned();
        CompositePersona {
            primary,
            secondary,
            blend_ratio: 0.5,
            persona_ids,
            merge_strategy,
        }
    }
}

pub struct PersonaOutput {
    pub persona_id: String,
    pub response: String,
    pub confidence: f64,
}

pub fn merge_responses(strategy: &MergeStrategy, outputs: &[PersonaOutput]) -> String {
    match strategy {
        MergeStrategy::MajorityVote => merge_majority_vote(outputs),
        MergeStrategy::WeightedAverage => merge_weighted_average(outputs),
        MergeStrategy::Sequential => merge_sequential(outputs),
        MergeStrategy::Debate => merge_debate(outputs),
    }
}

fn merge_majority_vote(outputs: &[PersonaOutput]) -> String {
    if outputs.is_empty() {
        return String::new();
    }
    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for o in outputs {
        *counts.entry(&o.response).or_insert(0) += 1;
    }
    counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(resp, _)| resp.to_string())
        .unwrap_or_else(|| outputs[0].response.clone())
}

fn merge_weighted_average(outputs: &[PersonaOutput]) -> String {
    if outputs.is_empty() {
        return String::new();
    }
    let total_conf: f64 = outputs.iter().map(|o| o.confidence).sum();
    if total_conf <= 0.0 {
        return outputs
            .iter()
            .map(|o| o.response.clone())
            .collect::<Vec<_>>()
            .join("\n---\n");
    }
    let best = outputs
        .iter()
        .max_by(|a, b| {
            a.confidence
                .partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|o| o.response.clone())
        .unwrap_or_default();
    best
}

fn merge_sequential(outputs: &[PersonaOutput]) -> String {
    outputs
        .iter()
        .map(|o| o.response.clone())
        .collect::<Vec<_>>()
        .join("\n---\n")
}

fn merge_debate(outputs: &[PersonaOutput]) -> String {
    if outputs.is_empty() {
        return String::new();
    }
    let mut debate = String::from("=== Debate Summary ===\n");
    for (i, o) in outputs.iter().enumerate() {
        debate.push_str(&format!(
            "--- Participant {} ({}): confidence={:.2} ---\n{}\n",
            i + 1,
            o.persona_id,
            o.confidence,
            o.response
        ));
    }
    debate
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
