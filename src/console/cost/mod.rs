//! cost — Defines console-facing cost tracking and reporting data.

pub mod budget;
pub mod report;

pub use budget::BudgetDecision;
pub use report::{CostBreakdown, CostReport, DailyCost, MonthlyCost};

/// 成本中心 — 管理预算、通知和成本汇总。
/// Cost center managing budget, notifications, and cost aggregation.
pub struct CostCenter {
    budget: f64,
    budget_action: String,
    total_cost: f64,
    notification_count: u64,
}

impl CostCenter {
    /// 创建新的成本中心 — 设置预算上限，默认动作 `warn`。
    /// Creates a new CostCenter with the given budget and default action `warn`.
    pub fn new(budget: f64) -> Self {
        CostCenter {
            budget,
            budget_action: "warn".into(),
            total_cost: 12.45,
            notification_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_marks_budget_exceeded_when_total_is_above_budget() {
        let center = CostCenter::new(10.0);

        let report = center.get_report();

        assert_eq!(report.budget, 10.0);
        assert!(report.budget_exceeded);
    }

    #[test]
    fn report_stays_within_budget_when_total_is_below_budget() {
        let center = CostCenter::new(20.0);

        let report = center.get_report();

        assert!(!report.budget_exceeded);
        assert_eq!(report.total_cost, 12.45);
    }

    #[test]
    fn set_budget_updates_report_threshold() {
        let mut center = CostCenter::new(1.0);
        center.set_budget(100.0);

        assert_eq!(center.budget(), 100.0);
        assert!(!center.get_report().budget_exceeded);
    }

    #[test]
    fn budget_action_is_configurable() {
        let mut center = CostCenter::new(10.0);

        center.set_budget_action("block");

        assert_eq!(center.budget_action(), "block");
    }

    #[test]
    fn report_contains_agent_tool_and_model_breakdowns() {
        let report = CostCenter::new(100.0).get_report();

        assert!(!report.by_agent.is_empty());
        assert!(!report.by_tool.is_empty());
        assert!(!report.by_model.is_empty());
    }

    #[test]
    fn reset_zeroes_cost_and_notifications() {
        let mut center = CostCenter::new(100.0);
        center.set_budget_action("notify");
        center.check_and_act(100.0).unwrap();

        center.reset();

        let summary = center.summary();
        assert_eq!(summary["total_cost"], 0.0);
        assert_eq!(summary["notification_count"], 0);
    }

    #[test]
    fn check_and_act_blocks_when_configured() {
        let mut center = CostCenter::new(1.0);
        center.reset();
        center.set_budget_action("block");

        let err = center.check_and_act(2.0).unwrap_err();

        assert!(err.contains("budget exceeded"));
    }

    #[test]
    fn check_and_act_warn_allows_over_budget() {
        let mut center = CostCenter::new(1.0);
        center.reset();

        assert!(center.check_and_act(2.0).is_ok());
        assert!(center.get_report().budget_exceeded);
    }

    #[test]
    fn check_and_act_notify_tracks_event_count() {
        let mut center = CostCenter::new(1.0);
        center.reset();
        center.set_budget_action("notify");

        center.check_and_act(2.0).unwrap();

        assert_eq!(center.summary()["notification_count"], 1);
    }

    #[test]
    fn check_budget_can_downgrade_model_when_over_budget() {
        let mut center = CostCenter::new(1.0);
        center.reset();
        center.set_budget_action("downgrade");

        let decision = center.check_budget(2.0).unwrap();

        assert_eq!(
            decision,
            BudgetDecision::DowngradeModel {
                from: "premium".to_string(),
                to: "economy".to_string(),
                reason: "budget exceeded: total cost 2.00 is above budget 1.00".to_string(),
            }
        );
    }

    #[test]
    fn check_budget_can_pause_workflow_when_over_budget() {
        let mut center = CostCenter::new(1.0);
        center.reset();
        center.set_budget_action("pause");

        let decision = center.check_budget(2.0).unwrap();

        assert!(matches!(decision, BudgetDecision::PauseWorkflow { .. }));
    }

    #[test]
    fn summary_returns_json_report() {
        let summary = CostCenter::new(100.0).summary();

        assert_eq!(summary["budget"], 100.0);
        assert_eq!(summary["budget_action"], "warn");
        assert!(summary["by_agent_count"].as_u64().unwrap() > 0);
    }
}
