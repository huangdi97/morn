import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { NodeCanvas } from "./NodeCanvas";

interface AgentDef {
  name: string;
  persona: string;
  model: string;
  tools: string[];
  knowledge: string[];
  skills: string[];
}

interface ComponentSummary {
  id: string;
  name: string;
  component_type: string;
  status: string;
}

interface PresetInfo {
  id: string;
  name: string;
  description: string;
}

const PERSONAS = ["assistant", "analyst", "researcher", "writer", "coder", "translator", "reviewer", "cs_agent"];
const MODELS = ["deepseek-chat", "deepseek-reasoner", "gpt-4o", "claude-3"];

const PRESET_TO_PERSONA: Record<string, string> = {
  "preset-analyst": "analyst",
  "preset-researcher": "researcher",
  "preset-writer": "writer",
  "preset-coder": "coder",
  "preset-translator": "translator",
  "preset-assistant": "assistant",
  "preset-reviewer": "reviewer",
  "preset-cs-agent": "cs_agent",
};

const NL_EXAMPLES = [
  "创建一个股票分析助手，能获取行情数据、计算 MACD/RSI 指标、分析市场情绪并生成报告",
  "帮我写一个生物文献翻译 Agent，能查 PubMed，翻译论文并总结要点",
  "一个代码审查助手，可以检查代码质量、运行测试并生成审查报告",
  "数据分析 Agent，能搜索网络、读取文件、计算指标并生成图表",
];

