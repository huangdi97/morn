import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from '../i18n';

interface CostBreakdown {
  name: string;
  cost: number;
  calls: number;
  percentage: number;
}

interface DailyCost {
  date: string;
  cost: number;
}

const BUDGET_KEY = "morn_cost_budget";

export default function CostPanel() {
  const { t } = useTranslation();
  const [summary, setSummary] = useState("");
  const [daily, setDaily] = useState<DailyCost[]>([]);
  const [byAgent, setByAgent] = useState<CostBreakdown[]>([]);
  const [trendMode, setTrendMode] = useState<"daily" | "weekly">("daily");
  const [monthlyBudget, setMonthlyBudget] = useState(() => {
    return localStorage.getItem(`${BUDGET_KEY}_monthly`) || "";
  });
  const [dailyBudget, setDailyBudget] = useState(() => {
    return localStorage.getItem(`${BUDGET_KEY}_daily`) || "";
  });
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<string>("get_cost_summary").then(setSummary).catch(console.error);
    invoke<{ daily_costs: DailyCost[], agent_costs: { name: string, cost: number, calls: number }[] }>("get_cost_details")
      .then((res) => {
        setDaily(res.daily_costs || []);
        const ac = res.agent_costs || [];
        const total = ac.reduce((s, a) => s + a.cost, 0);
        setByAgent(ac.map(a => ({
          ...a,
          percentage: total > 0 ? (a.cost / total) * 100 : 0,
        })));
      })
      .catch(console.error);
  }, []);

  const saveBudget = () => {
    localStorage.setItem(`${BUDGET_KEY}_monthly`, monthlyBudget);
    localStorage.setItem(`${BUDGET_KEY}_daily`, dailyBudget);
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const mb = parseFloat(monthlyBudget) || 0;
  const db = parseFloat(dailyBudget) || 0;
  const totalMonthDays = new Date(new Date().getFullYear(), new Date().getMonth() + 1, 0).getDate();
  const projectedCost = daily.length > 0
    ? (daily.reduce((s, d) => s + d.cost, 0) / daily.length) * totalMonthDays
    : 0;

  const weeklyData = daily.reduce<{ week: string; cost: number }[]>((acc, d) => {
    const date = new Date(d.date);
    const weekStart = new Date(date);
    weekStart.setDate(date.getDate() - date.getDay());
    const key = weekStart.toISOString().slice(0, 10);
    const existing = acc.find(w => w.week === key);
    if (existing) {
      existing.cost += d.cost;
    } else {
      acc.push({ week: key, cost: d.cost });
    }
    return acc;
  }, []);

  const trendData = trendMode === "daily" ? daily : weeklyData;
  const maxTrend = Math.max(...trendData.map(d => d.cost), 0.01);

  const trendBarStyle = (value: number) => ({
    height: `${(value / maxTrend) * 120}px`,
    minHeight: "4px",
    width: "100%",
    background: "linear-gradient(to top, #58a6ff, #79c0ff)",
    borderRadius: "4px 4px 0 0",
    transition: "height 0.3s ease",
  });

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.cost_tracking.title')}</h2>

      <div className="cost-grid" style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "12px", marginBottom: "16px" }}>
        <div className="cost-card">
          <div style={{ color: "#8b949e", fontSize: "13px", marginBottom: "8px" }}>Summary</div>
          <div style={{ color: "#e6edf3", fontSize: "14px" }}>{summary || "Loading..."}</div>
        </div>
        <div className="cost-card">
          <div style={{ color: "#8b949e", fontSize: "13px", marginBottom: "8px" }}>{t('console.cost_panel.projected_cost')}</div>
          <div style={{ fontSize: "28px", fontWeight: "bold", color: mb > 0 && projectedCost > mb ? "#f85149" : "#3fb950" }}>
            ¥{projectedCost.toFixed(2)}
            {mb > 0 && (
              <span style={{ fontSize: "13px", fontWeight: "normal", marginLeft: "8px", color: projectedCost > mb ? "#f85149" : "#3fb950" }}>
                {projectedCost > mb ? `+${((projectedCost - mb) / mb * 100).toFixed(0)}%` : `${((1 - projectedCost / mb) * 100).toFixed(0)}%`}
              </span>
            )}
          </div>
        </div>
      </div>

      <div className="cost-card" style={{ marginBottom: "16px" }}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "12px" }}>
          <div style={{ color: "#e6edf3", fontWeight: "bold" }}>{t('console.cost_panel.budget_title')}</div>
          <button
            onClick={saveBudget}
            style={{
              padding: "4px 14px", fontSize: "12px", border: "none",
              borderRadius: "6px", background: saved ? "#3fb950" : "var(--accent-brand)",
              color: "#fff", cursor: "pointer",
            }}
          >
            {saved ? t('console.cost_panel.budget_saved') : t('console.cost_panel.save_budget')}
          </button>
        </div>
        <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "12px" }}>
          <div>
            <div style={{ color: "#8b949e", fontSize: "12px", marginBottom: "4px" }}>{t('console.cost_panel.monthly_budget')}</div>
            <input
              type="number"
              value={monthlyBudget}
              onChange={(e) => setMonthlyBudget(e.target.value)}
              placeholder="e.g. 500"
              style={{
                width: "100%", padding: "8px 10px", borderRadius: "6px",
                border: "1px solid var(--border-default)", background: "var(--bg-page)",
                color: "#e6edf3", fontSize: "14px", boxSizing: "border-box",
              }}
            />
          </div>
          <div>
            <div style={{ color: "#8b949e", fontSize: "12px", marginBottom: "4px" }}>{t('console.cost_panel.daily_budget')}</div>
            <input
              type="number"
              value={dailyBudget}
              onChange={(e) => setDailyBudget(e.target.value)}
              placeholder="e.g. 20"
              style={{
                width: "100%", padding: "8px 10px", borderRadius: "6px",
                border: "1px solid var(--border-default)", background: "var(--bg-page)",
                color: "#e6edf3", fontSize: "14px", boxSizing: "border-box",
              }}
            />
          </div>
        </div>
        {db > 0 && (
          <div style={{ marginTop: "8px", height: "6px", background: "#21262d", borderRadius: "3px", overflow: "hidden" }}>
            <div style={{
              height: "100%", width: `${Math.min((daily.length > 0 ? daily[daily.length - 1].cost / db : 0) * 100, 100)}%`,
              background: daily.length > 0 && daily[daily.length - 1].cost > db ? "#f85149" : "#3fb950",
              borderRadius: "3px", transition: "width 0.3s ease",
            }} />
          </div>
        )}
      </div>

      <div className="cost-card" style={{ marginBottom: "16px" }}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "12px" }}>
          <div style={{ color: "#e6edf3", fontWeight: "bold" }}>{t('console.cost_panel.trend_title')}</div>
          <div style={{ display: "flex", gap: "4px" }}>
            <button
              onClick={() => setTrendMode("daily")}
              style={{
                padding: "2px 10px", fontSize: "11px", borderRadius: "4px",
                border: "1px solid var(--border-default)",
                background: trendMode === "daily" ? "var(--accent-brand)" : "transparent",
                color: trendMode === "daily" ? "#fff" : "var(--text-tertiary)",
                cursor: "pointer",
              }}
            >
              {t('console.cost_panel.daily')}
            </button>
            <button
              onClick={() => setTrendMode("weekly")}
              style={{
                padding: "2px 10px", fontSize: "11px", borderRadius: "4px",
                border: "1px solid var(--border-default)",
                background: trendMode === "weekly" ? "var(--accent-brand)" : "transparent",
                color: trendMode === "weekly" ? "#fff" : "var(--text-tertiary)",
                cursor: "pointer",
              }}
            >
              {t('console.cost_panel.weekly')}
            </button>
          </div>
        </div>
        {trendData.length === 0 ? (
          <div style={{ color: "#8b949e" }}>{t('console.cost_panel.no_data')}</div>
        ) : (
          <div style={{ display: "flex", alignItems: "flex-end", gap: "6px", height: "140px" }}>
            {trendData.map(d => (
              <div key={"week" in d ? (d as any).week : (d as any).date} style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center" }}>
                <div style={trendBarStyle(d.cost)} />
                <div style={{ color: "#8b949e", fontSize: "9px", marginTop: "4px" }}>
                  {"week" in d ? (d as any).week.slice(5) : (d as any).date.slice(5)}
                </div>
                <div style={{ color: "#e6edf3", fontSize: "9px" }}>¥{d.cost.toFixed(1)}</div>
              </div>
            ))}
          </div>
        )}
      </div>

      <div className="cost-card">
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>{t('console.cost_panel.by_agent')}</div>
        {byAgent.length === 0 ? (
          <div style={{ color: "#8b949e" }}>{t('console.cost_panel.no_data')}</div>
        ) : (
          <table style={{ width: "100%", borderCollapse: "collapse", fontSize: "13px" }}>
            <thead>
              <tr style={{ color: "#8b949e", borderBottom: "1px solid #30363d" }}>
                <td style={{ padding: "6px 8px" }}>{t('console.cost_panel.agent')}</td>
                <td style={{ padding: "6px 8px" }}>{t('console.cost_panel.cost')}</td>
                <td style={{ padding: "6px 8px" }}>{t('console.cost_panel.calls')}</td>
                <td style={{ padding: "6px 8px" }}>{t('console.cost_panel.percentage')}</td>
              </tr>
            </thead>
            <tbody>
              {byAgent.map(a => (
                <tr key={a.name} style={{ borderBottom: "1px solid #21262d", color: "#e6edf3" }}>
                  <td style={{ padding: "6px 8px" }}>{a.name}</td>
                  <td style={{ padding: "6px 8px" }}>¥{a.cost.toFixed(2)}</td>
                  <td style={{ padding: "6px 8px" }}>{a.calls}</td>
                  <td style={{ padding: "6px 8px" }}>
                    <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                      <div style={{ width: "50px", height: "6px", background: "#21262d", borderRadius: "3px", overflow: "hidden" }}>
                        <div style={{ width: `${a.percentage}%`, height: "100%", background: "#58a6ff", borderRadius: "3px" }} />
                      </div>
                      <span>{a.percentage.toFixed(1)}%</span>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </div>
  );
}