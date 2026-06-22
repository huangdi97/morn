import { useState, useEffect } from "react";
import { api } from "../api";
import { NodeCanvas } from "./NodeCanvas";
import { StepWizard } from "./StepWizard";
import { AgentDef, ComponentSummary } from "./types";
import { useTranslation } from '../i18n';

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
  const { t } = useTranslation();
  const [showWizard, setShowWizard] = useState(false);
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
  const [isLoading, setIsLoading] = useState(true);
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
    api.listComponents(null).then((list: ComponentSummary[]) => {
      setTools(list.filter((c) => c.component_type === "tool").map((c) => c.id));
      setKnowledge(list.filter((c) => c.component_type === "knowledge").map((c) => c.id));
      setSkills(list.filter((c) => c.component_type === "skill").map((c) => c.id));
      setIsLoading(false);
    }).catch(() => { setIsLoading(false); });
    api.listPresetPersonas().then(setPresets).catch(() => {});
  }, []);

  const toggleArray = (arr: string[], item: string): string[] => {
    return arr.includes(item) ? arr.filter((x) => x !== item) : [...arr, item];
  };

  const handleNLGenerate = async () => {
    if (!nlInput.trim()) return;
    setNlLoading(true);
    setError(null);
    try {
      const result = await api.createAgentFromDescription(nlInput.trim()) as string;
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
      const persona = await api.getPresetPersona(presetId);
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
      if (agentId) {
        await api.updateComponent(agentId, def);
      } else {
        const result = await api.assembleAgent(def) as { agent_id: string };
        setAgentId(result.agent_id);
      }
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
      await api.publishComponent(agentId);
      setPublishMsg(t('agent_builder.publish_success'));
    } catch (e: any) {
      setPublishMsg(t('agent_builder.publish_failed') + " " + e.toString());
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
          {t('agent_builder.nl_mode')}
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
          {t('agent_builder.form_mode')}
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
          {t('agent_builder.form_edit')}
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
          {t('agent_builder.visual_edit')}
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
                <h3>{t('studio.builder.nl_title')}</h3>
                <p>{t('studio.builder.nl_desc')}</p>
                <textarea
                  value={nlInput}
                  onChange={(e) => setNlInput(e.target.value)}
                  placeholder={t('agent_builder.nl_placeholder')}
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
                  {nlLoading ? t('agent_builder.analyzing') : t('agent_builder.generate_agent')}
                </button>
                <div style={{ marginTop: "16px", fontSize: "13px", color: "var(--text-secondary)" }}>
                  <p>{t('studio.builder.examples')}</p>
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
                       {t(`agent_builder.example_${i}`)}
                    </div>
                  ))}
                </div>
              </>
            ) : (
              <>
                <h3>{t('studio.builder.form_title')}</h3>
                <p>{t('studio.builder.form_desc')}</p>
                <input
                  type="text"
                  placeholder={t('agent_builder.form_placeholder')}
                  value={def.name}
                  onChange={(e) => setDef({ ...def, name: e.target.value })}
                  onKeyDown={(e) => e.key === "Enter" && buildFromNaturalLanguage(def.name)}
                />
                <button onClick={() => buildFromNaturalLanguage(def.name)}>{t('studio.builder.build')}</button>
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
                <h3>{t('studio.builder.configure')}</h3>
                <label>{t('studio.builder.agent_name')}</label>
                <input
                  type="text"
                  value={def.name}
                  onChange={(e) => setDef({ ...def, name: e.target.value })}
                />
                <label>{t('studio.builder.persona')}</label>
                <select value={def.persona} onChange={(e) => setDef({ ...def, persona: e.target.value })}>
                  {PERSONAS.map((p) => <option key={p} value={p}>{p}</option>)}
                </select>
                <label>{t('agent_builder.model_label')}</label>
                <select value={def.model} onChange={(e) => setDef({ ...def, model: e.target.value })}>
                  {MODELS.map((m) => <option key={m} value={m}>{m}</option>)}
                </select>
                <label>{t('agent_builder.tools_label')}</label>
                <div className="checkbox-group">
                  {tools.map((t) => (
                    <label key={t}>
                      <input type="checkbox" checked={def.tools.includes(t)}
                        onChange={() => setDef({ ...def, tools: toggleArray(def.tools, t) })} />
                      {t}
                    </label>
                  ))}
                </div>
                <label>{t('agent_builder.knowledge_label')}</label>
                <div className="checkbox-group">
                  {knowledge.map((k) => (
                    <label key={k}>
                      <input type="checkbox" checked={def.knowledge.includes(k)}
                        onChange={() => setDef({ ...def, knowledge: toggleArray(def.knowledge, k) })} />
                      {k}
                    </label>
                  ))}
                </div>
                <label>{t('agent_builder.skills_label')}</label>
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
                    <h4 style={{ marginBottom: "8px", fontSize: "14px", color: "var(--text-secondary)" }}>{t('studio.builder.preset_personas')}</h4>
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
                <h3>{t('studio.builder.visual_editor')}</h3>
                <p style={{ fontSize: "13px", color: "var(--text-secondary)", marginBottom: "12px" }}>
                  {t('agent_builder.visual_desc')}
                </p>
                <NodeCanvas def={def} onDefChange={setDef} />
              </>
            )}
            <div className="step-buttons" style={{ marginTop: "12px" }}>
              <button onClick={() => setStep(0)}>{t('agent_builder.back')}</button>
              <button onClick={handleBuild} disabled={building}>
                {building ? t('agent_builder.building') : (agentId ? t('agent_builder.update_agent') : t('agent_builder.build_agent'))}
              </button>
            </div>
            {error && <div className="error-indicator">{error}</div>}
          </div>
        );
      case 2:
        return (
          <div className="studio-step">
            <h3>{t('studio.builder.created')}</h3>
            <div className="test-panel">
              <p>Agent: {def.name}</p>
              <p>Persona: {def.persona}</p>
              <p>Model: {def.model}</p>
              {agentId && <p className="agent-id">Agent ID: {agentId}</p>}
              <NodeCanvas def={def} onDefChange={(newDef) => { setDef(newDef); setEditTab("visual"); setStep(1); }} />
            </div>
            <div className="step-buttons">
              <button onClick={handlePublish} disabled={publishing || !agentId}>
                {publishing ? t('agent_builder.publishing') : t('agent_builder.publish')}
              </button>
              <button onClick={() => { setEditTab("visual"); setStep(1); }}>
                {t('agent_builder.edit_agent')}
              </button>
              <button onClick={() => { setStep(0); setAgentId(null); setPublishMsg(null); }}>{t('agent_builder.create_another')}</button>
            </div>
            {publishMsg && <div className={publishMsg.startsWith(t('agent_builder.publish_failed')) ? "error-indicator" : "success-indicator"}>{publishMsg}</div>}
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <div className="agent-builder">
      {isLoading ? (
        <div className="skeleton-canvas">
          <div className="skeleton-canvas-sidebar">
            {[1,2,3,4].map(i => <div key={i} className="skeleton" />)}
          </div>
          <div className="skeleton-canvas-main">
            {[1,2,3].map(i => <div key={i} className="skeleton" />)}
          </div>
        </div>
      ) : (<>
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "16px" }}>
        <h2 style={{ margin: 0 }}>{t('studio.builder.title')}</h2>
        <button
          onClick={() => setShowWizard(true)}
          style={{
            padding: "8px 16px", borderRadius: "6px",
            background: "var(--accent)", color: "#fff", border: "none",
            cursor: "pointer", fontSize: "13px", fontWeight: 500,
          }}
        >
          {t('agent_builder.wizard_new')}
        </button>
      </div>
      {showWizard && <StepWizard onClose={() => setShowWizard(false)} />}
      <div className="steps-indicator">
        <span className={step >= 0 ? "active" : ""}>{t('agent_builder.step_describe')}</span>
        <span className={step >= 1 ? "active" : ""}>{t('studio.builder.step_configure')}</span>
        <span className={step >= 2 ? "active" : ""}>{t('agent_builder.step_done')}</span>
      </div>
      {renderStep()}
      </>
      )}
    </div>
  );
}
