import { useState, useEffect } from "react";
import { api } from "../api";
import { TemplateSelector } from "../studio/TemplateSelector";
import ExecutionHistory from "./ExecutionHistory";
import { useTranslation } from '../i18n';

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
  background: "var(--bg-surface)", borderRadius: "var(--radius-xl)",
  border: "1px solid var(--border-default)", padding: "32px",
  maxWidth: "600px", width: "90%", maxHeight: "80vh", overflow: "auto",
};

const cardGradient: Record<string, string> = {
  "#58a6ff": "accent-blue",
  "#3fb950": "accent-green",
  "#d29922": "accent-yellow",
  "#f85149": "accent-red",
  "#bc8cff": "accent-purple",
  "#79c0ff": "accent-cyan",
};

export default function AdminDashboard() {
  const { t } = useTranslation();
  const [loading, setLoading] = useState(true);
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
      setLoading(false);
    }).catch(() => {
      setLoading(false);
    });
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
      <div className="dashboard-chart-card">
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "10px" }}>
          <div style={{ color: "var(--text-primary)", fontWeight: 600 }}>{title}</div>
          <div style={{ color: "var(--text-tertiary)", fontSize: "12px" }}>
            Peak {maxValue.toFixed(0)}{unit}
          </div>
        </div>
        <svg viewBox={`0 0 ${width} ${height}`} style={{ width: "100%", height: "150px", display: "block" }}>
          <line x1={padding} y1={height - padding} x2={width - padding} y2={height - padding} stroke="var(--border-default)" />
          <polyline points={chartPoints.map((point) => `${point.x},${point.y}`).join(" ")} fill="none" stroke={`${color}33`} strokeWidth="9" strokeLinecap="round" strokeLinejoin="round" />
          <path d={path} fill="none" stroke={color} strokeWidth="3" strokeLinecap="round" strokeLinejoin="round" />
          {chartPoints.map((point) => (
            <g key={point.label}>
              <circle cx={point.x} cy={point.y} r="4" fill={color} />
              <text x={point.x} y={height - 8} textAnchor="middle" fontSize="10" fill="var(--text-tertiary)">{point.label}</text>
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
      <div className="dashboard-card dashboard-alerts">
        <div style={{ color: "var(--text-primary)", fontWeight: 600, marginBottom: "12px" }}>{t('console.dashboard.alerts')}</div>
        {alerts.length === 0 && <div style={{ color: "var(--text-tertiary)", fontSize: "13px" }}>{t('console.dashboard.no_alerts')}</div>}
        <div style={{ display: "grid", gap: "10px" }}>
          {alerts.map((alert) => (
            <div key={alert.id} style={{ border: `1px solid ${severityColor(alert.severity)}55`, borderLeft: `3px solid ${severityColor(alert.severity)}`, borderRadius: "6px", padding: "10px 12px", background: "var(--bg-page)" }}>
              <div style={{ color: severityColor(alert.severity), fontSize: "11px", textTransform: "uppercase" }}>{alert.kind}</div>
              <div style={{ color: "var(--text-primary)", fontWeight: 600, marginTop: "3px" }}>{alert.title}</div>
              <div style={{ color: "var(--text-tertiary)", fontSize: "12px", marginTop: "4px" }}>{alert.detail}</div>
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
              <h2 style={{ color: "var(--text-primary)", textAlign: "center", margin: "0 0 8px 0" }}>{t('console.dashboard.welcome')}</h2>
              <p style={{ color: "var(--text-tertiary)", textAlign: "center", fontSize: "14px", lineHeight: "1.6", margin: "0 0 24px 0" }}>
                Morn 是一个智能 AI 助手平台，帮你构建和使用 AI Agent。
                你可以通过自然语言快速创建定制 Agent，或从模板开始。
              </p>
              <div style={{ display: "flex", gap: "12px", justifyContent: "center" }}>
                <button
                  onClick={completeOnboarding}
                  style={{ padding: "10px 24px", borderRadius: "6px", border: "1px solid var(--border-default)", background: "transparent", color: "var(--text-tertiary)", cursor: "pointer" }}
                >
                  跳过
                </button>
                <button
                  onClick={() => setOnboardingStep(2)}
                  style={{ padding: "10px 24px", borderRadius: "6px", border: "none", background: "var(--accent-brand)", color: "#fff", cursor: "pointer" }}
                >
                  开始使用
                </button>
              </div>
            </>
          )}
          {onboardingStep === 2 && (
            <>
              <h3 style={{ color: "var(--text-primary)", margin: "0 0 16px 0" }}>{t('console.dashboard.select_agent')}</h3>
              <p style={{ color: "var(--text-tertiary)", fontSize: "14px", margin: "0 0 16px 0" }}>
                选择一个模板开始，你也可以之后在 Studio 中自定义。
              </p>
              <TemplateSelector
                onSelect={() => setOnboardingStep(3)}
              />
              <div style={{ textAlign: "center", marginTop: "16px" }}>
                <button
                  onClick={completeOnboarding}
                  style={{ padding: "8px 20px", borderRadius: "6px", border: "1px solid var(--border-default)", background: "transparent", color: "var(--text-tertiary)", cursor: "pointer" }}
                >
                  稍后再说
                </button>
              </div>
            </>
          )}
          {onboardingStep === 3 && (
            <>
              <div style={{ fontSize: "48px", textAlign: "center", marginBottom: "16px" }}>🚀</div>
              <h2 style={{ color: "var(--text-primary)", textAlign: "center", margin: "0 0 8px 0" }}>{t('console.dashboard.try_it')}</h2>
              <p style={{ color: "var(--text-tertiary)", textAlign: "center", fontSize: "14px", margin: "0 0 24px 0" }}>
                前往 Workbench 开始对话，或在 Studio 中创建你的第一个 Agent。
              </p>
              <div style={{ display: "flex", gap: "12px", justifyContent: "center" }}>
                <button
                  onClick={completeOnboarding}
                  style={{ padding: "10px 24px", borderRadius: "6px", border: "none", background: "var(--accent-brand)", color: "#fff", cursor: "pointer" }}
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

  if (loading) {
    return (
      <div className="console-grid">
        {[1,2,3,4,5,6].map(i => (
          <div key={i} className="skeleton skeleton-console-card" style={{ height: '100px' }} />
        ))}
        <div className="skeleton" style={{ gridColumn: '1 / -1', height: '60px', marginTop: '8px' }} />
        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '12px', gridColumn: '1 / -1' }}>
          <div className="skeleton" style={{ height: '200px' }} />
          <div className="skeleton" style={{ height: '200px' }} />
        </div>
      </div>
    );
  }

  return (
    <div>
      <h2 style={{ color: "var(--text-primary)", marginBottom: "16px" }}>{t('console.dashboard.title')}</h2>
      <div className="dashboard-grid">
        {cards.map((card) => (
          <div key={card.label} className={`dashboard-card ${cardGradient[card.color] ?? ""}`}>
            <div className="dashboard-card-label">{card.label}</div>
            <div className="dashboard-card-value" style={{ color: card.color }}>{card.value}</div>
          </div>
        ))}
      </div>
      <div className={`dashboard-card dashboard-card-uptime`} style={{ marginTop: "var(--space-md)" }}>
        <div className="dashboard-card-label">{t('console.dashboard.uptime')}</div>
        <div className="dashboard-card-value" style={{ color: "var(--accent-brand)" }}>{data.uptime_hours.toFixed(1)} hours</div>
      </div>
      <div className="dashboard-chart-grid">
        {renderTrendChart("Request Trend", data.request_trend ?? defaultRequestTrend, "#58a6ff")}
        {renderTrendChart("Latency Trend", data.latency_trend ?? defaultLatencyTrend, "#d29922", "ms")}
      </div>
      {renderAlerts()}
      <ExecutionHistory />
      {showOnboarding && renderOnboarding()}
    </div>
  );
}