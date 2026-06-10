use super::CostCenter;

impl CostCenter {
    /// 设置预算上限 — Updates the budget threshold.
    pub fn set_budget(&mut self, budget: f64) {
        self.budget = budget;
    }

    /// 获取当前预算 — Returns the current budget.
    pub fn budget(&self) -> f64 {
        self.budget
    }

    /// 设置预算超支动作 — `warn` | `block` | `notify`.
    /// Sets the budget exceeded action: `warn` | `block` | `notify`.
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

    /// 检查成本并执行动作 — 累加成本，超预算则按配置动作响应。
    /// Checks cost and acts — accumulates cost, triggers action if over budget.
    pub fn check_and_act(&mut self, cost: f64) -> Result<(), String> {
        self.total_cost += cost;
        if self.total_cost <= self.budget {
            return Ok(());
        }

        match self.budget_action.as_str() {
            "warn" => {
                tracing::warn!(
                    "budget exceeded: total cost {:.2} is above budget {:.2}",
                    self.total_cost,
                    self.budget
                );
                Ok(())
            }
            "block" => Err(format!(
                "budget exceeded: total cost {:.2} is above budget {:.2}",
                self.total_cost, self.budget
            )),
            "notify" => {
                self.notification_count += 1;
                tracing::info!(
                    "budget notification emitted: total cost {:.2}, budget {:.2}",
                    self.total_cost,
                    self.budget
                );
                Ok(())
            }
            other => Err(format!("unknown budget action '{}'", other)),
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