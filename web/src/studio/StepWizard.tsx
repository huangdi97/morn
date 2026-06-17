import { useState, useMemo } from "react";
import { api } from "../api";
import { AgentDef } from "./types";

interface StepWizardProps {
  onClose: () => void;
}

const STEP_ICONS = ["🧠", "🛠", "⚙", "👤", "📡"];
const STEP_LABELS = ["选择记忆", "选择工具", "选择 LLM 模型", "选择人格", "选择渠道"];

const MEMORY_OPTIONS = [
  { id: "working_memory", label: "工作记忆", desc: "短期上下文存储" },
  { id: "episodic_memory", label: "情景记忆", desc: "历史交互记录" },
  { id: "flash_memory", label: "闪存", desc: "高频快速存取" },
  { id: "semantic_memory", label: "语义记忆", desc: "长期知识沉淀" },
];

const TOOL_OPTIONS = [
  { id: "web_search", label: "web_search", desc: "网络搜索" },
  { id: "read_file", label: "read_file", desc: "读取文件" },
  { id: "write_file", label: "write_file", desc: "写入文件" },
  { id: "code_exec", label: "code_exec", desc: "执行代码" },
  { id: "file_ops", label: "file_ops", desc: "文件操作" },
  { id: "get_time", label: "get_time", desc: "获取时间" },
  { id: "calc", label: "calc", desc: "计算器" },
  { id: "chart", label: "chart", desc: "图表生成" },
];

const MODEL_OPTIONS = [
  { id: "deepseek-chat", label: "DeepSeek Chat", provider: "DeepSeek" },
  { id: "deepseek-reasoner", label: "DeepSeek Reasoner", provider: "DeepSeek" },
  { id: "gpt-4o", label: "GPT-4o", provider: "OpenAI" },
  { id: "claude-3", label: "Claude 3", provider: "Anthropic" },
];

const PERSONA_OPTIONS = [
  { id: "assistant", label: "通用助手", desc: "混合工具集的通用助手，适合日常问答" },
  { id: "analyst", label: "数据分析师", desc: "获取数据、计算指标、生成图表" },
  { id: "researcher", label: "研究助手", desc: "多源信息检索、知识整合与摘要生成" },
  { id: "writer", label: "写作助手", desc: "翻译、语法检查、格式润色与风格优化" },
  { id: "coder", label: "编码助手", desc: "代码审查、调试、格式化和测试" },
  { id: "translator", label: "翻译 Agent", desc: "多语言翻译、校对和专业术语管理" },
  { id: "reviewer", label: "审查员", desc: "代码和文档审查" },
  { id: "cs_agent", label: "客服 Agent", desc: "客户服务与工单处理" },
];

const CHANNEL_OPTIONS = [
  { id: "telegram", label: "Telegram", desc: "通过 Telegram Bot 交互" },
  { id: "wechat", label: "微信", desc: "通过微信公众号/企业微信交互" },
  { id: "cli", label: "CLI", desc: "命令行终端交互" },
  { id: "web", label: "Web", desc: "Web 界面交互" },
];

const STEP_TOTAL = 5;