export function AgentBuilder() {
  const [step, setStep] = useState(0);
  const [agentId, setAgentId] = useState<string | null>(null);
  const [building, setBuilding] = useState(false);
  const [publishing, setPublishing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [publishMsg, setPublishMsg] = useState<string | null>(null);
  const [mode, setMode] = useState<"nl" | "form">("nl");
  const [nlInput, setNlInput] = useState("");
  const [nlLoading, setNlLoading] = useState(false);
  const [editTab, setEditTab] = useState<"form" | "visual">("form");
  const [def, setDef] = useState<AgentDef>({
    name: "",
    persona: "assistant",
    model: "deepseek-chat",
    tools: [],
    knowledge: [],
    skills: [],
  });
  const [tools, setTools] = useState<string[]>([]);
  const [knowledge, setKnowledge] = useState<string[]>([]);
  const [skills, setSkills] = useState<string[]>([]);
  const [presets, setPresets] = useState<PresetInfo[]>([]);

  useEffect(() => {
    invoke<ComponentSummary[]>("list_components", { typeFilter: null }).then((list) => {
      setTools(list.filter((c) => c.component_type === "tool").map((c) => c.id));
      setKnowledge(list.filter((c) => c.component_type === "knowledge").map((c) => c.id));
      setSkills(list.filter((c) => c.component_type === "skill").map((c) => c.id));
    }).catch(() => {});
    invoke<PresetInfo[]>("list_preset_personas").then(setPresets).catch(() => {});
  }, []);

  const toggleArray = (arr: string[], item: string): string[] => {
    return arr.includes(item) ? arr.filter((x) => x !== item) : [...arr, item];
  };

  const handleNLGenerate = async () => {
    if (!nlInput.trim()) return;
    setNlLoading(true);
    setError(null);
    try {
      const result = await invoke<string>("create_agent_from_description", { nl: nlInput.trim() });
      const parsed: AgentDef = JSON.parse(result);
      setDef(parsed);
      setStep(1);
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setNlLoading(false);
    }
  };

  const buildFromNaturalLanguage = async (description: string) => {
    const desc = description.toLowerCase();
    const persona = desc.includes("code") || desc.includes("program") ? "coder"
      : desc.includes("research") || desc.includes("science") ? "researcher"
      : desc.includes("analyst") || desc.includes("data") ? "analyst"
      : desc.includes("write") ? "writer"
      : "assistant";
    const selectedTools = desc.includes("search") || desc.includes("web") ? ["web_search"]
      : ["web_search", "read_file"];
    setDef({ ...def, name: description, persona, tools: selectedTools });
    setStep(1);
  };

  const handlePresetSelect = async (presetId: string) => {
    try {
      const persona = await invoke<any>("get_preset_persona", { name: presetId });
      const simpleName = PRESET_TO_PERSONA[presetId] || "assistant";
      setDef({ ...def, name: persona.name, persona: simpleName });
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleBuild = async () => {
    try {
      setBuilding(true);
      setError(null);
      const result = await invoke<{ agent_id: string }>("assemble_agent", {
        name: def.name,
        persona: def.persona,
        model: def.model,
        tools: def.tools,
        knowledge: def.knowledge,
        skills: def.skills,
      });
      setAgentId(result.agent_id);
      setStep(2);
    } catch (e: any) {
      setError(e.toString());
    } finally {
      setBuilding(false);
    }
  };

  const handlePublish = async () => {
    if (!agentId) return;
    try {
      setPublishing(true);
      setPublishMsg(null);
      await invoke("publish_component", { id: agentId });
      setPublishMsg("Published to Workbench successfully");
    } catch (e: any) {
      setPublishMsg("Publish failed: " + e.toString());
    } finally {
      setPublishing(false);
    }
  };

  const renderModeToggle = () => {
    if (step !== 0) return null;
    return (
      <div className="mode-toggle" style={{ display: "flex", gap: "8px", marginBottom: "16px" }}>
        <button
          className={mode === "nl" ? "active" : ""}
          onClick={() => setMode("nl")}
          style={{
            flex: 1, padding: "8px 16px", borderRadius: "6px", border: "1px solid var(--border)",
            background: mode === "nl" ? "var(--accent)" : "var(--bg-secondary)",
            color: mode === "nl" ? "#fff" : "var(--text-primary)", cursor: "pointer",
          }}
        >
          自然语言描述
        </button>
        <button
          className={mode === "form" ? "active" : ""}
          onClick={() => setMode("form")}
          style={{
            flex: 1, padding: "8px 16px", borderRadius: "6px", border: "1px solid var(--border)",
            background: mode === "form" ? "var(--accent)" : "var(--bg-secondary)",
            color: mode === "form" ? "#fff" : "var(--text-primary)", cursor: "pointer",
          }}
        >
          手动编辑
        </button>
      </div>
    );
  };

  const renderEditTabs = () => {
    if (step !== 1) return null;
    return (
      <div className="mode-toggle" style={{ display: "flex", gap: "8px", marginBottom: "16px" }}>
        <button
          className={editTab === "form" ? "active" : ""}
          onClick={() => setEditTab("form")}
          style={{
            flex: 1, padding: "8px 16px", borderRadius: "6px", border: "1px solid var(--border)",
            background: editTab === "form" ? "var(--accent)" : "var(--bg-secondary)",
            color: editTab === "form" ? "#fff" : "var(--text-primary)", cursor: "pointer",
          }}
        >
          表单编辑
        </button>
        <button
          className={editTab === "visual" ? "active" : ""}
          onClick={() => setEditTab("visual")}
          style={{
            flex: 1, padding: "8px 16px", borderRadius: "6px", border: "1px solid var(--border)",
            background: editTab === "visual" ? "var(--accent)" : "var(--bg-secondary)",
            color: editTab === "visual" ? "#fff" : "var(--text-primary)", cursor: "pointer",
          }}
        >
          可视化编辑
        </button>
      </div>
    );
  };

  const renderStep = () => {
    switch (step) {
      case 0:
        return (
          <div className="studio-step">
            {renderModeToggle()}
            {mode === "nl" ? (
              <>
                <h3>用自然语言描述你的 Agent</h3>
                <p>输入一句话，AI 自动分析并生成 Agent 配置</p>
                <textarea
                  value={nlInput}
                  onChange={(e) => setNlInput(e.target.value)}
                  placeholder='例如："创建一个股票分析助手，能获取行情数据、计算技术指标并生成报告"'
                  rows={5}
                  style={{
                    width: "100%", padding: "12px", borderRadius: "6px",
                    border: "1px solid var(--border)", background: "var(--bg-secondary)",
                    color: "var(--text-primary)", fontSize: "14px", resize: "vertical",
                    fontFamily: "inherit",
                  }}
                />
                <button
                  onClick={handleNLGenerate}
                  disabled={nlLoading || !nlInput.trim()}
                  style={{ marginTop: "12px" }}
                >
                  {nlLoading ? "AI 分析中..." : "生成 Agent"}
                </button>
                <div style={{ marginTop: "16px", fontSize: "13px", color: "var(--text-secondary)" }}>
                  <p>示例：</p>
                  {NL_EXAMPLES.map((ex, i) => (
                    <div
                      key={i}
                      onClick={() => setNlInput(ex)}
                      style={{
                        cursor: "pointer", padding: "6px 10px", margin: "4px 0",
                        borderRadius: "4px", background: "var(--bg-tertiary)",
                        border: "1px solid var(--border)",
                      }}
                    >
                      {ex}
                    </div>
                  ))}
                </div>
              </>
            ) : (
              <>
                <h3>Describe your Agent</h3>
                <p>Tell me what you want the agent to do in natural language</p>
                <input
                  type="text"
                  placeholder="e.g. A biology research agent"
                  value={def.name}
                  onChange={(e) => setDef({ ...def, name: e.target.value })}
                  onKeyDown={(e) => e.key === "Enter" && buildFromNaturalLanguage(def.name)}
                />
                <button onClick={() => buildFromNaturalLanguage(def.name)}>Build</button>
              </>
            )}
            {error && <div className="error-indicator">{error}</div>}
          </div>
        );
      case 1:
        return (
          <div className="studio-step">
            {renderEditTabs()}
            {editTab === "form" ? (
              <>
                <h3>Configure Persona & Model</h3>
                <label>Agent Name:</label>
                <input
                  type="text"
                  value={def.name}
                  onChange={(e) => setDef({ ...def, name: e.target.value })}
                />
                <label>Persona:</label>
                <select value={def.persona} onChange={(e) => setDef({ ...def, persona: e.target.value })}>
                  {PERSONAS.map((p) => <option key={p} value={p}>{p}</option>)}
                </select>
                <label>Model:</label>
                <select value={def.model} onChange={(e) => setDef({ ...def, model: e.target.value })}>
                  {MODELS.map((m) => <option key={m} value={m}>{m}</option>)}
                </select>
                <label>Tools:</label>
                <div className="checkbox-group">
                  {tools.map((t) => (
                    <label key={t}>
                      <input type="checkbox" checked={def.tools.includes(t)}
                        onChange={() => setDef({ ...def, tools: toggleArray(def.tools, t) })} />
                      {t}
                    </label>
                  ))}
                </div>
                <label>Knowledge:</label>
                <div className="checkbox-group">
                  {knowledge.map((k) => (
                    <label key={k}>
                      <input type="checkbox" checked={def.knowledge.includes(k)}
                        onChange={() => setDef({ ...def, knowledge: toggleArray(def.knowledge, k) })} />
                      {k}
                    </label>
                  ))}
                </div>
                <label>Skills:</label>
                <div className="checkbox-group">
                  {skills.map((s) => (
                    <label key={s}>
                      <input type="checkbox" checked={def.skills.includes(s)}
                        onChange={() => setDef({ ...def, skills: toggleArray(def.skills, s) })} />
                      {s}
                    </label>
                  ))}
                </div>
                {presets.length > 0 && (
                  <div className="template-selector" style={{ marginTop: "16px" }}>
                    <h4 style={{ marginBottom: "8px", fontSize: "14px", color: "var(--text-secondary)" }}>预置人格模板</h4>
                    <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: "8px" }}>
                      {presets.map((p) => (
                        <div
                          key={p.id}
                          onClick={() => handlePresetSelect(p.id)}
                          style={{
                            cursor: "pointer", padding: "10px", borderRadius: "6px",
                            border: "1px solid var(--border)", background: "var(--bg-tertiary)",
                            transition: "border-color 0.2s",
                          }}
                          onMouseEnter={(e) => { (e.currentTarget as HTMLElement).style.borderColor = "var(--accent)"; }}
                          onMouseLeave={(e) => { (e.currentTarget as HTMLElement).style.borderColor = ""; }}
                          className="preset-template-card"
                        >
                          <strong style={{ fontSize: "13px", display: "block", marginBottom: "4px" }}>{p.name}</strong>
                          <span style={{ fontSize: "11px", color: "var(--text-secondary)" }}>{p.description}</span>
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </>
            ) : (
              <>
                <h3>可视化节点编辑</h3>
                <p style={{ fontSize: "13px", color: "var(--text-secondary)", marginBottom: "12px" }}>
                  从左侧组件库拖拽组件到画布，连接端口构建 Agent
                </p>
                <NodeCanvas def={def} onDefChange={setDef} />
              </>
            )}
            <div className="step-buttons" style={{ marginTop: "12px" }}>
              <button onClick={() => setStep(0)}>Back</button>
              <button onClick={handleBuild} disabled={building}>
                {building ? "Building..." : "Build Agent"}
              </button>
            </div>
            {error && <div className="error-indicator">{error}</div>}
          </div>
        );
      case 2:
        return (
          <div className="studio-step">
            <h3>Agent Created Successfully!</h3>
            <div className="test-panel">
              <p>Agent: {def.name}</p>
              <p>Persona: {def.persona}</p>
              <p>Model: {def.model}</p>
              {agentId && <p className="agent-id">Agent ID: {agentId}</p>}
              <div className="component-breakdown">
                <h4>Component Breakdown</h4>
                <p><strong>Tools:</strong> {def.tools.length > 0 ? def.tools.join(", ") : "None"}</p>
                <p><strong>Knowledge:</strong> {def.knowledge.length > 0 ? def.knowledge.join(", ") : "None"}</p>
                <p><strong>Skills:</strong> {def.skills.length > 0 ? def.skills.join(", ") : "None"}</p>
              </div>
            </div>
            <div className="step-buttons">
              <button onClick={handlePublish} disabled={publishing || !agentId}>
                {publishing ? "Publishing..." : "Publish to Workbench"}
              </button>
              <button onClick={() => { setStep(0); setAgentId(null); setPublishMsg(null); }}>Create Another</button>
            </div>
            {publishMsg && <div className={publishMsg.startsWith("Publish failed") ? "error-indicator" : "success-indicator"}>{publishMsg}</div>}
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <div className="agent-builder">
      <h2>Agent Builder</h2>
      <div className="steps-indicator">
        <span className={step >= 0 ? "active" : ""}>Describe</span>
        <span className={step >= 1 ? "active" : ""}>Configure</span>
        <span className={step >= 2 ? "active" : ""}>Done</span>
      </div>
      {renderStep()}
    </div>
  );
}