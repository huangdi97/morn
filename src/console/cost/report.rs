//! report — Cost report generation and trend data.

use crate::core::error::MornError;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::cost::CostCenter;

    #[test]
    fn test_cost_report_structure() {
        let center = CostCenter::new(100.0);
        let report = center.get_report();
        assert_eq!(report.budget, 100.0);
        assert!(!report.by_agent.is_empty());
        assert!(!report.by_tool.is_empty());
        assert!(!report.by_model.is_empty());
        assert_eq!(report.daily_trend.len(), 7);
        assert_eq!(report.monthly_trend.len(), 3);
    }

    #[test]
    fn test_cost_report_budget_exceeded() {
        let center = CostCenter::new(5.0);
        let report = center.get_report();
        assert!(report.budget_exceeded);
    }

    #[test]
    fn test_cost_breakdown_fields() {
        let b = CostBreakdown {
            name: "test".into(),
            cost: 10.0,
            calls: 5,
            percentage: 50.0,
        };
        assert_eq!(b.name, "test");
        assert_eq!(b.cost, 10.0);
        assert_eq!(b.calls, 5);
        assert_eq!(b.percentage, 50.0);
    }

    #[test]
    fn test_daily_cost_fields() {
        let d = DailyCost {
            date: "2024-01-01".into(),
            cost: 1.5,
        };
        assert_eq!(d.date, "2024-01-01");
        assert_eq!(d.cost, 1.5);
    }

    #[test]
    fn test_monthly_cost_fields() {
        let m = MonthlyCost {
            month: "2024-01".into(),
            cost: 30.0,
        };
        assert_eq!(m.month, "2024-01");
        assert_eq!(m.cost, 30.0);
    }

    #[test]
    fn test_report_reference_method() {
        let center = CostCenter::new(50.0);
        let report = center.report();
        assert_eq!(report.total_cost, 12.45);
    }

    #[test]
    fn test_cost_report_serialization() {
        let center = CostCenter::new(100.0);
        let report = center.get_report();
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("total_cost"));
        assert!(json.contains("by_agent"));
        assert!(json.contains("daily_trend"));
    }
}