export function StepWizard({ onClose }: StepWizardProps) {
  const [step, setStep] = useState(0);
  const [building, setBuilding] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [agentId, setAgentId] = useState<string | null>(null);

  const [selectedMemory, setSelectedMemory] = useState<string | null>(null);
  const [selectedTools, setSelectedTools] = useState<string[]>([]);
  const [selectedModel, setSelectedModel] = useState("deepseek-chat");
  const [selectedPersona, setSelectedPersona] = useState("assistant");
  const [selectedChannels, setSelectedChannels] = useState<string[]>([]);

  const [agentName, setAgentName] = useState("");

  const selections = useMemo(() => ({
    memory: MEMORY_OPTIONS.find((m) => m.id === selectedMemory),
    tools: TOOL_OPTIONS.filter((t) => selectedTools.includes(t.id)),
    model: MODEL_OPTIONS.find((m) => m.id === selectedModel),
    persona: PERSONA_OPTIONS.find((p) => p.id === selectedPersona),
    channels: CHANNEL_OPTIONS.filter((c) => selectedChannels.includes(c.id)),
  }), [selectedMemory, selectedTools, selectedModel, selectedPersona, selectedChannels]);

  const canProceed = useMemo(() => {
    switch (step) {
      case 0: return selectedMemory !== null;
      case 1: return selectedTools.length > 0;
      case 2: return selectedModel !== "";
      case 3: return selectedPersona !== "";
      case 4: return agentName.trim().length > 0;
      default: return false;
    }
  }, [step, selectedMemory, selectedTools.length, selectedModel, selectedPersona, agentName]);

  const toggleTool = (id: string) => {
    setSelectedTools((prev) =>
      prev.includes(id) ? prev.filter((t) => t !== id) : [...prev, id],
    );
  };

  const toggleChannel = (id: string) => {
    setSelectedChannels((prev) =>
      prev.includes(id) ? prev.filter((c) => c !== id) : [...prev, id],
    );
  };

  const handleNext = () => {
    if (step < STEP_TOTAL - 1) {
      setStep((s) => s + 1);
    }
  };

  const handlePrev = () => {
    if (step > 0) {
      setStep((s) => s - 1);
    }
  };

  const handleFinish = async () => {
    try {
      setBuilding(true);
      setError(null);
      const def: AgentDef = {
        name: agentName.trim(),
        persona: selectedPersona,
        model: selectedModel,
        tools: selectedTools,
        knowledge: selectedMemory ? [selectedMemory] : [],
        skills: [],
      };
      const result = await api.assembleAgent(def) as { agent_id: string };
      setAgentId(result.agent_id);
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setBuilding(false);
    }
  };

  const stepStatus = (i: number): "completed" | "active" | "inactive" => {
    if (i < step) return "completed";
    if (i === step) return "active";
    return "inactive";
  };

  const renderSidebar = () => (
    <div style={{
      width: "200px", flexShrink: 0, borderRight: "1px solid var(--border)",
      padding: "24px 16px", display: "flex", flexDirection: "column", gap: "4px",
    }}>
      {Array.from({ length: STEP_TOTAL }, (_, i) => {
        const status = stepStatus(i);
        const isActive = status === "active";
        const isCompleted = status === "completed";
        return (
          <div
            key={i}
            onClick={() => { if (isCompleted || isActive) setStep(i); }}
            style={{
              display: "flex", alignItems: "center", gap: "10px", padding: "10px 12px",
              borderRadius: "8px", cursor: isCompleted || isActive ? "pointer" : "default",
              background: isActive ? "var(--bg-tertiary)" : "transparent",
              opacity: status === "inactive" ? 0.4 : 1,
              transition: "all 0.15s ease",
            }}
          >
            <div style={{
              width: "28px", height: "28px", borderRadius: "50%",
              display: "flex", alignItems: "center", justifyContent: "center",
              fontSize: "13px", fontWeight: 600, flexShrink: 0,
              background: isCompleted ? "var(--success)" : isActive ? "var(--accent)" : "var(--bg-tertiary)",
              color: "#fff",
              border: isActive ? "2px solid var(--accent)" : "2px solid transparent",
            }}>
              {isCompleted ? "✓" : i + 1}
            </div>
            <div style={{ fontSize: "13px", color: "var(--text-primary)", fontWeight: isActive ? 600 : 400 }}>
              {STEP_ICONS[i]} {STEP_LABELS[i]}
            </div>
          </div>
        );
      })}
    </div>
  );

  const renderStepContent = () => {
    switch (step) {
      case 0:
        return (
          <div>
            <h3 style={{ margin: "0 0 4px 0", fontSize: "16px", color: "var(--text-primary)" }}>🧠 选择记忆</h3>
            <p style={{ margin: "0 0 16px 0", fontSize: "13px", color: "var(--text-secondary)" }}>为 Agent 选择一种记忆方式</p>
            <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
              {MEMORY_OPTIONS.map((m) => (
                <div
                  key={m.id}
                  onClick={() => setSelectedMemory(m.id)}
                  style={{
                    padding: "12px 16px", borderRadius: "8px", cursor: "pointer",
                    background: selectedMemory === m.id ? "var(--bg-tertiary)" : "var(--bg-secondary)",
                    border: selectedMemory === m.id ? "2px solid var(--accent)" : "1px solid var(--border)",
                    transition: "all 0.15s ease",
                  }}
                >
                  <div style={{ fontWeight: 500, fontSize: "14px", color: "var(--text-primary)", marginBottom: "4px" }}>{m.label}</div>
                  <div style={{ fontSize: "12px", color: "var(--text-secondary)" }}>{m.desc}</div>
                </div>
              ))}
            </div>
          </div>
        );
      case 1:
        return (
          <div>
            <h3 style={{ margin: "0 0 4px 0", fontSize: "16px", color: "var(--text-primary)" }}>🛠 选择工具</h3>
            <p style={{ margin: "0 0 16px 0", fontSize: "13px", color: "var(--text-secondary)" }}>为 Agent 选择可用工具（可多选）</p>
            <div style={{ display: "flex", flexWrap: "wrap", gap: "8px" }}>
              {TOOL_OPTIONS.map((t) => {
                const selected = selectedTools.includes(t.id);
                return (
                  <div
                    key={t.id}
                    onClick={() => toggleTool(t.id)}
                    style={{
                      padding: "8px 14px", borderRadius: "20px", cursor: "pointer",
                      background: selected ? "var(--accent)" : "var(--bg-tertiary)",
                      color: selected ? "#fff" : "var(--text-primary)",
                      border: selected ? "1px solid var(--accent)" : "1px solid var(--border)",
                      fontSize: "13px", fontWeight: selected ? 500 : 400,
                      transition: "all 0.15s ease",
                      userSelect: "none",
                    }}
                  >
                    {t.label}
                    <span style={{ marginLeft: "6px", fontSize: "11px", opacity: 0.7 }}>{t.desc}</span>
                  </div>
                );
              })}
            </div>
          </div>
        );
      case 2:
        return (
          <div>
            <h3 style={{ margin: "0 0 4px 0", fontSize: "16px", color: "var(--text-primary)" }}>⚙ 选择 LLM 模型</h3>
            <p style={{ margin: "0 0 16px 0", fontSize: "13px", color: "var(--text-secondary)" }}>选择 Agent 使用的语言模型</p>
            <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
              {MODEL_OPTIONS.map((m) => (
                <div
                  key={m.id}
                  onClick={() => setSelectedModel(m.id)}
                  style={{
                    padding: "12px 16px", borderRadius: "8px", cursor: "pointer",
                    background: selectedModel === m.id ? "var(--bg-tertiary)" : "var(--bg-secondary)",
                    border: selectedModel === m.id ? "2px solid var(--accent)" : "1px solid var(--border)",
                    transition: "all 0.15s ease",
                    display: "flex", alignItems: "center", gap: "12px",
                  }}
                >
                  <div style={{
                    width: "12px", height: "12px", borderRadius: "50%", flexShrink: 0,
                    background: selectedModel === m.id ? "var(--accent)" : "var(--border)",
                    border: selectedModel === m.id ? "3px solid var(--accent)" : "2px solid var(--border)",
                  }} />
                  <div>
                    <div style={{ fontWeight: 500, fontSize: "14px", color: "var(--text-primary)" }}>{m.label}</div>
                    <div style={{ fontSize: "12px", color: "var(--text-secondary)" }}>{m.provider}</div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        );
      case 3:
        return (
          <div>
            <h3 style={{ margin: "0 0 4px 0", fontSize: "16px", color: "var(--text-primary)" }}>👤 选择人格</h3>
            <p style={{ margin: "0 0 16px 0", fontSize: "13px", color: "var(--text-secondary)" }}>为 Agent 选择一种预设人格</p>
            <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "8px" }}>
              {PERSONA_OPTIONS.map((p) => (
                <div
                  key={p.id}
                  onClick={() => setSelectedPersona(p.id)}
                  style={{
                    padding: "12px 16px", borderRadius: "8px", cursor: "pointer",
                    background: selectedPersona === p.id ? "var(--bg-tertiary)" : "var(--bg-secondary)",
                    border: selectedPersona === p.id ? "2px solid var(--accent)" : "1px solid var(--border)",
                    transition: "all 0.15s ease",
                  }}
                >
                  <div style={{ fontWeight: 500, fontSize: "14px", color: "var(--text-primary)", marginBottom: "4px" }}>{p.label}</div>
                  <div style={{ fontSize: "12px", color: "var(--text-secondary)", lineHeight: "1.4" }}>{p.desc}</div>
                </div>
              ))}
            </div>
          </div>
        );
      case 4:
        return (
          <div>
            <h3 style={{ margin: "0 0 4px 0", fontSize: "16px", color: "var(--text-primary)" }}>📡 完成配置</h3>
            <p style={{ margin: "0 0 16px 0", fontSize: "13px", color: "var(--text-secondary)" }}>命名你的 Agent 并选择部署渠道</p>
            <div style={{ marginBottom: "20px" }}>
              <label style={{ fontSize: "13px", color: "var(--text-secondary)", display: "block", marginBottom: "6px" }}>Agent 名称</label>
              <input
                type="text"
                value={agentName}
                onChange={(e) => setAgentName(e.target.value)}
                placeholder="例如：我的智能助手"
                style={{
                  width: "100%", padding: "10px 12px", borderRadius: "6px",
                  border: "1px solid var(--border)", background: "var(--bg-secondary)",
                  color: "var(--text-primary)", fontSize: "14px", outline: "none",
                  fontFamily: "inherit",
                }}
                onFocus={(e) => e.target.style.borderColor = "var(--accent)"}
                onBlur={(e) => e.target.style.borderColor = ""}
              />
            </div>
            <div>
              <label style={{ fontSize: "13px", color: "var(--text-secondary)", display: "block", marginBottom: "8px" }}>部署渠道（可多选）</label>
              <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
                {CHANNEL_OPTIONS.map((c) => (
                  <label
                    key={c.id}
                    style={{
                      display: "flex", alignItems: "center", gap: "10px", padding: "10px 12px",
                      borderRadius: "6px", cursor: "pointer",
                      background: selectedChannels.includes(c.id) ? "var(--bg-tertiary)" : "var(--bg-secondary)",
                      border: selectedChannels.includes(c.id) ? "1px solid var(--accent)" : "1px solid var(--border)",
                      transition: "all 0.15s ease",
                    }}
                  >
                    <input
                      type="checkbox"
                      checked={selectedChannels.includes(c.id)}
                      onChange={() => toggleChannel(c.id)}
                      style={{ accentColor: "var(--accent)" }}
                    />
                    <div>
                      <div style={{ fontSize: "14px", color: "var(--text-primary)", fontWeight: 500 }}>{c.label}</div>
                      <div style={{ fontSize: "12px", color: "var(--text-secondary)" }}>{c.desc}</div>
                    </div>
                  </label>
                ))}
              </div>
            </div>
          </div>
        );
      default:
        return null;
    }
  };

  const renderOverview = () => {
    const items: { label: string; value: string }[] = [];
    if (selections.memory) items.push({ label: "记忆", value: selections.memory.label });
    if (selectedTools.length > 0) items.push({ label: "工具", value: selectedTools.join(", ") });
    if (selections.model) items.push({ label: "模型", value: selections.model.label });
    if (selections.persona) items.push({ label: "人格", value: selections.persona.label });
    if (selectedChannels.length > 0) items.push({ label: "渠道", value: selectedChannels.join(", ") });
    if (agentName.trim()) items.unshift({ label: "名称", value: agentName.trim() });

    if (items.length === 0) return null;
    return (
      <div style={{
        padding: "12px 16px", borderRadius: "8px",
        background: "var(--bg-tertiary)", border: "1px solid var(--border)",
        marginTop: "20px",
      }}>
        <div style={{ fontSize: "12px", color: "var(--text-secondary)", marginBottom: "8px", fontWeight: 500, textTransform: "uppercase", letterSpacing: "0.5px" }}>当前配置概览</div>
        <div style={{ display: "flex", flexWrap: "wrap", gap: "6px" }}>
          {items.map((item, i) => (
            <span key={i} style={{
              fontSize: "12px", padding: "3px 8px", borderRadius: "4px",
              background: "var(--bg-secondary)", color: "var(--text-primary)",
              border: "1px solid var(--border)",
            }}>
              <span style={{ color: "var(--text-secondary)" }}>{item.label}: </span>
              {item.value}
            </span>
          ))}
        </div>
      </div>
    );
  };

  if (agentId) {
    return (
      <div style={{
        position: "fixed", inset: 0, zIndex: 1000,
        background: "rgba(0,0,0,0.6)", display: "flex", alignItems: "center", justifyContent: "center",
      }}>
        <div style={{
          background: "var(--bg-primary)", borderRadius: "12px",
          border: "1px solid var(--border)", width: "480px", maxWidth: "90vw",
          padding: "32px", textAlign: "center",
        }}>
          <div style={{ fontSize: "48px", marginBottom: "12px" }}>✅</div>
          <h2 style={{ margin: "0 0 8px 0", color: "var(--text-primary)" }}>Agent 创建成功!</h2>
          <p style={{ fontSize: "14px", color: "var(--text-secondary)", margin: "0 0 16px 0" }}>
            {agentName} 已创建
          </p>
          {agentId && (
            <p style={{ fontSize: "12px", color: "var(--text-secondary)", marginBottom: "20px", wordBreak: "break-all" }}>
              Agent ID: {agentId}
            </p>
          )}
          <button
            onClick={onClose}
            style={{
              padding: "8px 24px", borderRadius: "6px",
              background: "var(--accent)", color: "#fff", border: "none",
              cursor: "pointer", fontSize: "14px", fontWeight: 500,
            }}
          >
            关闭
          </button>
        </div>
      </div>
    );
  }

  return (
    <div style={{
      position: "fixed", inset: 0, zIndex: 1000,
      background: "rgba(0,0,0,0.6)", display: "flex", alignItems: "center", justifyContent: "center",
    }}>
      <div style={{
        background: "var(--bg-primary)", borderRadius: "12px",
        border: "1px solid var(--border)", width: "720px", maxWidth: "90vw",
        maxHeight: "85vh", display: "flex", flexDirection: "column",
        boxShadow: "0 16px 48px rgba(0,0,0,0.4)",
      }}>
        <div style={{
          display: "flex", alignItems: "center", justifyContent: "space-between",
          padding: "16px 20px", borderBottom: "1px solid var(--border)",
        }}>
          <h2 style={{ margin: 0, fontSize: "18px", color: "var(--text-primary)" }}>
            引导式构建 <span style={{ fontSize: "13px", color: "var(--text-secondary)", fontWeight: 400 }}>Step {step + 1}/{STEP_TOTAL}</span>
          </h2>
          <button
            onClick={onClose}
            style={{
              background: "none", border: "none", color: "var(--text-secondary)",
              cursor: "pointer", fontSize: "20px", padding: "4px 8px",
              borderRadius: "4px", lineHeight: 1,
            }}
          >
            ✕
          </button>
        </div>
        <div style={{ display: "flex", flex: 1, minHeight: 0 }}>
          {renderSidebar()}
          <div style={{ flex: 1, padding: "24px", overflowY: "auto" }}>
            {renderStepContent()}
            {renderOverview()}
            {error && (
              <div style={{
                marginTop: "12px", padding: "8px 12px", borderRadius: "6px",
                background: "rgba(248,81,73,0.1)", border: "1px solid var(--danger)",
                color: "var(--danger)", fontSize: "13px",
              }}>
                {error}
              </div>
            )}
          </div>
        </div>
        <div style={{
          display: "flex", justifyContent: "space-between", alignItems: "center",
          padding: "16px 20px", borderTop: "1px solid var(--border)",
        }}>
          <div style={{ fontSize: "12px", color: "var(--text-secondary)" }}>
            {step === 0 ? "开始配置你的 Agent" : ""}
          </div>
          <div style={{ display: "flex", gap: "8px" }}>
            <button
              onClick={handlePrev}
              disabled={step === 0}
              style={{
                padding: "8px 20px", borderRadius: "6px", cursor: step === 0 ? "not-allowed" : "pointer",
                background: "var(--bg-secondary)", color: "var(--text-primary)",
                border: "1px solid var(--border)", fontSize: "13px", fontWeight: 500,
                opacity: step === 0 ? 0.5 : 1,
              }}
            >
              上一步
            </button>
            {step < STEP_TOTAL - 1 ? (
              <button
                onClick={handleNext}
                disabled={!canProceed}
                style={{
                  padding: "8px 20px", borderRadius: "6px", cursor: canProceed ? "pointer" : "not-allowed",
                  background: "var(--accent)", color: "#fff",
                  border: "none", fontSize: "13px", fontWeight: 500,
                  opacity: canProceed ? 1 : 0.5,
                }}
              >
                下一步
              </button>
            ) : (
              <button
                onClick={handleFinish}
                disabled={building || !canProceed}
                style={{
                  padding: "8px 20px", borderRadius: "6px", cursor: building || !canProceed ? "not-allowed" : "pointer",
                  background: "var(--success)", color: "#fff",
                  border: "none", fontSize: "13px", fontWeight: 500,
                  opacity: building || !canProceed ? 0.5 : 1,
                }}
              >
                {building ? "创建中..." : "完成"}
              </button>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}
