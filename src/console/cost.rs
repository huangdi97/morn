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
}

impl CostCenter {
    pub fn new(budget: f64) -> Self {
        CostCenter {
            budget,
            budget_action: "warn".into(),
        }
    }

    pub fn get_report(&self) -> CostReport {
        CostReport {
            total_cost: 12.45,
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
            budget_exceeded: 12.45 > self.budget,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_cost_center() {
        let cc = CostCenter::new(100.0);
        assert_eq!(cc.budget(), 100.0);
        assert_eq!(cc.budget_action(), "warn");
    }

    #[test]
    fn test_get_report_under_budget() {
        let cc = CostCenter::new(100.0);
        let report = cc.get_report();
        assert_eq!(report.total_cost, 12.45);
        assert!(!report.budget_exceeded);
        assert_eq!(report.budget, 100.0);
    }

    #[test]
    fn test_get_report_exceeds_budget() {
        let cc = CostCenter::new(5.0);
        let report = cc.get_report();
        assert_eq!(report.total_cost, 12.45);
        assert!(report.budget_exceeded);
    }

    #[test]
    fn test_set_budget() {
        let mut cc = CostCenter::new(50.0);
        cc.set_budget(200.0);
        assert_eq!(cc.budget(), 200.0);
    }

    #[test]
    fn test_set_budget_action() {
        let mut cc = CostCenter::new(50.0);
        cc.set_budget_action("block");
        assert_eq!(cc.budget_action(), "block");
    }

    #[test]
    fn test_report_has_agents() {
        let cc = CostCenter::new(100.0);
        let report = cc.get_report();
        assert_eq!(report.by_agent.len(), 3);
        assert_eq!(report.by_agent[0].name, "Chat Agent");
        assert_eq!(report.by_agent[0].calls, 450);
    }

    #[test]
    fn test_report_has_tools() {
        let cc = CostCenter::new(100.0);
        let report = cc.get_report();
        assert_eq!(report.by_tool.len(), 4);
        assert_eq!(report.by_tool[0].name, "web_search");
    }

    #[test]
    fn test_report_has_models() {
        let cc = CostCenter::new(100.0);
        let report = cc.get_report();
        assert_eq!(report.by_model.len(), 2);
        assert_eq!(report.by_model[0].name, "deepseek-chat");
    }

    #[test]
    fn test_report_has_daily_trend() {
        let cc = CostCenter::new(100.0);
        let report = cc.get_report();
        assert_eq!(report.daily_trend.len(), 7);
    }

    #[test]
    fn test_report_has_monthly_trend() {
        let cc = CostCenter::new(100.0);
        let report = cc.get_report();
        assert_eq!(report.monthly_trend.len(), 3);
    }

    #[test]
    fn test_cost_breakdown_percentages() {
        let cc = CostCenter::new(100.0);
        let report = cc.get_report();
        let total: f64 = report.by_agent.iter().map(|b| b.percentage).sum();
        assert!((total - 100.0).abs() < 0.1);
    }
}
