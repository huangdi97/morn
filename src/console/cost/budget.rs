//! budget — Budget management with check_budget, auto-downgrade, and pause-workflow decisions.

use super::CostCenter;
use crate::core::error::{MornError, MornResult};

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum BudgetDecision {
    Continue,
    Warn {
        message: String,
    },
    Notify {
        message: String,
        notification_count: u64,
    },
    DowngradeModel {
        from: String,
        to: String,
        reason: String,
    },
    PauseWorkflow {
        reason: String,
    },
}

impl CostCenter {
    /// 设置预算上限 — Updates the budget threshold.
    pub fn set_budget(&mut self, budget: f64) {
        self.budget = budget;
    }

    /// 获取当前预算 — Returns the current budget.
    pub fn budget(&self) -> f64 {
        self.budget
    }

    /// 设置预算超支动作 — `warn` | `block` | `notify` | `downgrade` | `pause`.
    /// Sets the budget exceeded action: `warn` | `block` | `notify` | `downgrade` | `pause`.
    pub fn set_budget_action(&mut self, action: &str) {
        self.budget_action = action.to_string();
    }

    /// 获取预算超支动作 — Returns the budget exceeded action.
    pub fn budget_action(&self) -> &str {
        &self.budget_action
    }

    /// 重置成本与通知计数 — Resets total cost and notification count.
    pub fn reset(&mut self) {
        self.total_cost = 0.0;
        self.notification_count = 0;
    }

    /// 检查预算 — 累加成本，超预算时返回降级模型或暂停 workflow 等决策。
    /// Checks budget — accumulates cost and returns the configured enforcement decision.
    pub fn check_budget(&mut self, cost: f64) -> MornResult<BudgetDecision> {
        tracing::debug!(
            "checking budget with incremental cost {:.4}, current total {:.4}, budget {:.4}",
            cost,
            self.total_cost,
            self.budget
        );
        self.total_cost += cost;
        if self.total_cost <= self.budget {
            return Ok(BudgetDecision::Continue);
        }

        let message = format!(
            "budget exceeded: total cost {:.2} is above budget {:.2}",
            self.total_cost, self.budget
        );

        match self.budget_action.as_str() {
            "warn" => {
                tracing::warn!("{}", message);
                Ok(BudgetDecision::Warn { message })
            }
            "block" | "pause" => {
                tracing::warn!("pausing workflow because {}", message);
                Ok(BudgetDecision::PauseWorkflow { reason: message })
            }
            "notify" => {
                self.notification_count += 1;
                tracing::info!(
                    "budget notification emitted: total cost {:.2}, budget {:.2}",
                    self.total_cost,
                    self.budget
                );
                Ok(BudgetDecision::Notify {
                    message,
                    notification_count: self.notification_count,
                })
            }
            "downgrade" | "degrade" => {
                tracing::info!("downgrading model because {}", message);
                Ok(BudgetDecision::DowngradeModel {
                    from: "premium".to_string(),
                    to: "economy".to_string(),
                    reason: message,
                })
            }
            other => Err(MornError::Config(format!(
                "unknown budget action '{}'",
                other
            ))),
        }
    }

    /// 检查成本并执行动作 — 累加成本，超预算则按配置动作响应。
    /// Checks cost and acts — accumulates cost, triggers action if over budget.
    pub fn check_and_act(&mut self, cost: f64) -> MornResult<()> {
        match self.check_budget(cost)? {
            BudgetDecision::PauseWorkflow { reason } => Err(MornError::Budget(reason)),
            _ => Ok(()),
        }
    }

