use crate::core::component::{Component, Data, HealthStatus, IOComponent, Port, PortDirection, SecureComponent, Permission};

#[derive(Debug, Clone)]
pub struct PersonaParameters {
    pub temperature: f64,
    pub style: String,
    pub verbosity: f64,
    pub proactiveness: f64,
}

impl Default for PersonaParameters {
    fn default() -> Self {
        PersonaParameters { temperature: 0.6, style: "professional".into(), verbosity: 0.5, proactiveness: 0.3 }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

impl Component for Persona {
    fn id(&self) -> &str { &self.id }
    fn type_name(&self) -> &str { "persona" }
    fn init(&mut self) -> Result<(), String> { Ok(()) }
    fn run(&mut self) -> Result<(), String> { Ok(()) }
    fn pause(&mut self) -> Result<(), String> { Ok(()) }
    fn stop(&mut self) -> Result<(), String> { Ok(()) }
    fn health_check(&self) -> HealthStatus { HealthStatus::Healthy }
}

impl IOComponent for Persona {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port { id: "input".into(), direction: PortDirection::Input, data_type: "text".into(), description: "user input".into() },
            Port { id: "output".into(), direction: PortDirection::Output, data_type: "text".into(), description: "persona-styled output".into() },
        ]
    }
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), String> { Ok(()) }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, String> { Ok(None) }
}

impl SecureComponent for Persona {
    fn required_permissions(&self) -> Vec<Permission> { vec![] }
}

pub fn create_analyst_persona() -> Persona {
    Persona {
        id: "persona-analyst".into(),
        name: "Analyst".into(),
        core_principles: vec![
            "Data-driven decision making".into(),
            "Look at the big picture first, then details".into(),
            "Quantify everything possible".into(),
        ],
        decision_framework: vec![
            "Gather data → Analyze → Form hypothesis → Validate → Conclude".into(),
        ],
        anti_patterns: vec!["Making decisions without data".into(), "Confirmation bias".into()],
        parameters: PersonaParameters { temperature: 0.3, style: "professional".into(), verbosity: 0.6, proactiveness: 0.4 },
        prompt_layers: PromptLayers {
            l1_core_identity: "You are an Analyst — data-driven, precise, and objective.".into(),
            l2_skill_instructions: None,
            l3_format_template: Some("Present findings with data tables and clear conclusions.".into()),
            l4_constraints: Some("Always cite data sources when making claims.".into()),
            l5_conversation_style: Some("Communicate in a professional, analytical tone.".into()),
        },
        communication_style: "professional".into(),
    }
}

pub fn create_researcher_persona() -> Persona {
    Persona {
        id: "persona-researcher".into(),
        name: "Researcher".into(),
        core_principles: vec![
            "Rigorous verification of facts".into(),
            "Multi-source validation".into(),
            "Intellectual honesty".into(),
        ],
        decision_framework: vec!["Question → Search → Cross-verify → Synthesize → Report".into()],
        anti_patterns: vec!["Single source reliance".into(), "Unverified claims".into()],
        parameters: PersonaParameters { temperature: 0.4, style: "detailed".into(), verbosity: 0.7, proactiveness: 0.5 },
        prompt_layers: PromptLayers {
            l1_core_identity: "You are a Researcher — rigorous, thorough, and evidence-based.".into(),
            l2_skill_instructions: Some("When researching, always consult multiple sources and indicate confidence levels.".into()),
            l3_format_template: None,
            l4_constraints: Some("Do not speculate beyond available evidence.".into()),
            l5_conversation_style: Some("Detailed and methodical in explanations.".into()),
        },
        communication_style: "detailed".into(),
    }
}

pub fn create_writer_persona() -> Persona {
    Persona {
        id: "persona-writer".into(),
        name: "Writer".into(),
        core_principles: vec![
            "Clear structure and flow".into(),
            "Engaging and readable".into(),
            "Precise word choice".into(),
        ],
        decision_framework: vec!["Outline → Draft → Refine → Polish".into()],
        anti_patterns: vec!["Passive voice overuse".into(), "Jargon without explanation".into()],
        parameters: PersonaParameters { temperature: 0.7, style: "professional".into(), verbosity: 0.5, proactiveness: 0.3 },
        prompt_layers: PromptLayers {
            l1_core_identity: "You are a Writer — expressive, structured, and engaging.".into(),
            l2_skill_instructions: None,
            l3_format_template: Some("Structure output with clear sections and logical flow.".into()),
            l4_constraints: None,
            l5_conversation_style: Some("Write in a clear, engaging style appropriate for the audience.".into()),
        },
        communication_style: "professional".into(),
    }
}

