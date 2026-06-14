import { useState } from "react";
import { api } from "../api";

interface TeamTemplate {
  id: string;
  name: string;
  description: string;
  members: string[];
  mode: string;
  consensus: string;
}

const TEAM_TEMPLATES: TeamTemplate[] = [
  {
    id: "stock-research",
    name: "股票研究团队",
    description: "金融数据分析、市场搜索、图表绘制与智能问答",
    members: ["data-agent", "search-agent", "fin-plot", "chat-agent"],
    mode: "Chain",
    consensus: "AutoSynthesis",
  },
  {
    id: "software-dev",
    name: "软件开发团队",
    description: "需求分析、架构设计、编码实现与质量保障",
    members: ["pm-agent", "architect-agent", "coder-agent", "qa-agent"],
    mode: "ManagerWorker",
    consensus: "AutoSynthesis",
  },
  {
    id: "content-prod",
    name: "内容生产团队",
    description: "研究调研、内容撰写、编辑审核与发布分发",
    members: ["research-agent", "writer-agent", "editor-agent", "publisher-agent"],
    mode: "Chain",
    consensus: "CeoDecides",
  },
  {
    id: "market-research",
    name: "市场调研团队",
    description: "多渠道信息采集、数据分析与报告生成",
    members: ["search-agent", "analyst-agent", "report-agent"],
    mode: "Chain",
    consensus: "AutoSynthesis",
  },
  {
    id: "risk-control",
    name: "风控团队",
    description: "数据采集、规则引擎、风险评估与预警通知",
    members: ["data-agent", "rule-agent", "analyst-agent", "alert-agent"],
    mode: "Voting",
    consensus: "MungerVeto",
  },
  {
    id: "customer-service",
    name: "客服团队",
    description: "意图分类、会话处理、知识检索与升级流转",
    members: ["classifier-agent", "handler-agent", "knowledge-agent", "escalate-agent"],
    mode: "Routing",
    consensus: "AutoSynthesis",
  },
  {
    id: "monitoring",
    name: "监控团队",
    description: "定时巡检、健康检查、告警通知与报告生成",
    members: ["timer-agent", "check-agent", "alert-agent", "report-agent"],
    mode: "Broadcast",
    consensus: "AutoSynthesis",
  },
];

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

export function TeamBuilder() {
  const [selected, setSelected] = useState<string | null>(null);
  const [preview, setPreview] = useState<TeamTemplate | null>(null);
  const [creating, setCreating] = useState(false);

  const handleCreate = async (template: TeamTemplate) => {
    setCreating(true);
    try {
      await api.createTeam(template.name, template.description, "default-user");
      alert(`团队 "${template.name}" 创建成功`);
    } catch (e: any) {
      alert(`创建失败: ${e}`);
    } finally {
      setCreating(false);
    }
  };

  return (
    <div className="team-template-selector">
      <h2>团队构建</h2>
      <p style={{ color: "var(--text-secondary)", fontSize: "13px", marginBottom: "16px" }}>
        选择预置团队模板，快速创建协作团队
      </p>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(260px, 1fr))", gap: "12px" }}>
        {TEAM_TEMPLATES.map((t) => {
          const isSelected = selected === t.id;
          return (
            <div
              key={t.id}
              className={`team-template-card ${isSelected ? "selected" : ""}`}
              onClick={() => {
                setSelected(t.id);
                setPreview(t);
              }}
              style={{
                background: isSelected ? "var(--bg-tertiary)" : "var(--bg-secondary)",
                border: isSelected ? "2px solid var(--accent)" : "1px solid var(--border)",
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
                onClick={(e) => { e.stopPropagation(); handleCreate(t); }}
                disabled={creating}
                style={{
                  width: "100%", marginTop: "10px", padding: "6px 12px", borderRadius: "6px",
                  background: "var(--accent)", color: "#fff", border: "none",
                  cursor: creating ? "not-allowed" : "pointer", fontSize: "13px", fontWeight: 500,
                  opacity: creating ? 0.7 : 1,
                }}
              >
                {creating ? "创建中..." : "创建团队"}
              </button>
            </div>
          );
        })}
      </div>
      {preview && (
        <div style={{
          marginTop: "20px", padding: "16px", borderRadius: "8px",
          background: "var(--bg-secondary)", border: "1px solid var(--border)",
        }}>
          <h3 style={{ margin: "0 0 8px", color: "var(--text-primary)", fontSize: "15px" }}>
            {preview.name} — 预览
          </h3>
          <div style={{ fontSize: "13px", color: "var(--text-secondary)", marginBottom: "8px" }}>
            {preview.description}
          </div>
          <div style={{ fontSize: "13px", color: "var(--text-primary)", marginBottom: "4px" }}>
            协作模式: {MODE_LABELS[preview.mode] || preview.mode}
          </div>
          <div style={{ fontSize: "13px", color: "var(--text-primary)", marginBottom: "8px" }}>
            决策方式: {CONSENSUS_LABELS[preview.consensus] || preview.consensus}
          </div>
          <div style={{ display: "flex", flexWrap: "wrap", gap: "4px", marginBottom: "10px" }}>
            {preview.members.map((m, i) => (
              <span key={i} style={{
                fontSize: "12px", padding: "3px 8px", borderRadius: "4px",
                background: "var(--bg-tertiary)", color: "var(--text-secondary)",
                border: "1px solid var(--border)",
              }}>
                {m}
              </span>
            ))}
          </div>
          <button
            onClick={() => handleCreate(preview)}
            disabled={creating}
            style={{
              padding: "8px 20px", borderRadius: "6px",
              background: "var(--accent)", color: "#fff", border: "none",
              cursor: creating ? "not-allowed" : "pointer", fontSize: "14px", fontWeight: 500,
              opacity: creating ? 0.7 : 1,
            }}
          >
            {creating ? "创建中..." : "确认创建团队"}
          </button>
        </div>
      )}
    </div>
  );
}