    /// 获取成本摘要 JSON — Returns a JSON summary of costs and budget status.
    pub fn summary(&self) -> serde_json::Value {
        let report = self.get_report();
        serde_json::json!({
            "total_cost": report.total_cost,
            "budget": report.budget,
            "budget_exceeded": report.budget_exceeded,
            "budget_action": self.budget_action,
            "notification_count": self.notification_count,
            "by_agent_count": report.by_agent.len(),
            "by_tool_count": report.by_tool.len(),
            "by_model_count": report.by_model.len(),
            "daily_points": report.daily_trend.len(),
            "monthly_points": report.monthly_trend.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::cost::CostCenter;

    #[test]
    fn test_set_budget_updates_value() {
        let mut center = CostCenter::new(10.0);
        center.set_budget(50.0);
        assert_eq!(center.budget(), 50.0);
    }

    #[test]
    fn test_set_budget_action_warn() {
        let mut center = CostCenter::new(10.0);
        center.set_budget_action("warn");
        assert_eq!(center.budget_action(), "warn");
    }

    #[test]
    fn test_check_budget_continue_within_budget() {
        let mut center = CostCenter::new(100.0);
        center.reset();
        let decision = center.check_budget(10.0).unwrap();
        assert_eq!(decision, BudgetDecision::Continue);
    }

    #[test]
    fn test_check_budget_warn_on_exceed() {
        let mut center = CostCenter::new(10.0);
        center.reset();
        center.set_budget_action("warn");
        let decision = center.check_budget(20.0).unwrap();
        assert!(matches!(decision, BudgetDecision::Warn { .. }));
    }

    #[test]
    fn test_check_budget_notify_increments_count() {
        let mut center = CostCenter::new(10.0);
        center.reset();
        center.set_budget_action("notify");
        let d1 = center.check_budget(20.0).unwrap();
        let d2 = center.check_budget(5.0).unwrap();
        if let BudgetDecision::Notify {
            notification_count, ..
        } = d1
        {
            assert_eq!(notification_count, 1);
        } else {
            panic!("expected Notify");
        }
        if let BudgetDecision::Notify {
            notification_count, ..
        } = d2
        {
            assert_eq!(notification_count, 2);
        } else {
            panic!("expected Notify");
        }
    }

    #[test]
    fn test_check_budget_downgrade() {
        let mut center = CostCenter::new(10.0);
        center.reset();
        center.set_budget_action("downgrade");
        let decision = center.check_budget(20.0).unwrap();
        assert!(matches!(decision, BudgetDecision::DowngradeModel { .. }));
    }

    #[test]
    fn test_check_budget_pause() {
        let mut center = CostCenter::new(10.0);
        center.reset();
        center.set_budget_action("pause");
        let decision = center.check_budget(20.0).unwrap();
        assert!(matches!(decision, BudgetDecision::PauseWorkflow { .. }));
    }

    #[test]
    fn test_check_budget_unknown_action() {
        let mut center = CostCenter::new(10.0);
        center.reset();
        center.set_budget_action("invalid");
        let result = center.check_budget(20.0);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_and_act_ok_within_budget() {
        let mut center = CostCenter::new(100.0);
        center.reset();
        assert!(center.check_and_act(10.0).is_ok());
    }

    #[test]
    fn test_check_and_act_blocks_on_pause() {
        let mut center = CostCenter::new(10.0);
        center.reset();
        center.set_budget_action("pause");
        assert!(center.check_and_act(20.0).is_err());
    }

    #[test]
    fn test_summary_contains_all_keys() {
        let summary = CostCenter::new(100.0).summary();
        assert!(summary.get("total_cost").is_some());
        assert!(summary.get("budget").is_some());
        assert!(summary.get("budget_action").is_some());
        assert!(summary.get("notification_count").is_some());
        assert!(summary.get("by_agent_count").is_some());
        assert!(summary.get("by_tool_count").is_some());
        assert!(summary.get("daily_points").is_some());
    }

    #[test]
    fn test_check_budget_accumulates_cost() {
        let mut center = CostCenter::new(100.0);
        center.reset();
        center.check_budget(30.0).unwrap();
        let decision = center.check_budget(40.0).unwrap();
        assert_eq!(decision, BudgetDecision::Continue);
        let decision = center.check_budget(50.0).unwrap();
        assert!(matches!(decision, BudgetDecision::Warn { .. }));
    }
}
