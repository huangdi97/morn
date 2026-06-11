import { useState, useEffect } from "react";
import { api } from "../api";
import { TemplateSelector } from "../studio/TemplateSelector";

interface TrendPoint {
  label: string;
  value: number;
}

interface DashboardAlert {
  id: string;
  kind: "version" | "cost" | "security" | string;
  severity: "info" | "warning" | "critical" | string;
  title: string;
  detail: string;
}

interface DashboardData {
  total_tasks: number;
  success_rate: number;
  avg_latency_ms: number;
  today_cost: number;
  agent_count: number;
  active_channels: number;
  uptime_hours: number;
  request_trend?: TrendPoint[];
  latency_trend?: TrendPoint[];
  alerts?: DashboardAlert[];
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

const chartStyle: React.CSSProperties = {
  ...cardStyle,
  minHeight: "210px",
};

const defaultRequestTrend: TrendPoint[] = [
  { label: "Mon", value: 12 },
  { label: "Tue", value: 18 },
  { label: "Wed", value: 15 },
  { label: "Thu", value: 24 },
  { label: "Fri", value: 22 },
  { label: "Sat", value: 10 },
  { label: "Sun", value: 16 },
];

const defaultLatencyTrend: TrendPoint[] = [
  { label: "Mon", value: 960 },
  { label: "Tue", value: 1120 },
  { label: "Wed", value: 1040 },
  { label: "Thu", value: 1280 },
  { label: "Fri", value: 1180 },
  { label: "Sat", value: 900 },
  { label: "Sun", value: 1020 },
];

const ONBOARDING_KEY = "morn_onboarded";

const overlayStyle: React.CSSProperties = {
  position: "fixed", top: 0, left: 0, right: 0, bottom: 0,
  background: "rgba(0,0,0,0.7)", zIndex: 1000,
  display: "flex", alignItems: "center", justifyContent: "center",
};

const modalStyle: React.CSSProperties = {
  background: "#161b22", borderRadius: "12px",
  border: "1px solid #30363d", padding: "32px",
  maxWidth: "600px", width: "90%", maxHeight: "80vh", overflow: "auto",
};

export default function AdminDashboard() {
  const [data, setData] = useState<DashboardData>({
    total_tasks: 0,
    success_rate: 0,
    avg_latency_ms: 0,
    today_cost: 0,
    agent_count: 2,
    active_channels: 3,
    uptime_hours: 12.5,
    request_trend: defaultRequestTrend,
    latency_trend: defaultLatencyTrend,
    alerts: [],
  });
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [onboardingStep, setOnboardingStep] = useState(1);

  useEffect(() => {
    api.getSystemStatus().then((res: { dashboard: DashboardData; system_info: any }) => {
      setData(res.dashboard);
    }).catch(() => {});
  }, []);

  useEffect(() => {
    const onboarded = localStorage.getItem(ONBOARDING_KEY);
    if (!onboarded) {
      setShowOnboarding(true);
    }
  }, []);

  const completeOnboarding = () => {
    localStorage.setItem(ONBOARDING_KEY, "true");
    setShowOnboarding(false);
  };

  const cards = [
    { label: "Total Tasks", value: data.total_tasks.toString(), color: "#58a6ff" },
    { label: "Success Rate", value: `${(data.success_rate * 100).toFixed(0)}%`, color: "#3fb950" },
    { label: "Avg Latency", value: `${data.avg_latency_ms.toFixed(0)}ms`, color: "#d29922" },
    { label: "Today's Cost", value: `¥${data.today_cost.toFixed(3)}`, color: "#f85149" },
    { label: "Agents", value: data.agent_count.toString(), color: "#bc8cff" },
    { label: "Active Channels", value: data.active_channels.toString(), color: "#79c0ff" },
  ];

  const renderTrendChart = (title: string, points: TrendPoint[], color: string, unit = "") => {
    const width = 520;
    const height = 150;
    const padding = 28;
    const maxValue = Math.max(...points.map((point) => point.value), 1);
    const chartPoints = points.map((point, index) => {
      const x = padding + (index * (width - padding * 2)) / Math.max(points.length - 1, 1);
      const y = height - padding - (point.value / maxValue) * (height - padding * 2);
      return { ...point, x, y };
    });
    const path = chartPoints
      .map((point, index) => `${index === 0 ? "M" : "L"} ${point.x} ${point.y}`)
      .join(" ");

    return (
      <div style={chartStyle}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "10px" }}>
          <div style={{ color: "#e6edf3", fontWeight: 600 }}>{title}</div>
          <div style={{ color: "#8b949e", fontSize: "12px" }}>
            Peak {maxValue.toFixed(0)}{unit}
          </div>
        </div>
        <svg viewBox={`0 0 ${width} ${height}`} style={{ width: "100%", height: "150px", display: "block" }}>
          <line x1={padding} y1={height - padding} x2={width - padding} y2={height - padding} stroke="#30363d" />
          <polyline points={chartPoints.map((point) => `${point.x},${point.y}`).join(" ")} fill="none" stroke={`${color}33`} strokeWidth="9" strokeLinecap="round" strokeLinejoin="round" />
          <path d={path} fill="none" stroke={color} strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" />
          {chartPoints.map((point) => (
            <g key={point.label}>
              <circle cx={point.x} cy={point.y} r="4" fill={color} />
              <text x={point.x} y={height - 8} textAnchor="middle" fontSize="10" fill="#8b949e">{point.label}</text>
            </g>
          ))}
        </svg>
      </div>
    );
  };

  const renderAlerts = () => {
    const alerts = data.alerts ?? [];
    const severityColor = (severity: string) => {
      switch (severity) {
        case "critical": return "#f85149";
        case "warning": return "#d29922";
        default: return "#58a6ff";
      }
    };

    return (
      <div style={{ ...cardStyle, marginTop: "16px" }}>
        <div style={{ color: "#e6edf3", fontWeight: 600, marginBottom: "12px" }}>Alerts</div>
        {alerts.length === 0 && <div style={{ color: "#8b949e", fontSize: "13px" }}>No active alerts.</div>}
        <div style={{ display: "grid", gap: "10px" }}>
          {alerts.map((alert) => (
            <div key={alert.id} style={{ border: `1px solid ${severityColor(alert.severity)}55`, borderLeft: `3px solid ${severityColor(alert.severity)}`, borderRadius: "6px", padding: "10px 12px", background: "#0d1117" }}>
              <div style={{ color: severityColor(alert.severity), fontSize: "11px", textTransform: "uppercase" }}>{alert.kind}</div>
              <div style={{ color: "#e6edf3", fontWeight: 600, marginTop: "3px" }}>{alert.title}</div>
              <div style={{ color: "#8b949e", fontSize: "12px", marginTop: "4px" }}>{alert.detail}</div>
            </div>
          ))}
        </div>
      </div>
    );
  };

  const renderOnboarding = () => {
    return (
      <div style={overlayStyle}>
        <div style={modalStyle}>
          {onboardingStep === 1 && (
            <>
              <div style={{ fontSize: "48px", textAlign: "center", marginBottom: "16px" }}>🤖</div>
              <h2 style={{ color: "#e6edf3", textAlign: "center", margin: "0 0 8px 0" }}>欢迎使用 Morn</h2>
              <p style={{ color: "#8b949e", textAlign: "center", fontSize: "14px", lineHeight: "1.6", margin: "0 0 24px 0" }}>
                Morn 是一个智能 AI 助手平台，帮你构建和使用 AI Agent。
                你可以通过自然语言快速创建定制 Agent，或从模板开始。
              </p>
              <div style={{ display: "flex", gap: "12px", justifyContent: "center" }}>
                <button
                  onClick={completeOnboarding}
                  style={{ padding: "10px 24px", borderRadius: "6px", border: "1px solid #30363d", background: "transparent", color: "#8b949e", cursor: "pointer" }}
                >
                  跳过
                </button>
                <button
                  onClick={() => setOnboardingStep(2)}
                  style={{ padding: "10px 24px", borderRadius: "6px", border: "none", background: "#58a6ff", color: "#fff", cursor: "pointer" }}
                >
                  开始使用
                </button>
              </div>
            </>
          )}
          {onboardingStep === 2 && (
            <>
              <h3 style={{ color: "#e6edf3", margin: "0 0 16px 0" }}>选择你的第一个 Agent</h3>
              <p style={{ color: "#8b949e", fontSize: "14px", margin: "0 0 16px 0" }}>
                选择一个模板开始，你也可以之后在 Studio 中自定义。
              </p>
              <TemplateSelector
                onSelect={() => setOnboardingStep(3)}
              />
              <div style={{ textAlign: "center", marginTop: "16px" }}>
                <button
                  onClick={completeOnboarding}
                  style={{ padding: "8px 20px", borderRadius: "6px", border: "1px solid #30363d", background: "transparent", color: "#8b949e", cursor: "pointer" }}
                >
                  稍后再说
                </button>
              </div>
            </>
          )}
          {onboardingStep === 3 && (
            <>
              <div style={{ fontSize: "48px", textAlign: "center", marginBottom: "16px" }}>🚀</div>
              <h2 style={{ color: "#e6edf3", textAlign: "center", margin: "0 0 8px 0" }}>试试吧</h2>
              <p style={{ color: "#8b949e", textAlign: "center", fontSize: "14px", margin: "0 0 24px 0" }}>
                前往 Workbench 开始对话，或在 Studio 中创建你的第一个 Agent。
              </p>
              <div style={{ display: "flex", gap: "12px", justifyContent: "center" }}>
                <button
                  onClick={completeOnboarding}
                  style={{ padding: "10px 24px", borderRadius: "6px", border: "none", background: "#58a6ff", color: "#fff", cursor: "pointer" }}
                >
                  开始探索
                </button>
              </div>
            </>
          )}
        </div>
      </div>
    );
  };

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
      <div style={{ display: "grid", gridTemplateColumns: "repeat(2, minmax(0, 1fr))", gap: "12px", marginTop: "16px" }}>
        {renderTrendChart("Request Trend", data.request_trend ?? defaultRequestTrend, "#58a6ff")}
        {renderTrendChart("Latency Trend", data.latency_trend ?? defaultLatencyTrend, "#d29922", "ms")}
      </div>
      {renderAlerts()}
      {showOnboarding && renderOnboarding()}
    </div>
  );
}
