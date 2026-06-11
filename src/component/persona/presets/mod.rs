//! 预设人格集合 — 内置的行业/通用/创意三类预设
use super::Persona;

const PRESET_NAMES: &[(&str, &str, &str)] = &[
    ("preset-analyst", "数据分析师", "数据驱动的专业分析"),
    ("preset-researcher", "研究员", "多源信息收集与交叉验证"),
    ("preset-writer", "写作者", "将复杂信息转化为清晰文字"),
    ("preset-coder", "程序员", "简洁可维护的软件工程"),
    ("preset-translator", "翻译官", "信达雅的专业翻译"),
    ("preset-assistant", "系统管家", "高效随叫随到的 AI 管家"),
    ("preset-reviewer", "审查员", "严谨的代码/文档审查"),
    ("preset-cs-agent", "客服", "有耐心有同理心的客户服务"),
    ("preset-investment", "投资分析师", "风险与收益平衡的专业分析"),
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

fn preset_json_path(name: &str) -> String {
    let file_name = name.trim_start_matches("preset-").replace('-', "_");
    format!(
        "{}/src/component/persona/presets/{}.json",
        env!("CARGO_MANIFEST_DIR"),
        file_name
    )
}

pub fn all() -> Vec<Persona> {
    PRESET_NAMES
        .iter()
        .filter_map(|(id, _, _)| get_preset_persona(id))
        .collect()
}

pub fn get_preset_persona(name: &str) -> Option<Persona> {
    let name = name.trim_start_matches("preset-");
    let path = preset_json_path(name);
    let content = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn list_preset_personas() -> Vec<std::collections::HashMap<String, String>> {
    PRESET_NAMES
        .iter()
        .map(|(id, name, desc)| {
            let mut map = std::collections::HashMap::new();
            map.insert("id".into(), id.to_string());
            map.insert("name".into(), name.to_string());
            map.insert("description".into(), desc.to_string());
            map
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_assistant_preset_from_file() {
        let persona = get_preset_persona("preset-assistant");
        assert!(persona.is_some());
        let persona = persona.unwrap();
        assert_eq!(persona.name, "系统管家");
        assert_eq!(persona.id, "preset-assistant");
    }

    #[test]
    fn test_all_returns_52_presets() {
        let presets = all();
        assert_eq!(presets.len(), 52);
    }

    #[test]
    fn test_all_presets_have_valid_data() {
        for p in all() {
            assert!(!p.name.is_empty(), "Name empty for {}", p.id);
            assert!(!p.prompt_layers.l1_core_identity.is_empty(), "Identity empty for {}", p.id);
            assert!((0.0..=2.0).contains(&p.parameters.temperature));
            assert!((0.0..=1.0).contains(&p.parameters.verbosity));
            assert!((0.0..=1.0).contains(&p.parameters.proactiveness));
        }
    }

    #[test]
    fn test_list_preset_personas_count() {
        let list = list_preset_personas();
        assert_eq!(list.len(), 52);
        assert!(list[0].contains_key("id"));
        assert!(list[0].contains_key("name"));
        assert!(list[0].contains_key("description"));
    }
}