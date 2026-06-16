//! Pre-built team templates for rapid team composition.

use super::{CollaborationMode, ConsensusMechanism, TeamDef};

pub struct TeamPreset {
    pub keywords: Vec<&'static str>,
    pub team: TeamDef,
}

pub fn get_presets() -> Vec<TeamPreset> {
    vec![
        TeamPreset {
            keywords: vec!["code review", "review code", "pull request", "pr review"],
            team: code_review_team(),
        },
        TeamPreset {
            keywords: vec!["stock", "finance", "market", "trading", "stock research"],
            team: stock_research_team(),
        },
        TeamPreset {
            keywords: vec!["research", "investigate", "deep dive", "literature review"],
            team: research_team(),
        },
        TeamPreset {
            keywords: vec!["support", "customer", "ticket", "helpdesk", "faq"],
            team: support_team(),
        },
        TeamPreset {
            keywords: vec![
                "content",
                "write",
                "blog",
                "article",
                "documentation",
                "doc",
            ],
            team: content_team(),
        },
        TeamPreset {
            keywords: vec!["observability", "surveillance", "health check"],
            team: monitoring_team(),
        },
        TeamPreset {
            keywords: vec!["devops", "deploy", "monitor", "alert", "incident"],
            team: devops_team(),
        },
        TeamPreset {
            keywords: vec!["data", "etl", "dataset", "pipeline", "metrics"],
            team: data_team(),
        },
        TeamPreset {
            keywords: vec!["management", "manage", "coordinate", "decision", "execute"],
            team: management_team(),
        },
        TeamPreset {
            keywords: vec!["code", "develop", "program", "implementation", "build"],
            team: TeamDef {
                id: "preset-dev".into(),
                name: "Development Team".into(),
                members: vec![
                    "agent-architect".into(),
                    "agent-coder".into(),
                    "agent-reviewer".into(),
                    "agent-tester".into(),
                ],
                mode: CollaborationMode::Chain,
                consensus: ConsensusMechanism::CeoDecides,
            },
        },
        TeamPreset {
            keywords: vec!["analysis", "analyze", "data", "report", "dashboard"],
            team: TeamDef {
                id: "preset-analytics".into(),
                name: "Analytics Team".into(),
                members: vec![
                    "agent-data-engineer".into(),
                    "agent-analyst".into(),
                    "agent-visualizer".into(),
                ],
                mode: CollaborationMode::Broadcast,
                consensus: ConsensusMechanism::Vote,
            },
        },
        TeamPreset {
            keywords: vec!["review", "audit", "quality", "qa", "inspect"],
            team: TeamDef {
                id: "preset-review".into(),
                name: "Review Team".into(),
                members: vec![
                    "agent-reviewer".into(),
                    "agent-auditor".into(),
                    "agent-qa".into(),
                ],
                mode: CollaborationMode::Voting,
                consensus: ConsensusMechanism::MungerVeto,
            },
        },
        TeamPreset {
            keywords: vec!["stock", "research", "finance", "market", "trading"],
            team: stock_research_team(),
        },
        TeamPreset {
            keywords: vec!["risk", "control", "compliance", "audit", "security"],
            team: risk_control_team(),
        },
    ]
}

pub fn code_review_team() -> TeamDef {
    TeamDef {
        id: "preset-code-review".into(),
        name: "Code Review Team".into(),
        members: vec!["agent-reviewer".into(), "agent-author".into()],
        mode: CollaborationMode::Voting,
        consensus: ConsensusMechanism::MungerVeto,
    }
}

pub fn research_team() -> TeamDef {
    TeamDef {
        id: "preset-research".into(),
        name: "Research Team".into(),
        members: vec![
            "agent-researcher".into(),
            "agent-analyst".into(),
            "agent-writer".into(),
        ],
        mode: CollaborationMode::ManagerWorker,
        consensus: ConsensusMechanism::AutoSynthesis,
    }
}

pub fn support_team() -> TeamDef {
    TeamDef {
        id: "preset-support".into(),
        name: "Support Team".into(),
        members: vec!["agent-support".into(), "agent-quality".into()],
        mode: CollaborationMode::Routing,
        consensus: ConsensusMechanism::CeoDecides,
    }
}

pub fn content_team() -> TeamDef {
    TeamDef {
        id: "preset-content".into(),
        name: "Content Team".into(),
        members: vec![
            "agent-editor".into(),
            "agent-designer".into(),
            "agent-proofreader".into(),
        ],
        mode: CollaborationMode::Chain,
        consensus: ConsensusMechanism::CeoDecides,
    }
}

pub fn devops_team() -> TeamDef {
    TeamDef {
        id: "preset-devops".into(),
        name: "DevOps Team".into(),
        members: vec![
            "agent-deployer".into(),
            "agent-monitor".into(),
            "agent-alert".into(),
        ],
        mode: CollaborationMode::Blackboard,
        consensus: ConsensusMechanism::AutoSynthesis,
    }
}

