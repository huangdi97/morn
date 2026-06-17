//! Built-in preset agent definitions and seeding logic used during first-time setup.

use crate::core::storage::Storage;
use crate::core::supervisor::types::{NLAgentDef, NLPersonaConfig};
use crate::studio::manager::{CreateComponentDef, StudioManager};

/// Returns the built-in set of preset agent definitions.
///
/// Eight default agents are provided: a general-purpose assistant, a file assistant,
/// a system assistant, a writing assistant, a translator, a system butler, a reviewer,
/// and a customer service agent. Each carries a Chinese-language persona,
/// a default model (`gpt-4o`), relevant tools, knowledge bases, and skills.
pub fn preset_agent_defs() -> Vec<NLAgentDef> {
    vec![
        NLAgentDef {
            name: "万能助手".into(),
            persona: "你是一个通用的AI助手，能够处理各种日常任务和问题。".into(),
            model: "gpt-4o".into(),
            tools: vec!["web_search".into(), "calculator".into()],
            knowledge: vec!["general_knowledge".into()],
            skills: vec!["communication".into(), "problem_solving".into()],
            memory: vec![],
            persona_config: NLPersonaConfig::default(),
            communication_style: "professional".into(),
            suggestions: vec![],
        },
        NLAgentDef {
            name: "文件助手".into(),
            persona: "你是一个文件管理助手，擅长文件整理、格式转换、内容搜索和批量处理。".into(),
            model: "gpt-4o".into(),
            tools: vec![
                "file_ops".into(),
                "read_file".into(),
                "write_file".into(),
                "compress".into(),
            ],
            knowledge: vec!["file_formats".into(), "compression_methods".into()],
            skills: vec![
                "file_management".into(),
                "format_conversion".into(),
                "batch_processing".into(),
            ],
            memory: vec![],
            persona_config: NLPersonaConfig::default(),
            communication_style: "professional".into(),
            suggestions: vec![],
        },
        NLAgentDef {
            name: "系统助手".into(),
            persona: "你是一个系统管理助手，擅长系统信息查询、进程监控、性能分析和故障排查。"
                .into(),
            model: "gpt-4o".into(),
            tools: vec![
                "system_info".into(),
                "process_monitor".into(),
                "disk_usage".into(),
                "network_check".into(),
            ],
            knowledge: vec!["system_administration".into(), "performance_tuning".into()],
            skills: vec![
                "system_diagnostics".into(),
                "performance_analysis".into(),
                "troubleshooting".into(),
            ],
            memory: vec![],
            persona_config: NLPersonaConfig::default(),
            communication_style: "professional".into(),
            suggestions: vec![],
        },
        NLAgentDef {
            name: "写作助手".into(),
            persona: "你是一个专业写作助手，擅长文章撰写、文档排版、内容润色和多语言翻译。".into(),
            model: "gpt-4o".into(),
            tools: vec![
                "draft".into(),
                "proofread".into(),
                "format_doc".into(),
                "translate".into(),
            ],
            knowledge: vec!["writing_style_guides".into(), "document_templates".into()],
            skills: vec![
                "content_writing".into(),
                "document_formatting".into(),
                "copy_editing".into(),
            ],
            memory: vec![],
            persona_config: NLPersonaConfig::default(),
            communication_style: "professional".into(),
            suggestions: vec![],
        },
        NLAgentDef {
            name: "翻译官".into(),
            persona: "你是一名专业的翻译官，精通多国语言互译和本地化。".into(),
            model: "gpt-4o".into(),
            tools: vec!["detect_lang".into(), "translate".into(), "proofread".into()],
            knowledge: vec!["bilingual_dict".into()],
            skills: vec!["translation".into(), "proofreading".into()],
            memory: vec![],
            persona_config: NLPersonaConfig::default(),
            communication_style: "professional".into(),
            suggestions: vec![],
        },
        NLAgentDef {
            name: "系统管家".into(),
            persona: "你是一个贴心的系统管家，擅长桌面操作、文件管理和应用启动。".into(),
            model: "gpt-4o".into(),
            tools: vec!["launch_app".into(), "read_file".into(), "browse_web".into()],
            knowledge: vec![],
            skills: vec!["system_management".into(), "file_operations".into()],
            memory: vec![],
            persona_config: NLPersonaConfig::default(),
            communication_style: "professional".into(),
            suggestions: vec![],
        },
        NLAgentDef {
            name: "审查员".into(),
            persona: "你是一名严谨的审查员，擅长代码审查、文档校对和合规检查。".into(),
            model: "gpt-4o".into(),
            tools: vec!["read_file".into(), "diff".into(), "lint_check".into()],
            knowledge: vec!["coding_standards".into()],
            skills: vec!["code_review".into(), "quality_assurance".into()],
            memory: vec![],
            persona_config: NLPersonaConfig::default(),
            communication_style: "professional".into(),
            suggestions: vec![],
        },
        NLAgentDef {
            name: "客服".into(),
            persona: "你是一个友好的客服代表，擅长解答用户问题、处理反馈和升级工单。".into(),
            model: "gpt-4o".into(),
            tools: vec![
                "search_kb".into(),
                "classify_intent".into(),
                "escalate".into(),
            ],
            knowledge: vec!["faq_db".into()],
            skills: vec!["customer_service".into(), "communication".into()],
            memory: vec![],
            persona_config: NLPersonaConfig::default(),
            communication_style: "friendly".into(),
            suggestions: vec![],
        },
    ]
}

