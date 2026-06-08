//! Natural-language decision rule commands for Supervisor.

use super::*;
use crate::core::decision_rules::{DecisionRule, DecisionRuleStore};

impl Supervisor {
    /// Parses a natural language instruction about decision rules (add/delete/update/list/find)
    /// and dispatches to the appropriate DecisionRuleStore method.
    pub fn modify_rule_from_nl(&self, nl: &str) -> Result<String, String> {
        let storage = self
            .storage
            .as_ref()
            .ok_or_else(|| "Storage not available".to_string())?;

        let lower = nl.to_lowercase();

        if lower.starts_with("add") || lower.starts_with("create") {
            let parts: Vec<&str> = nl.splitn(5, '|').collect();
            if parts.len() < 5 {
                return Err("Format: add | <action> | <level> | <condition> | <effect>".to_string());
            }
            let level = crate::core::decision_rules::parse_decision_level(parts[2]).ok_or_else(|| {
                format!(
                    "Invalid decision level: '{}'. Use L1-L6 or direct_answer/single_tool/single_agent/team/workflow/jump_studio",
                    parts[2].trim()
                )
            })?;
            let rule = DecisionRule {
                id: format!("rule-{}", uuid::Uuid::new_v4()),
                action: parts[1].trim().to_string(),
                level,
                condition: parts[3].trim().to_string(),
                effect: parts[4].trim().to_string(),
                created_at: chrono::Utc::now().to_rfc3339(),
            };
            storage.add_rule(rule)?;
            Ok("Rule added".to_string())
        } else if lower.starts_with("del")
            || lower.starts_with("remove")
            || lower.starts_with("delete")
        {
            let id = nl
                .split_whitespace()
                .nth(1)
                .ok_or("Usage: delete <rule_id>")?;
            storage.remove_rule(id)?;
            Ok(format!("Rule '{}' deleted", id))
        } else if lower.starts_with("list") || lower.starts_with("all") {
            let rules = storage.list_rules()?;
            let json = serde_json::to_string(&rules).map_err(|e| e.to_string())?;
            Ok(json)
        } else if lower.starts_with("find") || lower.starts_with("search") {
            let action = nl
                .split_whitespace()
                .skip(1)
                .collect::<Vec<&str>>()
                .join(" ");
            if action.is_empty() {
                return Err("Usage: find <action>".to_string());
            }
            match storage.find_rule(&action)? {
                Some(rule) => {
                    let json = serde_json::to_string(&rule).map_err(|e| e.to_string())?;
                    Ok(json)
                }
                None => Ok(format!("No rule found for action '{}'", action)),
            }
        } else {
            Err("Unknown command. Use: add | <action> | <level> | <condition> | <effect>, delete <id>, list, find <action>".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::storage::Storage;

    fn supervisor_with_storage() -> Supervisor {
        Supervisor::new(Some(Storage::new_in_memory().unwrap()), None)
    }

    #[test]
    fn parses_add_command() {
        let supervisor = supervisor_with_storage();

        let result = supervisor
            .modify_rule_from_nl("add | deploy | L4 | contains prod | require approval")
            .unwrap();

        assert_eq!(result, "Rule added");
        let listed = supervisor.modify_rule_from_nl("list").unwrap();
        assert!(listed.contains("deploy"));
        assert!(listed.contains("require approval"));
    }

    #[test]
    fn parses_delete_command() {
        let supervisor = supervisor_with_storage();
        supervisor
            .modify_rule_from_nl("add | search | L2 | contains web | allow")
            .unwrap();
        let listed = supervisor.modify_rule_from_nl("list").unwrap();
        let rules: Vec<DecisionRule> = serde_json::from_str(&listed).unwrap();

        let result = supervisor
            .modify_rule_from_nl(&format!("delete {}", rules[0].id))
            .unwrap();

        assert!(result.contains("deleted"));
        assert_eq!(supervisor.modify_rule_from_nl("list").unwrap(), "[]");
    }

    #[test]
    fn parses_list_command() {
        let supervisor = supervisor_with_storage();
        supervisor
            .modify_rule_from_nl("add | chat | L1 | always | allow")
            .unwrap();

        let listed = supervisor.modify_rule_from_nl("list").unwrap();
        let rules: Vec<DecisionRule> = serde_json::from_str(&listed).unwrap();

        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].action, "chat");
    }

    #[test]
    fn parses_find_command() {
        let supervisor = supervisor_with_storage();
        supervisor
            .modify_rule_from_nl("add | analyze report | L3 | long input | delegate")
            .unwrap();

        let found = supervisor
            .modify_rule_from_nl("find analyze report")
            .unwrap();
        let rule: DecisionRule = serde_json::from_str(&found).unwrap();

        assert_eq!(rule.action, "analyze report");
        assert_eq!(rule.level, DecisionLevel::L3SingleAgent);
    }

    #[test]
    fn invalid_command_returns_error() {
        let supervisor = supervisor_with_storage();

        let err = supervisor.modify_rule_from_nl("rename rule").unwrap_err();

        assert!(err.contains("Unknown command"));
    }
}
