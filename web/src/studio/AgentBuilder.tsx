import { useState } from "react";

interface AgentDef {
  name: string;
  persona: string;
  model: string;
  tools: string[];
  knowledge: string[];
  skills: string[];
}

const PERSONAS = ["assistant", "analyst", "researcher", "writer", "coder", "translator", "reviewer"];
const MODELS = ["deepseek-chat", "deepseek-reasoner", "gpt-4o", "claude-3"];
const TOOLS = ["web_search", "read_file", "write_file", "exec_python", "get_time", "calc", "send_msg", "http_request"];
const KNOWLEDGE = ["static", "file", "sqlite"];
const SKILLS = ["web_research", "data_analysis", "report_gen", "code_review"];

export function AgentBuilder() {
  const [step, setStep] = useState(0);
  const [def, setDef] = useState<AgentDef>({
    name: "",
    persona: "assistant",
    model: "deepseek-chat",
    tools: [],
    knowledge: [],
    skills: [],
  });

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
    const tools = desc.includes("search") || desc.includes("web") ? ["web_search"]
      : ["web_search", "read_file"];
    setDef({ ...def, name: description, persona, tools });
    setStep(1);
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
              {TOOLS.map((t) => (
                <label key={t}>
                  <input type="checkbox" checked={def.tools.includes(t)}
                    onChange={() => setDef({ ...def, tools: toggleArray(def.tools, t) })} />
                  {t}
                </label>
              ))}
            </div>
            <label>Skills:</label>
            <div className="checkbox-group">
              {SKILLS.map((s) => (
                <label key={s}>
                  <input type="checkbox" checked={def.skills.includes(s)}
                    onChange={() => setDef({ ...def, skills: toggleArray(def.skills, s) })} />
                  {s}
                </label>
              ))}
            </div>
            <div className="step-buttons">
              <button onClick={() => setStep(0)}>Back</button>
              <button onClick={() => setStep(2)}>Test Agent</button>
            </div>
          </div>
        );
      case 2:
        return (
          <div className="studio-step">
            <h3>Test Your Agent</h3>
            <div className="test-panel">
              <p>Agent: {def.name}</p>
              <p>Persona: {def.persona}</p>
              <p>Model: {def.model}</p>
              <p>Tools: {def.tools.join(", ")}</p>
              <p>Skills: {def.skills.join(", ")}</p>
            </div>
            <div className="step-buttons">
              <button onClick={() => setStep(1)}>Back</button>
              <button onClick={() => alert("Agent created!")}>Create Agent</button>
            </div>
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
        <span className={step >= 2 ? "active" : ""}>Test</span>
      </div>
      {renderStep()}
    </div>
  );
}