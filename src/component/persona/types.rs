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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_parameters_default() {
        let p = PersonaParameters::default();
        assert!((p.temperature - 0.6).abs() < f64::EPSILON);
        assert_eq!(p.style, "professional");
        assert!((p.verbosity - 0.5).abs() < f64::EPSILON);
        assert!((p.proactiveness - 0.3).abs() < f64::EPSILON);
    }

    #[test]
    fn test_prompt_layers_default() {
        let pl = PromptLayers::default();
        assert_eq!(pl.l1_core_identity, "You are a helpful AI assistant.");
        assert!(pl.l2_skill_instructions.is_none());
        assert!(pl.l3_format_template.is_none());
        assert!(pl.l4_constraints.is_none());
        assert!(pl.l5_conversation_style.is_none());
    }

    #[test]
    fn test_persona_new() {
        let p = Persona::new("test-id", "Test Persona");
        assert_eq!(p.id, "test-id");
        assert_eq!(p.name, "Test Persona");
        assert!(p.core_principles.is_empty());
        assert!(p.anti_patterns.is_empty());
        assert_eq!(p.communication_style, "professional");
        assert_eq!(p.parameters.temperature, 0.6);
    }

    #[test]
    fn test_persona_build_system_prompt_core_identity_only() {
        let p = Persona::new("simple", "Simple");
        let prompt = p.build_system_prompt();
        assert_eq!(prompt, "You are a helpful AI assistant.");
    }

    #[test]
    fn test_persona_build_system_prompt_full() {
        let p = Persona {
            id: "full".into(),
            name: "Full".into(),
            core_principles: vec!["Be clear".into(), "Be concise".into()],
            decision_framework: vec!["Analyze → Act".into()],
            anti_patterns: vec!["Haste".into()],
            parameters: PersonaParameters::default(),
            prompt_layers: PromptLayers {
                l1_core_identity: "You are a test persona.".into(),
                l2_skill_instructions: Some("Think step by step.".into()),
                l3_format_template: Some("Output as JSON.".into()),
                l4_constraints: Some("Be safe.".into()),
                l5_conversation_style: Some("Be professional.".into()),
            },
            communication_style: "professional".into(),
        };
        let prompt = p.build_system_prompt();
        assert!(prompt.contains("You are a test persona."));
        assert!(prompt.contains("Think step by step."));
        assert!(prompt.contains("Output as JSON."));
        assert!(prompt.contains("Be safe."));
        assert!(prompt.contains("Be professional."));
        assert!(prompt.contains("Core Principles:"));
        assert!(prompt.contains("Avoid:"));
        assert!(prompt.contains("Be clear"));
        assert!(prompt.contains("Haste"));
    }

    #[test]
    fn test_merge_strategy_label() {
        assert_eq!(MergeStrategy::MajorityVote.label(), "majority_vote");
        assert_eq!(MergeStrategy::WeightedAverage.label(), "weighted_average");
        assert_eq!(MergeStrategy::Sequential.label(), "sequential");
        assert_eq!(MergeStrategy::Debate.label(), "debate");
    }

    #[test]
    fn test_composite_persona_default() {
        let cp = CompositePersona::default();
        assert!(cp.primary.is_empty());
        assert!(cp.secondary.is_none());
        assert!((cp.blend_ratio - 0.5).abs() < f32::EPSILON);
        assert!(cp.persona_ids.is_empty());
        assert_eq!(cp.merge_strategy, MergeStrategy::WeightedAverage);
    }

    #[test]
    fn test_composite_persona_new() {
        let cp = CompositePersona::new(vec!["p1".into(), "p2".into()], MergeStrategy::MajorityVote);
        assert_eq!(cp.primary, "p1");
        assert_eq!(cp.secondary, Some("p2".into()));
        assert_eq!(cp.persona_ids.len(), 2);
        assert_eq!(cp.merge_strategy, MergeStrategy::MajorityVote);
    }

    #[test]
    fn test_merge_responses_empty() {
        let result = merge_responses(&MergeStrategy::Sequential, &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_merge_responses_sequential() {
        let outputs = vec![
            PersonaOutput {
                persona_id: "a".into(),
                response: "First".into(),
                confidence: 0.5,
            },
            PersonaOutput {
                persona_id: "b".into(),
                response: "Second".into(),
                confidence: 0.8,
            },
        ];
        let result = merge_responses(&MergeStrategy::Sequential, &outputs);
        assert_eq!(result, "First\n---\nSecond");
    }

    #[test]
    fn test_merge_responses_majority_vote() {
        let outputs = vec![
            PersonaOutput {
                persona_id: "a".into(),
                response: "yes".into(),
                confidence: 0.5,
            },
            PersonaOutput {
                persona_id: "b".into(),
                response: "yes".into(),
                confidence: 0.6,
            },
            PersonaOutput {
                persona_id: "c".into(),
                response: "no".into(),
                confidence: 0.9,
            },
        ];
        let result = merge_responses(&MergeStrategy::MajorityVote, &outputs);
        assert_eq!(result, "yes");
    }

    #[test]
    fn test_merge_responses_weighted_average() {
        let outputs = vec![
            PersonaOutput {
                persona_id: "a".into(),
                response: "low".into(),
                confidence: 0.1,
            },
            PersonaOutput {
                persona_id: "b".into(),
                response: "high".into(),
                confidence: 0.9,
            },
        ];
        let result = merge_responses(&MergeStrategy::WeightedAverage, &outputs);
        assert_eq!(result, "high");
    }

    #[test]
    fn test_merge_responses_debate() {
        let outputs = vec![PersonaOutput {
            persona_id: "p1".into(),
            response: "arg1".into(),
            confidence: 0.7,
        }];
        let result = merge_responses(&MergeStrategy::Debate, &outputs);
        assert!(result.contains("Debate Summary"));
        assert!(result.contains("p1"));
    }

    #[test]
    fn test_persona_parameters_serialize() {
        let p = PersonaParameters::default();
        let json = serde_json::to_string(&p).unwrap();
        assert!(json.contains("temperature"));
        assert!(json.contains("style"));
    }

    #[test]
    fn test_persona_parameters_deserialize() {
        let json = r#"{"temperature":0.5,"style":"casual","verbosity":0.7,"proactiveness":0.4}"#;
        let p: PersonaParameters = serde_json::from_str(json).unwrap();
        assert!((p.temperature - 0.5).abs() < f64::EPSILON);
        assert_eq!(p.style, "casual");
    }

    #[test]
    fn test_composite_persona_serialize() {
        let cp = CompositePersona::default();
        let json = serde_json::to_string(&cp).unwrap();
        assert!(json.contains("blend_ratio"));
        assert!(json.contains("merge_strategy"));
    }

    #[test]
    fn test_merge_majority_vote_empty() {
        let result = merge_majority_vote(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_merge_weighted_average_zero_confidence() {
        let outputs = vec![
            PersonaOutput {
                persona_id: "a".into(),
                response: "r1".into(),
                confidence: 0.0,
            },
            PersonaOutput {
                persona_id: "b".into(),
                response: "r2".into(),
                confidence: 0.0,
            },
        ];
        let result = merge_weighted_average(&outputs);
        assert_eq!(result, "r1\n---\nr2");
    }

    #[test]
    fn test_merge_debate_empty() {
        let result = merge_debate(&[]);
        assert!(result.is_empty());
    }
}
