//! rules — Composition constraint engine for agent assembly (§2.5 of DESIGN.md)
//!
//! Enforces the following composition rules:
//! - Required: at least 1 memory + 1 tool + 1 LLM (auto-complete with defaults if missing)
//! - Conflict: no duplicate type layers; local LLM + cloud tools = incompatible
//! - Constraint: memory_layers × active_agents ≤ 5; tools ≤ 15/session
//! - Compatibility: port mismatch auto-inserts Transformer middleware

use crate::core::error::MornError;
#[derive(Debug, Clone, PartialEq)]
pub enum RuleViolation {
    MissingMemory,
    MissingTool,
    MissingLlm,
    TooManyMemory(usize),
    TooManyTools(usize),
    TooManyLlm(usize),
    LocalLlmWithCloudTool,
    MemoryAgentOverflow(usize, usize, usize),
    PortMismatch {
        source: String,
        target: String,
        source_port: String,
        target_port: String,
    },
}

#[derive(Debug, Clone)]
pub struct ComponentSelection {
    pub memory_ids: Vec<String>,
    pub tool_ids: Vec<String>,
    pub llm_ids: Vec<String>,
    pub channel_ids: Vec<String>,
    pub persona_ids: Vec<String>,
    pub skill_ids: Vec<String>,
    pub active_agents: usize,
}

impl ComponentSelection {
    pub fn new(active_agents: usize) -> Self {
        ComponentSelection {
            memory_ids: Vec::new(),
            tool_ids: Vec::new(),
            llm_ids: Vec::new(),
            channel_ids: Vec::new(),
            persona_ids: Vec::new(),
            skill_ids: Vec::new(),
            active_agents,
        }
    }

    pub fn with_memory(mut self, ids: Vec<String>) -> Self {
        self.memory_ids = ids;
        self
    }

    pub fn with_tools(mut self, ids: Vec<String>) -> Self {
        self.tool_ids = ids;
        self
    }

    pub fn with_llm(mut self, ids: Vec<String>) -> Self {
        self.llm_ids = ids;
        self
    }

    pub fn with_channels(mut self, ids: Vec<String>) -> Self {
        self.channel_ids = ids;
        self
    }

    pub fn with_personas(mut self, ids: Vec<String>) -> Self {
        self.persona_ids = ids;
        self
    }

    pub fn with_skills(mut self, ids: Vec<String>) -> Self {
        self.skill_ids = ids;
        self
    }
}

pub struct CompositionRules;

