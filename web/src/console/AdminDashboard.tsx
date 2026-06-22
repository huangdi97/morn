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

interface OnboardingProps {
  onNavigate?: (tab: string) => void;
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

const TOTAL_STEPS = 5;

export default function AdminDashboard({ onNavigate }: OnboardingProps) {
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

  const renderProgressDots = () => (
    <div style={{ display: "flex", gap: "8px", justifyContent: "center", marginBottom: "16px" }}>
      {Array.from({ length: TOTAL_STEPS }, (_, i) => (
        <div
          key={i}
          style={{
            width: "8px", height: "8px", borderRadius: "50%",
            background: i + 1 === onboardingStep ? "var(--accent-brand)" : i + 1 < onboardingStep ? "#3fb950" : "var(--border-default)",
            transition: "background 0.3s ease",
          }}
        />
      ))}
    </div>
  );

  const renderStepIndicator = () => (
    <div style={{ textAlign: "center", marginBottom: "8px" }}>
      <span style={{ color: "var(--text-tertiary)", fontSize: "12px" }}>
        {t('console.dashboard.step_of', { current: onboardingStep, total: TOTAL_STEPS })}
      </span>
    </div>
  );

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
          {renderProgressDots()}
          {renderStepIndicator()}
          {onboardingStep === 1 && (
            <>
              <div style={{ fontSize: "48px", textAlign: "center", marginBottom: "16px" }}>🤖</div>
              <h2 style={{ color: "var(--text-primary)", textAlign: "center", margin: "0 0 8px 0" }}>{t('console.dashboard.welcome')}</h2>
              <p style={{ color: "var(--text-tertiary)", textAlign: "center", fontSize: "14px", lineHeight: "1.6", margin: "0 0 24px 0" }}>
                {t('console.dashboard.welcome_desc')}
              </p>
              <div style={{ display: "flex", gap: "12px", justifyContent: "center" }}>
                <button
                  onClick={completeOnboarding}
                  style={{ padding: "10px 24px", borderRadius: "6px", border: "1px solid var(--border-default)", background: "transparent", color: "var(--text-tertiary)", cursor: "pointer" }}
                >
                  {t('console.dashboard.skip')}
                </button>
                <button
                  onClick={() => setOnboardingStep(2)}
                  style={{ padding: "10px 24px", borderRadius: "6px", border: "none", background: "var(--accent-brand)", color: "#fff", cursor: "pointer" }}
                >
                  {t('console.dashboard.get_started')}
                </button>
              </div>
            </>
          )}
          {onboardingStep === 2 && (
            <>
              <h3 style={{ color: "var(--text-primary)", margin: "0 0 16px 0" }}>{t('console.dashboard.select_agent')}</h3>
              <p style={{ color: "var(--text-tertiary)", fontSize: "14px", margin: "0 0 16px 0" }}>
                {t('console.dashboard.select_template_desc')}
              </p>
              <TemplateSelector
                onSelect={() => setOnboardingStep(3)}
              />
              <div style={{ display: "flex", gap: "12px", justifyContent: "center", marginTop: "16px" }}>
                <button
                  onClick={completeOnboarding}
                  style={{ padding: "8px 20px", borderRadius: "6px", border: "1px solid var(--border-default)", background: "transparent", color: "var(--text-tertiary)", cursor: "pointer" }}
                >
                  {t('console.dashboard.skip')}
                </button>
                <button
                  onClick={() => setOnboardingStep(3)}
                  style={{ padding: "8px 20px", borderRadius: "6px", border: "none", background: "var(--accent-brand)", color: "#fff", cursor: "pointer" }}
                >
                  {t('console.dashboard.next_step')}
                </button>
              </div>
            </>
          )}
          {onboardingStep === 3 && (
            <>
              <div style={{ fontSize: "48px", textAlign: "center", marginBottom: "16px" }}>🚀</div>
              <h2 style={{ color: "var(--text-primary)", textAlign: "center", margin: "0 0 8px 0" }}>{t('console.dashboard.try_it')}</h2>
              <p style={{ color: "var(--text-tertiary)", textAlign: "center", fontSize: "14px", margin: "0 0 24px 0" }}>
                {t('console.dashboard.try_it_desc')}
              </p>
              <div style={{ display: "flex", gap: "12px", justifyContent: "center" }}>
                <button
                  onClick={completeOnboarding}
                  style={{ padding: "8px 20px", borderRadius: "6px", border: "1px solid var(--border-default)", background: "transparent", color: "var(--text-tertiary)", cursor: "pointer" }}
                >
                  {t('console.dashboard.skip')}
                </button>
                <button
                  onClick={() => setOnboardingStep(4)}
                  style={{ padding: "10px 24px", borderRadius: "6px", border: "none", background: "var(--accent-brand)", color: "#fff", cursor: "pointer" }}
                >
                  {t('console.dashboard.next_step')}
                </button>
              </div>
            </>
          )}
          {onboardingStep === 4 && (
            <>
              <div style={{ fontSize: "48px", textAlign: "center", marginBottom: "16px" }}>🧭</div>
              <h2 style={{ color: "var(--text-primary)", textAlign: "center", margin: "0 0 8px 0" }}>{t('console.dashboard.explore_features')}</h2>
              <p style={{ color: "var(--text-tertiary)", textAlign: "center", fontSize: "14px", margin: "0 0 20px 0" }}>
                {t('console.dashboard.explore_desc')}
              </p>
              <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "10px", marginBottom: "20px" }}>
                {[
                  { label: t('console.dashboard.memory_panel'), tab: "memory", icon: "🧠" },
                  { label: t('console.dashboard.cost_panel'), tab: "cost_tracking", icon: "💰" },
                  { label: t('console.dashboard.journey_panel'), tab: "journey", icon: "🗺️" },
                  { label: t('console.dashboard.studio_panel'), tab: "", icon: "🔧" },
                ].map(item => (
                  <button
                    key={item.label}
                    onClick={() => {
                      if (item.tab && onNavigate) {
                        completeOnboarding();
                        onNavigate(item.tab);
                      } else if (!item.tab) {
                        completeOnboarding();
                      }
                    }}
                    style={{
                      display: "flex", alignItems: "center", gap: "8px",
                      padding: "12px", borderRadius: "8px",
                      border: "1px solid var(--border-default)",
                      background: "var(--bg-page)", color: "var(--text-primary)",
                      cursor: "pointer", fontSize: "13px",
                      transition: "background 0.2s",
                    }}
                  >
                    <span style={{ fontSize: "18px" }}>{item.icon}</span>
                    <span>{item.label}</span>
                  </button>
                ))}
              </div>
              <div style={{ display: "flex", gap: "12px", justifyContent: "center" }}>
                <button
                  onClick={completeOnboarding}
                  style={{ padding: "8px 20px", borderRadius: "6px", border: "1px solid var(--border-default)", background: "transparent", color: "var(--text-tertiary)", cursor: "pointer" }}
                >
                  {t('console.dashboard.skip')}
                </button>
                <button
                  onClick={() => setOnboardingStep(5)}
                  style={{ padding: "10px 24px", borderRadius: "6px", border: "none", background: "var(--accent-brand)", color: "#fff", cursor: "pointer" }}
                >
                  {t('console.dashboard.finish')}
                </button>
              </div>
            </>
          )}
          {onboardingStep === 5 && (
            <>
              <div style={{ fontSize: "48px", textAlign: "center", marginBottom: "16px" }}>🎉</div>
              <h2 style={{ color: "var(--text-primary)", textAlign: "center", margin: "0 0 8px 0" }}>{t('console.dashboard.complete_title')}</h2>
              <p style={{ color: "var(--text-tertiary)", textAlign: "center", fontSize: "14px", margin: "0 0 12px 0" }}>
                {t('console.dashboard.complete_desc')}
              </p>
              <p style={{ color: "var(--text-tertiary)", textAlign: "center", fontSize: "12px", fontStyle: "italic", margin: "0 0 24px 0" }}>
                {t('console.dashboard.reopen_hint')}
              </p>
              <div style={{ display: "flex", gap: "12px", justifyContent: "center" }}>
                <button
                  onClick={completeOnboarding}
                  style={{ padding: "10px 24px", borderRadius: "6px", border: "none", background: "var(--accent-brand)", color: "#fff", cursor: "pointer" }}
                >
                  {t('console.dashboard.start_exploring')}
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
      <div className="skeleton-dashboard">
        {[1,2,3,4,5,6].map(i => <div key={i} className="skeleton" />)}
        <div className="skeleton" style={{ height: '60px' }} />
        <div className="skeleton" style={{ height: '200px' }} />
        <div className="skeleton" style={{ height: '200px' }} />
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