pub fn data_team() -> TeamDef {
    TeamDef {
        id: "preset-data".into(),
        name: "Data Team".into(),
        members: vec![
            "agent-collector".into(),
            "agent-processor".into(),
            "agent-reporter".into(),
        ],
        mode: CollaborationMode::Chain,
        consensus: ConsensusMechanism::AutoSynthesis,
    }
}

pub fn management_team() -> TeamDef {
    TeamDef {
        id: "preset-management".into(),
        name: "Management Team".into(),
        members: vec![
            "agent-decision".into(),
            "agent-execution".into(),
            "agent-evaluation".into(),
        ],
        mode: CollaborationMode::ManagerWorker,
        consensus: ConsensusMechanism::CeoDecides,
    }
}

pub fn stock_research_team() -> TeamDef {
    TeamDef {
        id: "preset-stock-research".into(),
        name: "Stock Research Team".into(),
        members: vec![
            "data-agent".into(),
            "search-agent".into(),
            "fin-plot".into(),
            "chat-agent".into(),
        ],
        mode: CollaborationMode::Chain,
        consensus: ConsensusMechanism::AutoSynthesis,
    }
}

pub fn risk_control_team() -> TeamDef {
    TeamDef {
        id: "preset-risk-control".into(),
        name: "Risk Control Team".into(),
        members: vec![
            "data-agent".into(),
            "rule-agent".into(),
            "analyst-agent".into(),
            "alert-agent".into(),
        ],
        mode: CollaborationMode::Voting,
        consensus: ConsensusMechanism::MungerVeto,
    }
}

pub fn monitoring_team() -> TeamDef {
    TeamDef {
        id: "preset-monitoring".into(),
        name: "Monitoring Team".into(),
        members: vec![
            "timer-agent".into(),
            "check-agent".into(),
            "alert-agent".into(),
            "report-agent".into(),
        ],
        mode: CollaborationMode::Broadcast,
        consensus: ConsensusMechanism::AutoSynthesis,
    }
}

pub fn find_preset(input: &str) -> Option<TeamDef> {
    let lower = input.to_lowercase();
    for preset in get_presets() {
        if preset.keywords.iter().any(|kw| lower.contains(kw)) {
            return Some(preset.team);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_review_team_has_reviewer_and_author() {
        let team = code_review_team();
        assert_eq!(team.members, vec!["agent-reviewer", "agent-author"]);
    }

    #[test]
    fn research_team_has_three_roles() {
        assert_eq!(research_team().members.len(), 3);
    }

    #[test]
    fn support_team_has_two_roles() {
        assert_eq!(support_team().members.len(), 2);
    }

    #[test]
    fn content_team_has_editor_design_and_proofreader() {
        let team = content_team();
        assert!(team.members.contains(&"agent-editor".to_string()));
        assert!(team.members.contains(&"agent-designer".to_string()));
        assert!(team.members.contains(&"agent-proofreader".to_string()));
    }

    #[test]
    fn devops_team_has_operational_roles() {
        assert_eq!(devops_team().mode, CollaborationMode::Blackboard);
    }

    #[test]
    fn data_team_has_three_roles() {
        assert_eq!(data_team().members.len(), 3);
    }

    #[test]
    fn management_team_has_decision_execution_evaluation() {
        let team = management_team();
        assert_eq!(team.members[0], "agent-decision");
        assert_eq!(team.members[2], "agent-evaluation");
    }

    #[test]
    fn find_preset_matches_code_review_first() {
        let team = find_preset("please review code").unwrap();
        assert_eq!(team.id, "preset-code-review");
    }

    #[test]
    fn stock_research_team_has_four_members() {
        let team = stock_research_team();
        assert_eq!(team.members.len(), 4);
        assert_eq!(team.mode, CollaborationMode::Chain);
    }

    #[test]
    fn risk_control_team_has_four_members() {
        let team = risk_control_team();
        assert_eq!(team.members.len(), 4);
        assert_eq!(team.mode, CollaborationMode::Voting);
    }

    #[test]
    fn monitoring_team_has_four_members() {
        let team = monitoring_team();
        assert_eq!(team.members.len(), 4);
        assert_eq!(team.mode, CollaborationMode::Broadcast);
    }

    #[test]
    fn find_preset_matches_stock_research() {
        let team = find_preset("stock market research").unwrap();
        assert_eq!(team.id, "preset-stock-research");
    }

    #[test]
    fn find_preset_matches_risk_control() {
        let team = find_preset("risk compliance").unwrap();
        assert_eq!(team.id, "preset-risk-control");
    }

    #[test]
    fn find_preset_matches_monitoring() {
        let team = find_preset("observability check").unwrap();
        assert_eq!(team.id, "preset-monitoring");
    }
}