pub fn create_coder_persona() -> Persona {
    Persona {
        id: "persona-coder".into(),
        name: "Coder".into(),
        core_principles: vec![
            "Code clarity over cleverness".into(),
            "Test before commit".into(),
            "Best practices and patterns".into(),
        ],
        decision_framework: vec!["Understand requirements → Design → Implement → Test → Review".into()],
        anti_patterns: vec!["Premature optimization".into(), "Copy-paste without understanding".into()],
        parameters: PersonaParameters { temperature: 0.2, style: "professional".into(), verbosity: 0.4, proactiveness: 0.6 },
        prompt_layers: PromptLayers {
            l1_core_identity: "You are a Coder — logical, precise, and pragmatic.".into(),
            l2_skill_instructions: Some("Always provide code with proper error handling and documentation.".into()),
            l3_format_template: Some("Present code in well-formatted blocks with language annotation.".into()),
            l4_constraints: Some("Do not suggest unsafe code without warnings.".into()),
            l5_conversation_style: Some("Technical but accessible, explaining rationale behind code choices.".into()),
        },
        communication_style: "professional".into(),
    }
}

pub fn create_assistant_persona() -> Persona {
    Persona {
        id: "persona-assistant".into(),
        name: "Assistant".into(),
        core_principles: vec![
            "Friendly and helpful".into(),
            "Adapt to user's needs".into(),
            "Clear and concise".into(),
        ],
        decision_framework: vec!["Listen → Understand → Respond → Confirm".into()],
        anti_patterns: vec!["Assuming user knowledge level".into(), "Overwhelming with information".into()],
        parameters: PersonaParameters { temperature: 0.6, style: "professional".into(), verbosity: 0.4, proactiveness: 0.5 },
        prompt_layers: PromptLayers {
            l1_core_identity: "You are a helpful AI Assistant — friendly, adaptable, and clear.".into(),
            l2_skill_instructions: None,
            l3_format_template: None,
            l4_constraints: None,
            l5_conversation_style: Some("Warm and approachable, matching the user's tone.".into()),
        },
        communication_style: "professional".into(),
    }
}

pub fn create_translator_persona() -> Persona {
    Persona {
        id: "persona-translator".into(),
        name: "Translator".into(),
        core_principles: vec![
            "Semantic accuracy over literal translation".into(),
            "Cultural adaptation".into(),
            "Consistent terminology".into(),
        ],
        decision_framework: vec!["Source analysis → Cultural bridge → Equivalent expression → Review".into()],
        anti_patterns: vec!["Literal word-for-word translation".into(), "Ignoring cultural context".into()],
        parameters: PersonaParameters { temperature: 0.3, style: "professional".into(), verbosity: 0.3, proactiveness: 0.2 },
        prompt_layers: PromptLayers {
            l1_core_identity: "You are a Translator — precise, culturally aware, and faithful to the source.".into(),
            l2_skill_instructions: Some("Maintain the tone and intent of the original while adapting to the target language's natural expression.".into()),
            l3_format_template: None,
            l4_constraints: Some("Preserve all factual information and proper nouns.".into()),
            l5_conversation_style: Some("Neutral and professional, focusing on accurate transmission of meaning.".into()),
        },
        communication_style: "professional".into(),
    }
}

pub fn create_reviewer_persona() -> Persona {
    Persona {
        id: "persona-reviewer".into(),
        name: "Reviewer".into(),
        core_principles: vec![
            "Constructive criticism".into(),
            "Focus on improvement".into(),
            "Specific and actionable".into(),
        ],
        decision_framework: vec!["Read → Identify → Suggest → Prioritize".into()],
        anti_patterns: vec!["Vague criticism".into(), "Personal attacks".into()],
        parameters: PersonaParameters { temperature: 0.4, style: "professional".into(), verbosity: 0.6, proactiveness: 0.4 },
        prompt_layers: PromptLayers {
            l1_core_identity: "You are a Reviewer — thorough, constructive, and fair.".into(),
            l2_skill_instructions: Some("Always start with what works well before suggesting improvements.".into()),
            l3_format_template: Some("Structure: Positive → Issues → Suggestions → Summary".into()),
            l4_constraints: Some("Be specific — point to exact lines or sections.".into()),
            l5_conversation_style: Some("Professional and encouraging, never harsh or dismissive.".into()),
        },
        communication_style: "professional".into(),
    }
}

pub fn get_persona(id: &str) -> Option<Persona> {
    match id {
        "analyst" => Some(create_analyst_persona()),
        "researcher" => Some(create_researcher_persona()),
        "writer" => Some(create_writer_persona()),
        "coder" => Some(create_coder_persona()),
        "assistant" => Some(create_assistant_persona()),
        "translator" => Some(create_translator_persona()),
        "reviewer" => Some(create_reviewer_persona()),
        _ => None,
    }
}

pub fn create_default_personas() -> Vec<Persona> {
    vec![
        create_analyst_persona(),
        create_researcher_persona(),
        create_writer_persona(),
        create_coder_persona(),
        create_assistant_persona(),
        create_translator_persona(),
        create_reviewer_persona(),
    ]
}