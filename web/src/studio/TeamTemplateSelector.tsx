import { useState, useEffect } from "react";
import { api } from "../api";

interface TeamTemplate {
  id: string;
  name: string;
  description: string;
  members: string[];
  mode: string;
  consensus: string;
}

const FALLBACK_TEMPLATES: TeamTemplate[] = [
  { id: "preset-code-review", name: "Code Review Team", description: "Automated code review with reviewer and author agents", members: ["agent-reviewer", "agent-author"], mode: "Voting", consensus: "MungerVeto" },
  { id: "preset-stock-research", name: "Stock Research Team", description: "Financial analysis, market research, and charting", members: ["data-agent", "search-agent", "fin-plot", "chat-agent"], mode: "Chain", consensus: "AutoSynthesis" },
  { id: "preset-research", name: "Research Team", description: "Deep research, analysis, and report writing", members: ["agent-researcher", "agent-analyst", "agent-writer"], mode: "ManagerWorker", consensus: "AutoSynthesis" },
  { id: "preset-support", name: "Support Team", description: "Customer support ticket handling and quality assurance", members: ["agent-support", "agent-quality"], mode: "Routing", consensus: "CeoDecides" },
  { id: "preset-content", name: "Content Team", description: "Content creation, design, and proofreading pipeline", members: ["agent-editor", "agent-designer", "agent-proofreader"], mode: "Chain", consensus: "CeoDecides" },
  { id: "preset-devops", name: "DevOps Team", description: "Deployment, monitoring, and incident alerting", members: ["agent-deployer", "agent-monitor", "agent-alert"], mode: "Blackboard", consensus: "AutoSynthesis" },
  { id: "preset-data", name: "Data Team", description: "Data collection, processing, and reporting pipeline", members: ["agent-collector", "agent-processor", "agent-reporter"], mode: "Chain", consensus: "AutoSynthesis" },
  { id: "preset-management", name: "Management Team", description: "Decision-making, execution, and evaluation", members: ["agent-decision", "agent-execution", "agent-evaluation"], mode: "ManagerWorker", consensus: "CeoDecides" },
  { id: "preset-monitoring", name: "Monitoring Team", description: "System health monitoring, inspection, and alerting", members: ["timer-agent", "check-agent", "alert-agent", "report-agent"], mode: "Broadcast", consensus: "AutoSynthesis" },
  { id: "preset-risk-control", name: "Risk Control Team", description: "Risk assessment, compliance, and security auditing", members: ["data-agent", "rule-agent", "analyst-agent", "alert-agent"], mode: "Voting", consensus: "MungerVeto" },
];

interface TeamTemplateSelectorProps {
  onSelect?: (template: TeamTemplate) => void;
}

const MODE_LABELS: Record<string, string> = {
  Chain: "链式",
  Voting: "投票",
  Broadcast: "广播",
  ManagerWorker: "管理-执行",
  Routing: "路由",
  Blackboard: "黑板",
};

const CONSENSUS_LABELS: Record<string, string> = {
  CeoDecides: "CEO决策",
  MungerVeto: "一票否决",
  AutoSynthesis: "自动合成",
  Vote: "多数投票",
};

export function TeamTemplateSelector({ onSelect }: TeamTemplateSelectorProps) {
  const [selected, setSelected] = useState<string | null>(null);
  const [templates, setTemplates] = useState<TeamTemplate[]>(FALLBACK_TEMPLATES);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.listTeamTemplates().then((list: TeamTemplate[]) => {
      if (list.length > 0) {
        setTemplates(list);
      }
    }).catch(() => {}).finally(() => setLoading(false));
  }, []);

  return (
    <div className="team-template-selector">
      <h2>团队模板</h2>
      {loading && <p style={{ color: "var(--text-secondary)" }}>加载中...</p>}
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(260px, 1fr))", gap: "12px" }}>
        {templates.map((t) => (
          <div
            key={t.id}
            className={`team-template-card ${selected === t.id ? "selected" : ""}`}
            onClick={() => {
              setSelected(t.id);
              onSelect?.(t);
            }}
            style={{
              background: selected === t.id ? "var(--bg-tertiary)" : "var(--bg-secondary)",
              border: selected === t.id ? "2px solid var(--accent)" : "1px solid var(--border)",
              borderRadius: "8px",
              padding: "16px",
              cursor: "pointer",
              transition: "all 0.15s ease",
            }}
          >
            <div style={{ fontWeight: 600, color: "var(--text-primary)", marginBottom: "4px", fontSize: "15px" }}>{t.name}</div>
            <div style={{ fontSize: "13px", color: "var(--text-secondary)", marginBottom: "10px", lineHeight: "1.4" }}>{t.description}</div>
            <div style={{ display: "flex", flexWrap: "wrap", gap: "4px", marginBottom: "8px" }}>
              {t.members.map((m, i) => (
                <span key={i} style={{
                  fontSize: "11px", padding: "2px 6px", borderRadius: "4px",
                  background: "var(--bg-tertiary)", color: "var(--text-secondary)",
                  border: "1px solid var(--border)",
                }}>
                  {m}
                </span>
              ))}
            </div>
            <div style={{ display: "flex", gap: "6px", flexWrap: "wrap" }}>
              <span style={{
                fontSize: "11px", padding: "2px 6px", borderRadius: "4px",
                background: "rgba(99,102,241,0.15)", color: "var(--accent)",
              }}>
                {MODE_LABELS[t.mode] || t.mode}
              </span>
              <span style={{
                fontSize: "11px", padding: "2px 6px", borderRadius: "4px",
                background: "rgba(34,197,94,0.15)", color: "rgb(34,197,94)",
              }}>
                {CONSENSUS_LABELS[t.consensus] || t.consensus}
              </span>
            </div>
            <button
              onClick={(e) => { e.stopPropagation(); onSelect?.(t); }}
              style={{
                width: "100%", marginTop: "10px", padding: "6px 12px", borderRadius: "6px",
                background: "var(--accent)", color: "#fff", border: "none",
                cursor: "pointer", fontSize: "13px", fontWeight: 500,
              }}
            >
              使用此团队
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
