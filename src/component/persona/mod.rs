use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

pub mod presets_creative;
pub mod presets_general;
pub mod presets_industry;
pub mod presets_tech;

pub use presets_creative::*;
pub use presets_general::*;
pub use presets_industry::*;
pub use presets_tech::*;

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

impl Component for Persona {
    fn id(&self) -> &str {
        &self.id
    }
    fn type_name(&self) -> &str {
        "persona"
    }
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn run(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn pause(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn stop(&mut self) -> Result<(), String> {
        Ok(())
    }
    fn health_check(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

impl IOComponent for Persona {
    fn ports(&self) -> Vec<Port> {
        vec![
            Port {
                id: "input".into(),
                direction: PortDirection::Input,
                data_type: "text".into(),
                description: "user input".into(),
            },
            Port {
                id: "output".into(),
                direction: PortDirection::Output,
                data_type: "text".into(),
                description: "persona-styled output".into(),
            },
        ]
    }
    fn send(&mut self, _port: &str, _data: Data) -> Result<(), String> {
        Ok(())
    }
    fn recv(&mut self, _port: &str) -> Result<Option<Data>, String> {
        Ok(None)
    }
}

impl SecureComponent for Persona {
    fn required_permissions(&self) -> Vec<Permission> {
        vec![]
    }
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
        anti_patterns: vec![
            "Making decisions without data".into(),
            "Confirmation bias".into(),
        ],
        parameters: PersonaParameters {
            temperature: 0.3,
            style: "professional".into(),
            verbosity: 0.6,
            proactiveness: 0.4,
        },
        prompt_layers: PromptLayers {
            l1_core_identity: "You are an Analyst — data-driven, precise, and objective.".into(),
            l2_skill_instructions: None,
            l3_format_template: Some(
                "Present findings with data tables and clear conclusions.".into(),
            ),
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
        parameters: PersonaParameters {
            temperature: 0.4,
            style: "detailed".into(),
            verbosity: 0.7,
            proactiveness: 0.5,
        },
        prompt_layers: PromptLayers {
            l1_core_identity: "You are a Researcher — rigorous, thorough, and evidence-based."
                .into(),
            l2_skill_instructions: Some(
                "When researching, always consult multiple sources and indicate confidence levels."
                    .into(),
            ),
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
        anti_patterns: vec![
            "Passive voice overuse".into(),
            "Jargon without explanation".into(),
        ],
        parameters: PersonaParameters {
            temperature: 0.7,
            style: "professional".into(),
            verbosity: 0.5,
            proactiveness: 0.3,
        },
        prompt_layers: PromptLayers {
            l1_core_identity: "You are a Writer — expressive, structured, and engaging.".into(),
            l2_skill_instructions: None,
            l3_format_template: Some(
                "Structure output with clear sections and logical flow.".into(),
            ),
            l4_constraints: None,
            l5_conversation_style: Some(
                "Write in a clear, engaging style appropriate for the audience.".into(),
            ),
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
        decision_framework: vec![
            "Understand requirements → Design → Implement → Test → Review".into(),
        ],
        anti_patterns: vec![
            "Premature optimization".into(),
            "Copy-paste without understanding".into(),
        ],
        parameters: PersonaParameters {
            temperature: 0.2,
            style: "professional".into(),
            verbosity: 0.4,
            proactiveness: 0.6,
        },
        prompt_layers: PromptLayers {
            l1_core_identity: "You are a Coder — logical, precise, and pragmatic.".into(),
            l2_skill_instructions: Some(
                "Always provide code with proper error handling and documentation.".into(),
            ),
            l3_format_template: Some(
                "Present code in well-formatted blocks with language annotation.".into(),
            ),
            l4_constraints: Some("Do not suggest unsafe code without warnings.".into()),
            l5_conversation_style: Some(
                "Technical but accessible, explaining rationale behind code choices.".into(),
            ),
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
        anti_patterns: vec![
            "Assuming user knowledge level".into(),
            "Overwhelming with information".into(),
        ],
        parameters: PersonaParameters {
            temperature: 0.6,
            style: "professional".into(),
            verbosity: 0.4,
            proactiveness: 0.5,
        },
        prompt_layers: PromptLayers {
            l1_core_identity: "You are a helpful AI Assistant — friendly, adaptable, and clear."
                .into(),
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
        parameters: PersonaParameters {
            temperature: 0.4,
            style: "professional".into(),
            verbosity: 0.6,
            proactiveness: 0.4,
        },
        prompt_layers: PromptLayers {
            l1_core_identity: "You are a Reviewer — thorough, constructive, and fair.".into(),
            l2_skill_instructions: Some(
                "Always start with what works well before suggesting improvements.".into(),
            ),
            l3_format_template: Some("Structure: Positive → Issues → Suggestions → Summary".into()),
            l4_constraints: Some("Be specific — point to exact lines or sections.".into()),
            l5_conversation_style: Some(
                "Professional and encouraging, never harsh or dismissive.".into(),
            ),
        },
        communication_style: "professional".into(),
    }
}

pub fn create_cs_agent_persona() -> Persona {
    Persona {
        id: "persona-cs-agent".into(),
        name: "客服".into(),
        core_principles: vec![
            "态度友善、有同理心".into(),
            "耐心倾听用户需求".into(),
            "准确解决问题".into(),
        ],
        decision_framework: vec!["识别问题 → 查知识库 → 给出方案 → 确认解决".into()],
        anti_patterns: vec!["推诿责任".into(), "机械重复回答".into()],
        parameters: PersonaParameters {
            temperature: 0.5,
            style: "friendly".into(),
            verbosity: 0.5,
            proactiveness: 0.6,
        },
        prompt_layers: PromptLayers {
            l1_core_identity: "你是一名专业客服。你有耐心、有同理心。".into(),
            l2_skill_instructions: Some("客服流程：识别问题→查知识库→给出方案→确认解决".into()),
            l3_format_template: None,
            l4_constraints: Some("态度友善、不推诿、不机械重复".into()),
            l5_conversation_style: Some("温暖友好的语气，让用户感到被重视和理解。".into()),
        },
        communication_style: "friendly".into(),
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
        "cs_agent" => Some(create_cs_agent_persona()),
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
        create_cs_agent_persona(),
    ]
}

pub fn list_preset_personas() -> Vec<std::collections::HashMap<String, String>> {
    let presets = vec![
        ("preset-analyst", "数据分析师", "数据驱动的专业分析"),
        ("preset-researcher", "研究员", "多源信息收集与交叉验证"),
        ("preset-writer", "写作者", "将复杂信息转化为清晰文字"),
        ("preset-coder", "程序员", "简洁可维护的软件工程"),
        ("preset-translator", "翻译官", "信达雅的专业翻译"),
        ("preset-assistant", "系统管家", "高效随叫随到的 AI 管家"),
        ("preset-reviewer", "审查员", "严谨的代码/文档审查"),
        ("preset-cs-agent", "客服", "有耐心有同理心的客户服务"),
        (
            "preset-investment",
            "投资分析师",
            "风险与收益平衡的专业分析",
        ),
        ("preset-medical", "医疗顾问", "循证医学健康信息"),
        ("preset-legal", "法律顾问", "法律信息参考"),
        ("preset-tutor", "教育导师", "因材施教的启发式教学"),
        ("preset-marketing", "营销策划", "创意与实效并重"),
        ("preset-hr", "HR 顾问", "公平公正的人力资源管理"),
        ("preset-pm", "项目经理", "目标导向的项目管理"),
        ("preset-product", "产品经理", "用户价值驱动的产品设计"),
        ("preset-ui-designer", "UI 设计师", "美观易用的界面设计"),
        ("preset-data-engineer", "数据工程师", "数据管道和平台构建"),
        ("preset-devops", "运维工程师", "可靠系统和自动化运维"),
        ("preset-security", "安全分析师", "网络安全威胁防护"),
        ("preset-qa", "测试工程师", "软件质量保障"),
        ("preset-tech-writer", "技术文档工程师", "复杂技术文档化"),
        ("preset-social-media", "社交媒体经理", "社媒策略与运营"),
        ("preset-copywriter", "文案策划", "用文字打动人心"),
        ("preset-editor", "编辑", "内容审校与质量优化"),
        ("preset-journalist", "记者", "客观公正的新闻报道"),
        ("preset-philosopher", "哲思顾问", "深度思考与分析"),
        ("preset-psychologist", "心理咨询师", "倾听与心理支持"),
        ("preset-career-coach", "职业规划师", "职业发展与规划"),
        ("preset-travel-guide", "旅行顾问", "精彩旅行规划"),
        ("preset-language-tutor", "语言教师", "外语教学与文化交流"),
        ("preset-math-tutor", "数学教师", "直观的数学教学"),
        ("preset-science-tutor", "科学教师", "实验与科学原理"),
        ("preset-life-coach", "人生教练", "潜能发掘与目标实现"),
        ("preset-fitness", "健身教练", "科学安全训练"),
        ("preset-chef", "美食顾问", "食谱设计与烹饪"),
        ("preset-music", "音乐导师", "音乐教学与艺术指导"),
        ("preset-history", "历史教师", "历史教育与文明理解"),
        ("preset-startup", "创业顾问", "精益创业指导"),
        ("preset-architect", "系统架构师", "可扩展系统设计"),
        ("preset-seo", "SEO 专家", "搜索引擎排名优化"),
        ("preset-business-analyst", "商业分析师", "业务增长洞察"),
        ("preset-financial-analyst", "财务分析师", "财务数据解读"),
        ("preset-trainer", "培训师", "高效培训课程"),
        ("preset-content-moderator", "内容审核", "内容合规判定"),
        ("preset-social-assistant", "社交助手", "日常社交沟通协助"),
        ("preset-game-designer", "游戏策划", "游戏设计与玩法"),
        ("preset-video-editor", "视频编辑", "视频剪辑与叙事"),
        ("preset-research-assistant", "科研助手", "学术研究协助"),
        ("preset-academic-writer", "学术写作", "学术论文撰写"),
        ("preset-debate-coach", "辩论教练", "逻辑思辨训练"),
        ("preset-negotiator", "谈判专家", "商务谈判策略"),
    ];
    presets
        .into_iter()
        .map(|(id, name, desc)| {
            let mut map = std::collections::HashMap::new();
            map.insert("id".into(), id.into());
            map.insert("name".into(), name.into());
            map.insert("description".into(), desc.into());
            map
        })
        .collect()
}

pub fn get_preset_persona(name: &str) -> Option<Persona> {
    match name {
        "preset-analyst" => Some(preset_analyst()),
        "preset-researcher" => Some(preset_researcher()),
        "preset-writer" => Some(preset_writer()),
        "preset-coder" => Some(preset_coder()),
        "preset-translator" => Some(preset_translator()),
        "preset-assistant" => Some(preset_assistant()),
        "preset-reviewer" => Some(preset_reviewer()),
        "preset-cs-agent" => Some(preset_cs_agent()),
        "preset-investment" => Some(preset_investment()),
        "preset-medical" => Some(preset_medical()),
        "preset-legal" => Some(preset_legal()),
        "preset-tutor" => Some(preset_tutor()),
        "preset-marketing" => Some(preset_marketing()),
        "preset-hr" => Some(preset_hr()),
        "preset-pm" => Some(preset_pm()),
        "preset-product" => Some(preset_product()),
        "preset-ui-designer" => Some(preset_ui_designer()),
        "preset-data-engineer" => Some(preset_data_engineer()),
        "preset-devops" => Some(preset_devops()),
        "preset-security" => Some(preset_security()),
        "preset-qa" => Some(preset_qa()),
        "preset-tech-writer" => Some(preset_tech_writer()),
        "preset-social-media" => Some(preset_social_media()),
        "preset-copywriter" => Some(preset_copywriter()),
        "preset-editor" => Some(preset_editor()),
        "preset-journalist" => Some(preset_journalist()),
        "preset-philosopher" => Some(preset_philosopher()),
        "preset-psychologist" => Some(preset_psychologist()),
        "preset-career-coach" => Some(preset_career_coach()),
        "preset-travel-guide" => Some(preset_travel_guide()),
        "preset-language-tutor" => Some(preset_language_tutor()),
        "preset-math-tutor" => Some(preset_math_tutor()),
        "preset-science-tutor" => Some(preset_science_tutor()),
        "preset-life-coach" => Some(preset_life_coach()),
        "preset-fitness" => Some(preset_fitness()),
        "preset-chef" => Some(preset_chef()),
        "preset-music" => Some(preset_music()),
        "preset-history" => Some(preset_history()),
        "preset-startup" => Some(preset_startup()),
        "preset-architect" => Some(preset_architect()),
        "preset-seo" => Some(preset_seo()),
        "preset-business-analyst" => Some(preset_business_analyst()),
        "preset-financial-analyst" => Some(preset_financial_analyst()),
        "preset-trainer" => Some(preset_trainer()),
        "preset-content-moderator" => Some(preset_content_moderator()),
        "preset-social-assistant" => Some(preset_social_assistant()),
        "preset-game-designer" => Some(preset_game_designer()),
        "preset-video-editor" => Some(preset_video_editor()),
        "preset-research-assistant" => Some(preset_research_assistant()),
        "preset-academic-writer" => Some(preset_academic_writer()),
        "preset-debate-coach" => Some(preset_debate_coach()),
        "preset-negotiator" => Some(preset_negotiator()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_persona_create() {
        let p = Persona::new("test-1", "TestBot");
        assert_eq!(p.id, "test-1");
        assert_eq!(p.name, "TestBot");
        assert!(p.core_principles.is_empty());
        assert!(p.anti_patterns.is_empty());
    }

    #[test]
    fn test_persona_default() {
        let params = PersonaParameters::default();
        assert_eq!(params.temperature, 0.6);
        assert_eq!(params.style, "professional");
        assert_eq!(params.verbosity, 0.5);
        assert_eq!(params.proactiveness, 0.3);

        let layers = PromptLayers::default();
        assert_eq!(layers.l1_core_identity, "You are a helpful AI assistant.");
        assert!(layers.l2_skill_instructions.is_none());
    }

    #[test]
    fn test_persona_to_system_prompt() {
        let persona = create_analyst_persona();
        let prompt = persona.build_system_prompt();
        assert!(prompt.contains("Analyst"));
        assert!(prompt.contains("data-driven"));

        let coder = create_coder_persona();
        let coder_prompt = coder.build_system_prompt();
        assert!(coder_prompt.contains("Coder"));
        assert!(coder_prompt.contains("error handling"));
    }

    #[test]
    fn test_cs_agent_persona() {
        let p = create_cs_agent_persona();
        assert_eq!(p.id, "persona-cs-agent");
        assert!(p.parameters.temperature == 0.5);
        assert_eq!(p.parameters.style, "friendly");
    }

    #[test]
    fn test_get_persona_cs_agent() {
        let p = get_persona("cs_agent");
        assert!(p.is_some());
        assert_eq!(p.unwrap().name, "客服");
    }

    #[test]
    fn test_default_personas_count() {
        let personas = create_default_personas();
        assert_eq!(personas.len(), 8);
    }

    #[test]
    fn test_preset_functions() {
        let presets = vec![
            preset_analyst(),
            preset_researcher(),
            preset_writer(),
            preset_coder(),
            preset_translator(),
            preset_assistant(),
            preset_reviewer(),
            preset_cs_agent(),
            preset_investment(),
            preset_medical(),
            preset_legal(),
            preset_tutor(),
            preset_marketing(),
            preset_hr(),
            preset_pm(),
            preset_product(),
            preset_ui_designer(),
            preset_data_engineer(),
            preset_devops(),
            preset_security(),
            preset_qa(),
            preset_tech_writer(),
            preset_social_media(),
            preset_copywriter(),
            preset_editor(),
            preset_journalist(),
            preset_philosopher(),
            preset_psychologist(),
            preset_career_coach(),
            preset_travel_guide(),
            preset_language_tutor(),
            preset_math_tutor(),
            preset_science_tutor(),
            preset_life_coach(),
            preset_fitness(),
            preset_chef(),
            preset_music(),
            preset_history(),
            preset_startup(),
            preset_architect(),
            preset_seo(),
            preset_business_analyst(),
            preset_financial_analyst(),
            preset_trainer(),
            preset_content_moderator(),
            preset_social_assistant(),
            preset_game_designer(),
            preset_video_editor(),
            preset_research_assistant(),
            preset_academic_writer(),
            preset_debate_coach(),
            preset_negotiator(),
        ];
        assert_eq!(presets.len(), 52);
        assert!(presets[0].id.contains("analyst"));
        assert!(presets[7].id.contains("cs-agent"));
        assert!(presets[51].id.contains("negotiator"));
    }

    #[test]
    fn test_list_preset_personas() {
        let list = list_preset_personas();
        assert_eq!(list.len(), 52);
        assert!(list[0].contains_key("id"));
        assert!(list[0].contains_key("name"));
        assert!(list[0].contains_key("description"));
    }

    #[test]
    fn test_get_preset_persona() {
        let p = get_preset_persona("preset-analyst");
        assert!(p.is_some());
        let p = p.unwrap();
        assert_eq!(p.name, "数据分析师");
        assert_eq!(p.parameters.temperature, 0.3);
        assert_eq!(p.parameters.style, "professional");

        let p = get_preset_persona("preset-negotiator");
        assert!(p.is_some());
        let p = p.unwrap();
        assert_eq!(p.name, "谈判专家");

        let invalid = get_preset_persona("nonexistent");
        assert!(invalid.is_none());
    }
}
