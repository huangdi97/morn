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
        CostCenter { budget, budget_action: "warn".into() }
    }

    pub fn get_report(&self) -> CostReport {
        CostReport {
            total_cost: 12.45,
            by_agent: vec![
                CostBreakdown { name: "Chat Agent".into(), cost: 8.20, calls: 450, percentage: 65.9 },
                CostBreakdown { name: "Research Agent".into(), cost: 3.15, calls: 120, percentage: 25.3 },
                CostBreakdown { name: "Analyst Agent".into(), cost: 1.10, calls: 45, percentage: 8.8 },
            ],
            by_tool: vec![
                CostBreakdown { name: "web_search".into(), cost: 5.50, calls: 300, percentage: 44.2 },
                CostBreakdown { name: "llm_call".into(), cost: 4.80, calls: 250, percentage: 38.6 },
                CostBreakdown { name: "file_ops".into(), cost: 1.15, calls: 80, percentage: 9.2 },
                CostBreakdown { name: "calc".into(), cost: 1.00, calls: 70, percentage: 8.0 },
            ],
            by_model: vec![
                CostBreakdown { name: "deepseek-chat".into(), cost: 8.50, calls: 400, percentage: 68.3 },
                CostBreakdown { name: "deepseek-reasoner".into(), cost: 3.95, calls: 150, percentage: 31.7 },
            ],
            daily_trend: vec![
                DailyCost { date: "Mon".into(), cost: 1.20 },
                DailyCost { date: "Tue".into(), cost: 2.30 },
                DailyCost { date: "Wed".into(), cost: 1.80 },
                DailyCost { date: "Thu".into(), cost: 3.10 },
                DailyCost { date: "Fri".into(), cost: 2.45 },
                DailyCost { date: "Sat".into(), cost: 0.80 },
                DailyCost { date: "Sun".into(), cost: 0.80 },
            ],
            monthly_trend: vec![
                MonthlyCost { month: "Jan".into(), cost: 45.0 },
                MonthlyCost { month: "Feb".into(), cost: 52.0 },
                MonthlyCost { month: "Mar".into(), cost: 48.0 },
            ],
            budget: self.budget,
            budget_exceeded: 12.45 > self.budget,
        }
    }

    pub fn set_budget(&mut self, budget: f64) {
        self.budget = budget;
    }

    pub fn budget(&self) -> f64 { self.budget }

    pub fn set_budget_action(&mut self, action: &str) {
        self.budget_action = action.to_string();
    }

    pub fn budget_action(&self) -> &str { &self.budget_action }
}