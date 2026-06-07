//! version — Compares semantic capability versions for registry selection.
use std::cmp::Ordering;

use super::{Capability, Registry};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentTemplate {
    pub id: String,
    pub version: String,
    pub name: String,
    pub icon: String,
    pub description: String,
    pub persona: String,
    pub model: String,
    pub tools: Vec<String>,
    pub knowledge: Vec<String>,
    pub skills: Vec<String>,
}

/// Compares two dotted version strings and returns their ordering.
pub fn compare_versions(a: &str, b: &str) -> Ordering {
    let parse = |version: &str| {
        version
            .split('.')
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };

    let a_parts = parse(a);
    let b_parts = parse(b);
    let max_len = a_parts.len().max(b_parts.len());

    for i in 0..max_len {
        let a_part = a_parts.get(i).copied().unwrap_or(0);
        let b_part = b_parts.get(i).copied().unwrap_or(0);
        match a_part.cmp(&b_part) {
            Ordering::Equal => {}
            ordering => return ordering,
        }
    }

    Ordering::Equal
}

pub(super) fn default_templates() -> Vec<AgentTemplate> {
    vec![
        AgentTemplate {
            id: "research-assistant".into(),
            version: "0.1.0".into(),
            name: "研究助手".into(),
            icon: "🔬".into(),
            description: "多源信息检索、知识整合与摘要生成，适合学术研究和文献综述".into(),
            persona: "researcher".into(),
            model: "deepseek-chat".into(),
            tools: vec!["web_search".into(), "read_file".into()],
            knowledge: vec!["docs".into(), "data_sources".into()],
            skills: vec!["summarization".into(), "report_generation".into()],
        },
        AgentTemplate {
            id: "data-analyst".into(),
            version: "0.1.0".into(),
            name: "数据分析师".into(),
            icon: "📊".into(),
            description: "获取行情数据、计算技术指标、生成图表和报告，适合金融与数据领域".into(),
            persona: "analyst".into(),
            model: "deepseek-chat".into(),
            tools: vec![
                "get_kline".into(),
                "calc_macd".into(),
                "chart".into(),
                "exec_python".into(),
            ],
            knowledge: vec!["docs".into(), "data_sources".into()],
            skills: vec!["report_generation".into()],
        },
        AgentTemplate {
            id: "writing-assistant".into(),
            version: "0.1.0".into(),
            name: "写作助手".into(),
            icon: "✍️".into(),
            description: "多语言翻译、语法检查、格式润色与风格优化，适合内容创作者".into(),
            persona: "writer".into(),
            model: "deepseek-chat".into(),
            tools: vec!["web_search".into(), "read_file".into(), "write_file".into()],
            knowledge: vec!["glossary".into()],
            skills: vec![
                "translation".into(),
                "grammar_check".into(),
                "format".into(),
                "style".into(),
                "proofread".into(),
            ],
        },
        AgentTemplate {
            id: "coding-helper".into(),
            version: "0.1.0".into(),
            name: "编码助手".into(),
            icon: "💻".into(),
            description: "代码审查、调试、格式化和测试，适合软件开发与编程".into(),
            persona: "coder".into(),
            model: "deepseek-chat".into(),
            tools: vec![
                "read_file".into(),
                "write_file".into(),
                "exec_python".into(),
            ],
            knowledge: vec!["docs".into()],
            skills: vec![
                "code_review".into(),
                "debug".into(),
                "format".into(),
                "test".into(),
            ],
        },
        AgentTemplate {
            id: "translation-agent".into(),
            version: "0.1.0".into(),
            name: "翻译 Agent".into(),
            icon: "🌐".into(),
            description: "多语言翻译、校对和专业术语管理，适合跨语言工作".into(),
            persona: "translator".into(),
            model: "deepseek-chat".into(),
            tools: vec!["web_search".into(), "read_file".into()],
            knowledge: vec!["glossary".into()],
            skills: vec!["translation".into(), "proofread".into()],
        },
        AgentTemplate {
            id: "general-assistant".into(),
            version: "0.1.0".into(),
            name: "通用助手".into(),
            icon: "🤖".into(),
            description: "混合工具集的通用助手，适合日常问答、搜索和简单任务".into(),
            persona: "assistant".into(),
            model: "deepseek-chat".into(),
            tools: vec![
                "web_search".into(),
                "read_file".into(),
                "get_time".into(),
                "calc".into(),
            ],
            knowledge: vec!["docs".into()],
            skills: vec![],
        },
    ]
}

impl Registry {
    /// Lists registered capabilities whose version exactly matches the provided version string.
    pub fn list_by_version(&self, version: &str) -> Vec<&Capability> {
        self.capabilities
            .values()
            .filter(|c| c.version == version)
            .collect()
    }

    /// Checks whether an existing capability or template id has a different version than requested.
    pub fn check_conflict(&self, id: &str, version: &str) -> bool {
        self.capabilities
            .get(id)
            .map(|c| c.version != version)
            .or_else(|| self.templates.get(id).map(|t| t.version != version))
            .unwrap_or(false)
    }

    /// Returns the version string for a capability or template id when it exists.
    pub fn get_version(&self, id: &str) -> Option<&str> {
        self.capabilities
            .get(id)
            .map(|c| c.version.as_str())
            .or_else(|| self.templates.get(id).map(|t| t.version.as_str()))
    }

    /// Returns references to all built-in agent templates.
    pub fn list_templates(&self) -> Vec<&AgentTemplate> {
        self.templates.values().collect()
    }

    /// Looks up a built-in agent template by id and returns it when found.
    pub fn get_template(&self, id: &str) -> Option<&AgentTemplate> {
        self.templates.get(id)
    }
}
