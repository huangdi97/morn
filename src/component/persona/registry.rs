//! registry — Maintains persona registration, lookup, and default selection.
use super::*;

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

pub fn compose(id1: &str, id2: &str, ratio: f32) -> Result<Persona, String> {
    let primary = lookup_persona(id1).ok_or_else(|| format!("Persona not found: {}", id1))?;
    let secondary = lookup_persona(id2).ok_or_else(|| format!("Persona not found: {}", id2))?;
    let primary_weight = ratio.clamp(0.0, 1.0) as f64;
    let secondary_weight = 1.0 - primary_weight;

    let mut composed = primary.clone();
    composed.id = format!("composite-{}-{}", primary.id, secondary.id);
    composed.name = format!("{} + {}", primary.name, secondary.name);
    composed.parameters.temperature = weighted(
        primary.parameters.temperature,
        secondary.parameters.temperature,
        primary_weight,
        secondary_weight,
    );
    composed.parameters.verbosity = weighted(
        primary.parameters.verbosity,
        secondary.parameters.verbosity,
        primary_weight,
        secondary_weight,
    );
    composed.parameters.proactiveness = weighted(
        primary.parameters.proactiveness,
        secondary.parameters.proactiveness,
        primary_weight,
        secondary_weight,
    );
    composed.core_principles = merge_unique(&primary.core_principles, &secondary.core_principles);
    composed.decision_framework =
        merge_unique(&primary.decision_framework, &secondary.decision_framework);
    composed.anti_patterns = merge_unique(&primary.anti_patterns, &secondary.anti_patterns);
    Ok(composed)
}

fn lookup_persona(id: &str) -> Option<Persona> {
    get_preset_persona(id).or_else(|| get_persona(id))
}

fn weighted(primary: f64, secondary: f64, primary_weight: f64, secondary_weight: f64) -> f64 {
    primary * primary_weight + secondary * secondary_weight
}

fn merge_unique(primary: &[String], secondary: &[String]) -> Vec<String> {
    let mut merged = Vec::new();
    for value in primary.iter().chain(secondary.iter()) {
        if !merged.contains(value) {
            merged.push(value.clone());
        }
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_persona_known_ids() {
        let analyst = get_persona("analyst");
        assert!(analyst.is_some());
        assert_eq!(analyst.unwrap().id, "persona-analyst");

        let coder = get_persona("coder");
        assert!(coder.is_some());
        assert_eq!(coder.unwrap().id, "persona-coder");
    }

    #[test]
    fn test_get_persona_unknown_id() {
        assert!(get_persona("nonexistent").is_none());
    }

    #[test]
    fn test_get_persona_all_ids() {
        for id in &["analyst", "researcher", "writer", "coder", "assistant", "translator", "reviewer", "cs_agent"] {
            let persona = get_persona(id);
            assert!(persona.is_some(), "Persona {} should exist", id);
        }
    }

    #[test]
    fn test_create_default_personas_count() {
        let personas = create_default_personas();
        assert_eq!(personas.len(), 8);
    }

    #[test]
    fn test_create_default_personas_unique_ids() {
        let personas = create_default_personas();
        let mut ids: Vec<&str> = personas.iter().map(|p| p.id.as_str()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), personas.len());
    }

    #[test]
    fn test_list_preset_personas_count() {
        let presets = list_preset_personas();
        assert!(presets.len() > 40);
    }

    #[test]
    fn test_list_preset_personas_has_expected_fields() {
        let presets = list_preset_personas();
        for map in &presets {
            assert!(map.contains_key("id"));
            assert!(map.contains_key("name"));
            assert!(map.contains_key("description"));
        }
    }

    #[test]
    fn test_get_preset_persona_known() {
        let p = get_preset_persona("preset-analyst");
        assert!(p.is_some());
        assert_eq!(p.unwrap().id, "preset-analyst");
    }

    #[test]
    fn test_get_preset_persona_unknown() {
        assert!(get_preset_persona("preset-nonexistent").is_none());
    }

    #[test]
    fn test_compose_returns_result() {
        let result = compose("analyst", "coder", 0.7);
        assert!(result.is_ok());
        let persona = result.unwrap();
        assert!(persona.id.starts_with("composite-"));
        assert!(persona.name.contains("+"));
    }

    #[test]
    fn test_compose_with_unknown_primary() {
        let result = compose("nonexistent", "coder", 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_compose_with_unknown_secondary() {
        let result = compose("analyst", "nonexistent", 0.5);
        assert!(result.is_err());
    }

    #[test]
    fn test_compose_clamps_ratio() {
        let p = compose("analyst", "coder", 2.0).unwrap();
        assert!(p.parameters.temperature > 0.0);
        let p = compose("analyst", "coder", -1.0).unwrap();
        assert!(p.parameters.temperature > 0.0);
    }

    #[test]
    fn test_compose_merges_principles() {
        let p = compose("analyst", "coder", 0.5).unwrap();
        assert!(!p.core_principles.is_empty());
        assert!(!p.decision_framework.is_empty());
        assert!(!p.anti_patterns.is_empty());
    }

    #[test]
    fn test_weighted() {
        assert!((super::weighted(0.5, 0.7, 0.5, 0.5) - 0.6).abs() < f64::EPSILON);
        assert!((super::weighted(1.0, 0.0, 1.0, 0.0) - 1.0).abs() < f64::EPSILON);
        assert!((super::weighted(0.0, 1.0, 0.0, 1.0) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_merge_unique() {
        let a = vec!["x".into(), "y".into()];
        let b = vec!["y".into(), "z".into()];
        let merged = super::merge_unique(&a, &b);
        assert_eq!(merged.len(), 3);
        assert!(merged.contains(&"x".into()));
        assert!(merged.contains(&"y".into()));
        assert!(merged.contains(&"z".into()));
    }

    #[test]
    fn test_merge_unique_empty() {
        let a: Vec<String> = vec![];
        let b: Vec<String> = vec!["a".into()];
        let merged = super::merge_unique(&a, &b);
        assert_eq!(merged.len(), 1);
    }
}
