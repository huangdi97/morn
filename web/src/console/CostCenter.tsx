import { useState } from "react";

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

const cardStyle: React.CSSProperties = {
  background: "#161b22",
  borderRadius: "8px",
  padding: "16px",
  border: "1px solid #30363d",
  marginBottom: "12px",
};

const tableStyle: React.CSSProperties = {
  width: "100%",
  borderCollapse: "collapse",
  fontSize: "13px",
};

export default function CostCenter() {
  const [byAgent] = useState<CostBreakdown[]>([
    { name: "Chat Agent", cost: 8.20, calls: 450, percentage: 65.9 },
    { name: "Research Agent", cost: 3.15, calls: 120, percentage: 25.3 },
    { name: "Analyst Agent", cost: 1.10, calls: 45, percentage: 8.8 },
  ]);

  const [daily] = useState<DailyCost[]>([
    { date: "Mon", cost: 1.20 },
    { date: "Tue", cost: 2.30 },
    { date: "Wed", cost: 1.80 },
    { date: "Thu", cost: 3.10 },
    { date: "Fri", cost: 2.45 },
    { date: "Sat", cost: 0.80 },
    { date: "Sun", cost: 0.80 },
  ]);

  const maxDaily = Math.max(...daily.map(d => d.cost), 0.01);

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>Cost Center</h2>

      <div style={cardStyle}>
        <div style={{ color: "#8b949e", fontSize: "13px", marginBottom: "8px" }}>Total Cost (7 days)</div>
        <div style={{ fontSize: "28px", fontWeight: "bold", color: "#f85149" }}>¥12.45</div>
      </div>

      <div style={cardStyle}>
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>By Agent</div>
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
      </div>

      <div style={cardStyle}>
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>Daily Trend</div>
        <div style={{ display: "flex", alignItems: "flex-end", gap: "8px", height: "80px" }}>
          {daily.map(d => (
            <div key={d.date} style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center" }}>
              <div style={{ height: `${(d.cost / maxDaily) * 70}px`, width: "100%", background: "#58a6ff", borderRadius: "4px 4px 0 0", minHeight: "4px" }} />
              <div style={{ color: "#8b949e", fontSize: "10px", marginTop: "4px" }}>{d.date}</div>
              <div style={{ color: "#e6edf3", fontSize: "10px" }}>¥{d.cost.toFixed(2)}</div>
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}