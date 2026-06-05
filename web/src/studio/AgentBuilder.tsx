import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

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

const PERSONAS = ["assistant", "analyst", "researcher", "writer", "coder", "translator", "reviewer"];
const MODELS = ["deepseek-chat", "deepseek-reasoner", "gpt-4o", "claude-3"];

export function AgentBuilder() {
  const [step, setStep] = useState(0);
  const [agentId, setAgentId] = useState<string | null>(null);
  const [building, setBuilding] = useState(false);
  const [publishing, setPublishing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [publishMsg, setPublishMsg] = useState<string | null>(null);
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

  useEffect(() => {
    invoke<ComponentSummary[]>("list_components", { typeFilter: null }).then((list) => {
      setTools(list.filter((c) => c.component_type === "tool").map((c) => c.id));
      setKnowledge(list.filter((c) => c.component_type === "knowledge").map((c) => c.id));
      setSkills(list.filter((c) => c.component_type === "skill").map((c) => c.id));
    }).catch(() => {});
  }, []);

  const toggleArray = (arr: string[], item: string): string[] => {
    return arr.includes(item) ? arr.filter((x) => x !== item) : [...arr, item];
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
      await invoke("publish_agent", { agentId });
      setPublishMsg("Published to Workbench successfully");
    } catch (e: any) {
      setPublishMsg("Publish failed: " + e.toString());
    } finally {
      setPublishing(false);
    }
  };

  const renderStep = () => {
    switch (step) {
      case 0:
        return (
          <div className="studio-step">
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
          </div>
        );
      case 1:
        return (
          <div className="studio-step">
            <h3>Configure Persona & Model</h3>
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
            <div className="step-buttons">
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