impl CompositionRules {
    pub fn check_required(sel: &ComponentSelection) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        if sel.memory_ids.is_empty() {
            violations.push(RuleViolation::MissingMemory);
        }
        if sel.tool_ids.is_empty() {
            violations.push(RuleViolation::MissingTool);
        }
        if sel.llm_ids.is_empty() {
            violations.push(RuleViolation::MissingLlm);
        }
        violations
    }

    pub fn check_conflicts(sel: &ComponentSelection) -> Vec<RuleViolation> {
        let mut violations = Vec::new();

        if sel.memory_ids.len() > 3 {
            violations.push(RuleViolation::TooManyMemory(sel.memory_ids.len()));
        }
        if sel.tool_ids.len() > 15 {
            violations.push(RuleViolation::TooManyTools(sel.tool_ids.len()));
        }
        if sel.llm_ids.len() > 3 {
            violations.push(RuleViolation::TooManyLlm(sel.llm_ids.len()));
        }

        let has_local_llm = sel
            .llm_ids
            .iter()
            .any(|id| id.contains("local") || id.contains("gguf"));
        let has_cloud_tool = sel
            .tool_ids
            .iter()
            .any(|id| id.contains("web") || id.contains("api") || id.contains("search"));
        if has_local_llm && has_cloud_tool {
            violations.push(RuleViolation::LocalLlmWithCloudTool);
        }

        violations
    }

    pub fn check_constraints(sel: &ComponentSelection) -> Vec<RuleViolation> {
        let mut violations = Vec::new();
        let memory_layers = sel.memory_ids.len();
        let product = memory_layers * sel.active_agents;
        if product > 5 {
            violations.push(RuleViolation::MemoryAgentOverflow(
                memory_layers,
                sel.active_agents,
                product,
            ));
        }
        violations
    }

    pub fn check_port_mismatch(
        source_type: &str,
        target_type: &str,
        source_port: &str,
        target_port: &str,
    ) -> Option<RuleViolation> {
        if source_port != target_port {
            Some(RuleViolation::PortMismatch {
                source: source_type.to_string(),
                target: target_type.to_string(),
                source_port: source_port.to_string(),
                target_port: target_port.to_string(),
            })
        } else {
            None
        }
    }

    pub fn validate_all(sel: &ComponentSelection) -> Vec<RuleViolation> {
        let mut all = Self::check_required(sel);
        all.extend(Self::check_conflicts(sel));
        all.extend(Self::check_constraints(sel));
        all
    }

    pub fn auto_fill_defaults(sel: &mut ComponentSelection) {
        if sel.memory_ids.is_empty() {
            sel.memory_ids = vec!["working_memory".to_string()];
        }
        if sel.tool_ids.is_empty() {
            sel.tool_ids = vec!["web_search".to_string(), "read_file".to_string()];
        }
        if sel.llm_ids.is_empty() {
            sel.llm_ids = vec!["deepseek-chat".to_string()];
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_required_all_present() {
        let sel = ComponentSelection::new(1)
            .with_memory(vec!["working".into()])
            .with_tools(vec!["web_search".into()])
            .with_llm(vec!["deepseek".into()]);
        assert!(CompositionRules::check_required(&sel).is_empty());
    }

    #[test]
    fn test_required_missing_memory() {
        let sel = ComponentSelection::new(1)
            .with_tools(vec!["web_search".into()])
            .with_llm(vec!["deepseek".into()]);
        let violations = CompositionRules::check_required(&sel);
        assert!(violations.contains(&RuleViolation::MissingMemory));
    }

    #[test]
    fn test_required_missing_tool() {
        let sel = ComponentSelection::new(1)
            .with_memory(vec!["working".into()])
            .with_llm(vec!["deepseek".into()]);
        let violations = CompositionRules::check_required(&sel);
        assert!(violations.contains(&RuleViolation::MissingTool));
    }

    #[test]
    fn test_required_missing_llm() {
        let sel = ComponentSelection::new(1)
            .with_memory(vec!["working".into()])
            .with_tools(vec!["web_search".into()]);
        let violations = CompositionRules::check_required(&sel);
        assert!(violations.contains(&RuleViolation::MissingLlm));
    }

    #[test]
    fn test_conflict_local_llm_with_cloud_tool() {
        let sel = ComponentSelection::new(1)
            .with_memory(vec!["working".into()])
            .with_tools(vec!["web_search".into()])
            .with_llm(vec!["local-gguf".into()]);
        let violations = CompositionRules::check_conflicts(&sel);
        assert!(violations.contains(&RuleViolation::LocalLlmWithCloudTool));
    }

    #[test]
    fn test_constraint_memory_agent_overflow() {
        let sel = ComponentSelection::new(3)
            .with_memory(vec!["a".into(), "b".into(), "c".into()])
            .with_tools(vec!["web_search".into()])
            .with_llm(vec!["deepseek".into()]);
        let violations = CompositionRules::check_constraints(&sel);
        assert!(violations.contains(&RuleViolation::MemoryAgentOverflow(3, 3, 9)));
    }

    #[test]
    fn test_constraint_within_limits() {
        let sel = ComponentSelection::new(1)
            .with_memory(vec!["working".into()])
            .with_tools(vec!["web_search".into()])
            .with_llm(vec!["deepseek".into()]);
        assert!(CompositionRules::check_constraints(&sel).is_empty());
    }

    #[test]
    fn test_port_mismatch_detected() {
        let result = CompositionRules::check_port_mismatch("tool", "llm", "output", "prompt");
        assert!(result.is_some());
    }

    #[test]
    fn test_port_match_ok() {
        let result = CompositionRules::check_port_mismatch("tool", "llm", "output", "output");
        assert!(result.is_none());
    }

    #[test]
    fn test_validate_all_empty_selector() {
        let sel = ComponentSelection::new(1);
        let violations = CompositionRules::validate_all(&sel);
        assert!(violations.contains(&RuleViolation::MissingMemory));
        assert!(violations.contains(&RuleViolation::MissingTool));
        assert!(violations.contains(&RuleViolation::MissingLlm));
    }

    #[test]
    fn test_auto_fill_defaults() {
        let mut sel = ComponentSelection::new(1);
        CompositionRules::auto_fill_defaults(&mut sel);
        assert!(!sel.memory_ids.is_empty());
        assert!(!sel.tool_ids.is_empty());
        assert!(!sel.llm_ids.is_empty());
    }

    #[test]
    fn test_auto_fill_does_not_override() {
        let mut sel = ComponentSelection::new(1)
            .with_memory(vec!["custom_mem".into()])
            .with_tools(vec!["custom_tool".into()])
            .with_llm(vec!["custom_llm".into()]);
        CompositionRules::auto_fill_defaults(&mut sel);
        assert_eq!(sel.memory_ids, vec!["custom_mem"]);
        assert_eq!(sel.tool_ids, vec!["custom_tool"]);
        assert_eq!(sel.llm_ids, vec!["custom_llm"]);
    }

    #[test]
    fn test_too_many_memory_conflict() {
        let sel = ComponentSelection::new(1)
            .with_memory(vec!["a".into(), "b".into(), "c".into(), "d".into()])
            .with_tools(vec!["tool".into()])
            .with_llm(vec!["llm".into()]);
        let violations = CompositionRules::check_conflicts(&sel);
        assert!(violations.contains(&RuleViolation::TooManyMemory(4)));
    }

    #[test]
    fn test_too_many_tools_conflict() {
        let sel = ComponentSelection::new(1)
            .with_memory(vec!["mem".into()])
            .with_tools((0..16).map(|i| format!("tool_{}", i)).collect())
            .with_llm(vec!["llm".into()]);
        let violations = CompositionRules::check_conflicts(&sel);
        assert!(violations.contains(&RuleViolation::TooManyTools(16)));
    }
}
