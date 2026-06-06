use crate::core::component::{
    Component, Data, HealthStatus, IOComponent, Permission, Port, PortDirection, SecureComponent,
};

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

pub mod presets_creative;
pub mod presets_general;
pub mod presets_industry;
pub mod presets_tech;
pub use presets_creative::*;
pub use presets_general::*;
pub use presets_industry::*;
pub use presets_tech::*;

pub fn create_analyst_persona() -> Persona {
    preset_analyst()
}

pub fn create_researcher_persona() -> Persona {
    preset_researcher()
}

pub fn create_writer_persona() -> Persona {
    preset_writer()
}

pub fn create_coder_persona() -> Persona {
    preset_coder()
}

pub fn create_translator_persona() -> Persona {
    preset_translator()
}

pub fn create_assistant_persona() -> Persona {
    preset_assistant()
}

pub fn create_reviewer_persona() -> Persona {
    preset_reviewer()
}

pub fn create_cs_agent_persona() -> Persona {
    preset_cs_agent()
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
        ("preset-analyst", "数据分析师", "数据驱动的深度分析"),
        ("preset-researcher", "研究员", "多源信息调研与验证"),
        ("preset-writer", "写作者", "清晰流畅的内容创作"),
        ("preset-coder", "程序员", "高质量代码开发"),
        ("preset-translator", "翻译官", "多语言翻译与文化适配"),
        ("preset-assistant", "助手", "通用AI助手"),
        ("preset-reviewer", "审核专家", "建设性内容审核"),
        ("preset-cs-agent", "客服", "专业客户服务"),
        ("preset-investment", "投资顾问", "资产配置与风险管理"),
        ("preset-medical", "医疗顾问", "循证医学知识"),
        ("preset-legal", "法律顾问", "法律咨询与分析"),
        ("preset-tutor", "教师", "因材施教"),
        ("preset-marketing", "营销专家", "品牌策略与推广"),
        ("preset-hr", "HR专家", "人才招聘与发展"),
        ("preset-pm", "项目经理", "项目管理与执行"),
        ("preset-product", "产品经理", "产品规划与设计"),
        ("preset-ui-designer", "UI设计师", "用户界面设计"),
        ("preset-data-engineer", "数据工程师", "数据管道构建"),
        ("preset-devops", "DevOps工程师", "自动化部署运维"),
        ("preset-security", "安全工程师", "系统安全防护"),
        ("preset-qa", "测试工程师", "质量保障与测试"),
        ("preset-tech-writer", "技术文档工程师", "技术文档撰写"),
        ("preset-social-media", "社交媒体运营", "社媒内容策划"),
        ("preset-copywriter", "文案策划", "创意文案撰写"),
        ("preset-editor", "编辑", "内容审校"),
        ("preset-journalist", "记者", "新闻报道"),
        ("preset-philosopher", "哲学家", "深度思辨分析"),
        ("preset-psychologist", "心理咨询师", "心理支持与引导"),
        ("preset-career-coach", "职业规划师", "职业发展指导"),
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
        ("preset-seo", "SEO专家", "搜索引擎排名优化"),
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
        let persona = preset_analyst();
        let prompt = persona.build_system_prompt();
        assert!(prompt.contains("数据分析师"));
        assert!(prompt.contains("数据驱动"));

        let coder = preset_coder();
        let coder_prompt = coder.build_system_prompt();
        assert!(coder_prompt.contains("软件工程师"));
    }

    #[test]
    fn test_cs_agent_persona() {
        let p = preset_cs_agent();
        assert_eq!(p.id, "preset-cs-agent");
        assert!(p.parameters.temperature == 0.5);
        assert_eq!(p.parameters.style, "friendly");
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
    fn test_get_preset_persona_analyst() {
        let p = preset_analyst();
        assert_eq!(p.name, "数据分析师");
        assert_eq!(p.parameters.temperature, 0.3);
        assert_eq!(p.parameters.style, "professional");
    }

    #[test]
    fn test_get_preset_persona_negotiator() {
        let p = preset_negotiator();
        assert_eq!(p.name, "谈判专家");
    }
}
