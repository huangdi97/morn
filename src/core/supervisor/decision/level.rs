use crate::core::supervisor::{DecisionLevel, Supervisor};

impl Supervisor {
    pub fn decide_level(&self, text: &str) -> DecisionLevel {
        let text_lower = text.to_lowercase();

        if let Some(forced_level) = forced_level_from_text(&text_lower) {
            return forced_level;
        }

        let simple_indicators = [
            "hello",
            "hi ",
            "thanks",
            "bye",
            "who are you",
            "what is your name",
            "good morning",
        ];
        if simple_indicators
            .iter()
            .any(|s| text_lower.contains(s) || text_lower == s.trim())
        {
            return DecisionLevel::L1DirectAnswer;
        }

        let tool_indicators = [
            "search",
            "look up",
            "find ",
            "calculate",
            "compute",
            "convert",
            "translate",
            "what time",
            "what's the time",
            "搜索",
        ];
        if tool_indicators.iter().any(|s| text_lower.contains(s)) {
            return DecisionLevel::L2SingleTool;
        }

        let studio_indicators = [
            "create an agent",
            "create a agent",
            "build an agent",
            "make an agent",
            "design a agent",
            "customize",
            "configure",
            "create workflow",
        ];
        if studio_indicators
            .iter()
            .any(|s| text_lower.contains(s) || text_lower.starts_with(s.trim()))
        {
            return DecisionLevel::L6JumpToStudio;
        }

        let workflow_indicators = [
            "report",
            "analysis",
            "research",
            "investigate",
            "compare",
            "plan",
            "strategy",
            "create a",
        ];
        if workflow_indicators.iter().any(|s| text_lower.contains(s)) {
            return DecisionLevel::L5Workflow;
        }

        let team_indicators = [
            "complex",
            "multi-step",
            "multiple",
            "various",
            "comprehensive",
            "full",
            "end-to-end",
        ];
        if team_indicators.iter().any(|s| text_lower.contains(s)) {
            return DecisionLevel::L4Team;
        }

        DecisionLevel::L3SingleAgent
    }
}

pub(crate) fn forced_level_from_text(text_lower: &str) -> Option<DecisionLevel> {
    if (text_lower.contains('用') && text_lower.contains("团队"))
        || text_lower.contains("use data team")
        || text_lower.contains("data team")
        || text_lower.contains("use team")
    {
        return Some(DecisionLevel::L4Team);
    }
    if text_lower.contains("直接说")
        || text_lower.contains("直接回答")
        || text_lower.contains("直接答")
        || text_lower.contains("just answer")
        || text_lower.contains("direct answer")
    {
        return Some(DecisionLevel::L1DirectAnswer);
    }
    if text_lower.contains("搜索") || text_lower.contains("search") {
        return Some(DecisionLevel::L2SingleTool);
    }
    None
}