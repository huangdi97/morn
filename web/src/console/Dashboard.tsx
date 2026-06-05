import { useState, useEffect } from "react";

interface DashboardData {
  total_tasks: number;
  success_rate: number;
  avg_latency_ms: number;
  today_cost: number;
  agent_count: number;
  active_channels: number;
  uptime_hours: number;
}

const cardStyle: React.CSSProperties = {
  background: "#161b22",
  borderRadius: "8px",
  padding: "16px",
  border: "1px solid #30363d",
};

const valueStyle: React.CSSProperties = {
  fontSize: "24px",
  fontWeight: "bold",
  color: "#58a6ff",
  marginTop: "8px",
};

export default function Dashboard() {
  const [data, setData] = useState<DashboardData>({
    total_tasks: 0,
    success_rate: 0,
    avg_latency_ms: 0,
    today_cost: 0,
    agent_count: 2,
    active_channels: 3,
    uptime_hours: 12.5,
  });

  useEffect(() => {
    const fetchData = async () => {
      try {
        const result = await (window as any).__TAURI_INTERNALS__?.invoke?.("get_dashboard");
        if (result) setData(result);
      } catch {
        // fallback to defaults
      }
    };
    fetchData();
  }, []);

  const cards = [
    { label: "Total Tasks", value: data.total_tasks.toString(), color: "#58a6ff" },
    { label: "Success Rate", value: `${(data.success_rate * 100).toFixed(0)}%`, color: "#3fb950" },
    { label: "Avg Latency", value: `${data.avg_latency_ms.toFixed(0)}ms`, color: "#d29922" },
    { label: "Today's Cost", value: `¥${data.today_cost.toFixed(3)}`, color: "#f85149" },
    { label: "Agents", value: data.agent_count.toString(), color: "#bc8cff" },
    { label: "Active Channels", value: data.active_channels.toString(), color: "#79c0ff" },
  ];

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>Dashboard</h2>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: "12px" }}>
        {cards.map((card) => (
          <div key={card.label} style={cardStyle}>
            <div style={{ color: "#8b949e", fontSize: "13px" }}>{card.label}</div>
            <div style={{ ...valueStyle, color: card.color }}>{card.value}</div>
          </div>
        ))}
      </div>
      <div style={{ ...cardStyle, marginTop: "16px" }}>
        <div style={{ color: "#8b949e", fontSize: "13px" }}>Uptime</div>
        <div style={valueStyle}>{data.uptime_hours.toFixed(1)} hours</div>
      </div>
    </div>
  );
}