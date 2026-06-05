import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { TemplateSelector } from "../studio/TemplateSelector";

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
  const [showOnboarding, setShowOnboarding] = useState(false);
  const [onboardingStep, setOnboardingStep] = useState(1);

  useEffect(() => {
    invoke<{ dashboard: DashboardData; system_info: any }>("get_system_status").then((res) => {
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
      {showOnboarding && renderOnboarding()}
    </div>
  );
}