/// Seeds preset agents into storage if no agents already exist.
///
/// This is a no-op when `storage` is `None` or when the storage already contains
/// at least one agent (to avoid duplicating presets on re-initialization).
///
/// For each preset definition returned by [`preset_agent_defs`], a component of
/// type `"agent"` is created through the provided [`StudioManager`].
pub fn seed_preset_agents(storage: &Option<Storage>, manager: &StudioManager) {
    let storage = match storage {
        Some(s) => s,
        None => return,
    };

    if storage
        .list_agents()
        .map(|a| !a.is_empty())
        .unwrap_or(false)
    {
        return;
    }

    for def in preset_agent_defs() {
        let config_json = serde_json::to_string(&def).ok();
        let _ = manager.create_component(CreateComponentDef {
            name: def.name,
            component_type: "agent".into(),
            config_json,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage::Storage;

    #[test]
    fn test_preset_agent_defs_count() {
        let defs = preset_agent_defs();
        assert_eq!(defs.len(), 8);
    }

    #[test]
    fn test_preset_agent_defs_have_names() {
        let defs = preset_agent_defs();
        for def in &defs {
            assert!(!def.name.is_empty());
            assert!(!def.persona.is_empty());
            assert!(!def.model.is_empty());
        }
    }

    #[test]
    fn test_seed_preset_agents_creates_eight_agents() {
        let storage = Storage::new_in_memory().unwrap();
        let manager = StudioManager::new(None, Some(storage.clone()), None);
        seed_preset_agents(&Some(storage.clone()), &manager);

        let agents = storage.list_agents().unwrap();
        assert_eq!(agents.len(), 8);
    }

    #[test]
    fn test_seed_preset_agents_idempotent() {
        let storage = Storage::new_in_memory().unwrap();
        let manager = StudioManager::new(None, Some(storage.clone()), None);
        seed_preset_agents(&Some(storage.clone()), &manager);
        seed_preset_agents(&Some(storage.clone()), &manager);

        let agents = storage.list_agents().unwrap();
        assert_eq!(agents.len(), 8);
    }

    #[test]
    fn test_seed_preset_agents_config_json_is_valid() {
        let storage = Storage::new_in_memory().unwrap();
        let manager = StudioManager::new(None, Some(storage.clone()), None);
        seed_preset_agents(&Some(storage.clone()), &manager);

        let agents = storage.list_agents().unwrap();
        for agent in &agents {
            let config = agent.config_json.as_deref().unwrap_or("{}");
            let def: NLAgentDef = serde_json::from_str(config).unwrap();
            assert!(!def.name.is_empty());
        }
    }
}
