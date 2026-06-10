//! agent_templates — Pre-configured Agent definitions from DESIGN.md §3.7.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AgentTemplate {
    pub name: &'static str,
    pub persona: &'static str,
    pub tools: &'static [&'static str],
    pub knowledge: &'static [&'static str],
    pub description: &'static str,
}

pub static AGENT_TEMPLATES: &[AgentTemplate] = &[
    AgentTemplate {
        name: "数据分析师",
        persona: "analyst",
        tools: &["get_kline", "calc_macd", "calc_rsi"],
        knowledge: &["stock_db"],
        description: "股票分析、数据查询",
    },
    AgentTemplate {
        name: "研究员",
        persona: "researcher",
        tools: &["web_search", "news_fetch", "summarize"],
        knowledge: &["domain_terms"],
        description: "行业调研、竞品分析",
    },
    AgentTemplate {
        name: "写作者",
        persona: "writer",
        tools: &["draft", "review", "format", "send_msg"],
        knowledge: &[],
        description: "报告撰写、内容创作",
    },
    AgentTemplate {
        name: "程序员",
        persona: "coder",
        tools: &["read_file", "write_file", "exec_code", "git_op"],
        knowledge: &[],
        description: "代码开发、脚本编写",
    },
    AgentTemplate {
        name: "翻译官",
        persona: "translator",
        tools: &["detect_lang", "translate", "proofread"],
        knowledge: &["bilingual_dict"],
        description: "文档翻译",
    },
    AgentTemplate {
        name: "系统管家",
        persona: "assistant",
        tools: &["launch_app", "read_file", "browse_web", "send_msg"],
        knowledge: &[],
        description: "日常电脑辅助",
    },
    AgentTemplate {
        name: "审查员",
        persona: "reviewer",
        tools: &["read_file", "diff", "lint_check", "security_scan"],
        knowledge: &["coding_standards"],
        description: "代码审查、文档校对",
    },
    AgentTemplate {
        name: "客服",
        persona: "assistant",
        tools: &["search_kb", "classify_intent", "escalate", "send_msg"],
        knowledge: &["faq_db"],
        description: "自动回复",
    },
];

pub fn all_templates() -> Vec<AgentTemplate> {
    AGENT_TEMPLATES.to_vec()
}

pub fn find_template(name: &str) -> Option<&'static AgentTemplate> {
    AGENT_TEMPLATES
        .iter()
        .find(|template| template.name.contains(name) || name.contains(template.name))
}
