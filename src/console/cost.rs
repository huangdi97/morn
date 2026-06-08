//! cost — Defines console-facing cost tracking and reporting data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CostReport {
    pub total_cost: f64,
    pub by_agent: Vec<CostBreakdown>,
    pub by_tool: Vec<CostBreakdown>,
    pub by_model: Vec<CostBreakdown>,
    pub daily_trend: Vec<DailyCost>,
    pub monthly_trend: Vec<MonthlyCost>,
    pub budget: f64,
    pub budget_exceeded: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CostBreakdown {
    pub name: String,
    pub cost: f64,
    pub calls: u64,
    pub percentage: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DailyCost {
    pub date: String,
    pub cost: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonthlyCost {
    pub month: String,
    pub cost: f64,
}

pub struct CostCenter {
    budget: f64,
    budget_action: String,
    total_cost: f64,
    notification_count: u64,
}

impl CostCenter {
    pub fn new(budget: f64) -> Self {
        CostCenter {
            budget,
            budget_action: "warn".into(),
            total_cost: 12.45,
            notification_count: 0,
        }
    }

    pub fn get_report(&self) -> CostReport {
        CostReport {
            total_cost: self.total_cost,
            by_agent: vec![
                CostBreakdown {
                    name: "Chat Agent".into(),
                    cost: 8.20,
                    calls: 450,
                    percentage: 65.9,
                },
                CostBreakdown {
                    name: "Research Agent".into(),
                    cost: 3.15,
                    calls: 120,
                    percentage: 25.3,
                },
                CostBreakdown {
                    name: "Analyst Agent".into(),
                    cost: 1.10,
                    calls: 45,
                    percentage: 8.8,
                },
            ],
            by_tool: vec![
                CostBreakdown {
                    name: "web_search".into(),
                    cost: 5.50,
                    calls: 300,
                    percentage: 44.2,
                },
                CostBreakdown {
                    name: "llm_call".into(),
                    cost: 4.80,
                    calls: 250,
                    percentage: 38.6,
                },
                CostBreakdown {
                    name: "file_ops".into(),
                    cost: 1.15,
                    calls: 80,
                    percentage: 9.2,
                },
                CostBreakdown {
                    name: "calc".into(),
                    cost: 1.00,
                    calls: 70,
                    percentage: 8.0,
                },
            ],
            by_model: vec![
                CostBreakdown {
                    name: "deepseek-chat".into(),
                    cost: 8.50,
                    calls: 400,
                    percentage: 68.3,
                },
                CostBreakdown {
                    name: "deepseek-reasoner".into(),
                    cost: 3.95,
                    calls: 150,
                    percentage: 31.7,
                },
            ],
            daily_trend: vec![
                DailyCost {
                    date: "Mon".into(),
                    cost: 1.20,
                },
                DailyCost {
                    date: "Tue".into(),
                    cost: 2.30,
                },
                DailyCost {
                    date: "Wed".into(),
                    cost: 1.80,
                },
                DailyCost {
                    date: "Thu".into(),
                    cost: 3.10,
                },
                DailyCost {
                    date: "Fri".into(),
                    cost: 2.45,
                },
                DailyCost {
                    date: "Sat".into(),
                    cost: 0.80,
                },
                DailyCost {
                    date: "Sun".into(),
                    cost: 0.80,
                },
            ],
            monthly_trend: vec![
                MonthlyCost {
                    month: "Jan".into(),
                    cost: 45.0,
                },
                MonthlyCost {
                    month: "Feb".into(),
                    cost: 52.0,
                },
                MonthlyCost {
                    month: "Mar".into(),
                    cost: 48.0,
                },
            ],
            budget: self.budget,
            budget_exceeded: self.total_cost > self.budget,
        }
    }

    pub fn set_budget(&mut self, budget: f64) {
        self.budget = budget;
    }

    pub fn budget(&self) -> f64 {
        self.budget
    }

    pub fn set_budget_action(&mut self, action: &str) {
        self.budget_action = action.to_string();
    }

    pub fn budget_action(&self) -> &str {
        &self.budget_action
    }

    pub fn reset(&mut self) {
        self.total_cost = 0.0;
        self.notification_count = 0;
    }

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
    fn summary_returns_json_report() {
        let summary = CostCenter::new(100.0).summary();

        assert_eq!(summary["budget"], 100.0);
        assert_eq!(summary["budget_action"], "warn");
        assert!(summary["by_agent_count"].as_u64().unwrap() > 0);
    }
}
