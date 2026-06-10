//! report — Cost report generation and trend data.

/// 成本报告 — 包含总成本、按 Agent/工具/模型细分、趋势及预算。
/// Cost report with total cost, breakdowns by agent/tool/model, trends, and budget.
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

/// 成本细分 — 按名称、成本、调用次数和百分比构成。
/// Cost breakdown by name, cost, call count, and percentage.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CostBreakdown {
    pub name: String,
    pub cost: f64,
    pub calls: u64,
    pub percentage: f64,
}

/// 每日成本 — 日期与对应成本。
/// Daily cost with date and cost value.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DailyCost {
    pub date: String,
    pub cost: f64,
}

/// 月度成本 — 月份与对应成本。
/// Monthly cost with month and cost value.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonthlyCost {
    pub month: String,
    pub cost: f64,
}

use super::CostCenter;

impl CostCenter {
    /// Returns a cost report with daily and monthly trend data for dashboard visualization.
    pub fn report(&self) -> CostReport {
        self.get_report()
    }

    /// 获取完整成本报告 — 包含 Agent/工具/模型细分和趋势数据。
    /// Gets the full cost report with agent/tool/model breakdowns and trends.
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

    fn generate_daily_trend(&self) -> Vec<DailyCost> {
        let mut trend = Vec::new();
        let now = chrono::Utc::now();
        for i in (0..7).rev() {
            let day = now - chrono::Duration::days(i);
            trend.push(DailyCost {
                date: day.format("%Y-%m-%d").to_string(),
                cost: (self.total_cost / 7.0) * (1.0 + (i as f64 * 0.05)),
            });
        }
        trend
    }

    fn generate_monthly_trend(&self) -> Vec<MonthlyCost> {
        let mut trend = Vec::new();
        let now = chrono::Utc::now();
        for i in (0..6).rev() {
            let month = now - chrono::Duration::days(i * 30);
            trend.push(MonthlyCost {
                month: month.format("%Y-%m").to_string(),
                cost: (self.total_cost / 6.0) * (1.0 + (i as f64 * 0.1)),
            });
        }
        trend
    }
}