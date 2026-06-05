import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

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

const tableStyle: React.CSSProperties = {
  width: "100%",
  borderCollapse: "collapse",
  fontSize: "13px",
};

export default function CostCenter() {
  const [byAgent, setByAgent] = useState<CostBreakdown[]>([]);
  const [daily, setDaily] = useState<DailyCost[]>([]);
  const [totalCost, setTotalCost] = useState("0.00");

  useEffect(() => {
    invoke<{ dashboard: { agent_costs?: { name: string, cost: number, calls: number }[], daily_costs?: { date: string, cost: number }[] } }>("get_system_status").then((res) => {
      const ac = res.dashboard.agent_costs;
      if (ac && ac.length > 0) {
        const total = ac.reduce((s, a) => s + a.cost, 0);
        setByAgent(ac.map(a => ({
          ...a,
          percentage: total > 0 ? (a.cost / total) * 100 : 0,
        })));
      }
      const dc = res.dashboard.daily_costs;
      if (dc && dc.length > 0) {
        setDaily(dc);
        setTotalCost(dc.reduce((s, d) => s + d.cost, 0).toFixed(2));
      }
    }).catch(() => {});
  }, []);

  const maxDaily = Math.max(...daily.map(d => d.cost), 0.01);

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>Cost Center</h2>

      <div className="cost-card">
        <div style={{ color: "#8b949e", fontSize: "13px", marginBottom: "8px" }}>Total Cost (7 days)</div>
        <div style={{ fontSize: "28px", fontWeight: "bold", color: "#f85149" }}>¥{totalCost}</div>
      </div>

      <div className="cost-card">
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>By Agent</div>
        {byAgent.length === 0 ? (
          <div style={{ color: "#8b949e" }}>No cost data available</div>
        ) : (
          <table style={tableStyle}>
            <thead>
              <tr style={{ color: "#8b949e", borderBottom: "1px solid #30363d" }}>
                <td style={{ padding: "6px 8px" }}>Agent</td><td style={{ padding: "6px 8px" }}>Cost</td><td style={{ padding: "6px 8px" }}>Calls</td><td style={{ padding: "6px 8px" }}>%</td>
              </tr>
            </thead>
            <tbody>
              {byAgent.map(a => (
                <tr key={a.name} style={{ borderBottom: "1px solid #21262d", color: "#e6edf3" }}>
                  <td style={{ padding: "6px 8px" }}>{a.name}</td>
                  <td style={{ padding: "6px 8px" }}>¥{a.cost.toFixed(2)}</td>
                  <td style={{ padding: "6px 8px" }}>{a.calls}</td>
                  <td style={{ padding: "6px 8px" }}>{a.percentage.toFixed(1)}%</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      <div className="cost-card">
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>Daily Trend</div>
        {daily.length === 0 ? (
          <div style={{ color: "#8b949e" }}>No daily cost data available</div>
        ) : (
          <div style={{ display: "flex", alignItems: "flex-end", gap: "8px", height: "80px" }}>
            {daily.map(d => (
              <div key={d.date} style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center" }}>
                <div style={{ height: `${(d.cost / maxDaily) * 70}px`, width: "100%", background: "#58a6ff", borderRadius: "4px 4px 0 0", minHeight: "4px" }} />
                <div style={{ color: "#8b949e", fontSize: "10px", marginTop: "4px" }}>{d.date}</div>
                <div style={{ color: "#e6edf3", fontSize: "10px" }}>¥{d.cost.toFixed(2)}</div>
              </div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}