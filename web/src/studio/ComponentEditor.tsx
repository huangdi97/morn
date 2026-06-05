import { useState } from "react";

type ComponentType = "tool" | "knowledge" | "skill" | "persona" | "memory" | "model";

interface ComponentDef {
  name: string;
  type: ComponentType;
  config: string;
}

export function ComponentEditor() {
  const [type, setType] = useState<ComponentType>("tool");
  const [def, setDef] = useState<ComponentDef>({ name: "", type: "tool", config: "{}" });
  const [saved, setSaved] = useState(false);

  const handleSave = () => {
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const renderConfigEditor = () => {
    switch (type) {
      case "tool":
        return (
          <div>
            <p>Tool Configuration</p>
            <textarea
              value={def.config}
              onChange={(e) => setDef({ ...def, config: e.target.value })}
              rows={6}
            />
          </div>
        );
      case "knowledge":
        return (
          <div>
            <p>Knowledge Source</p>
            <select>
              <option value="static">Static (key-value pairs)</option>
              <option value="file">File (JSON/YAML/CSV)</option>
              <option value="sqlite">SQLite Table</option>
            </select>
          </div>
        );
      case "skill":
        return (
          <div>
            <p>Skill Steps (one per line: tool_id -> depends_on)</p>
            <textarea
              value={def.config}
              onChange={(e) => setDef({ ...def, config: e.target.value })}
              rows={6}
              placeholder="web_search&#10;summarize -> web_search"
            />
          </div>
        );
      case "persona":
        return (
          <div>
            <p>Persona Prompt Layers</p>
            <textarea
              value={def.config}
              onChange={(e) => setDef({ ...def, config: e.target.value })}
              rows={6}
              placeholder="L1: Core identity&#10;L2: Skills&#10;L3: Format template"
            />
          </div>
        );
      case "memory":
        return (
          <div>
            <p>Memory Configuration</p>
            <select>
              <option value="sqlite">SQLite (working memory)</option>
              <option value="vector">Vector Store (future)</option>
            </select>
          </div>
        );
      case "model":
        return (
          <div>
            <p>Model Configuration</p>
            <label>Provider:</label>
            <select>
              <option value="deepseek">DeepSeek</option>
              <option value="openai">OpenAI</option>
              <option value="anthropic">Anthropic</option>
              <option value="local">Local</option>
            </select>
            <label>Model Name:</label>
            <input type="text" placeholder="deepseek-chat" />
            <label>Temperature:</label>
            <input type="range" min="0" max="2" step="0.1" defaultValue="0.6" />
          </div>
        );
    }
  };

  return (
    <div className="component-editor">
      <h2>Component Editor</h2>
      <div className="editor-form">
        <label>Type:</label>
        <select value={type} onChange={(e) => setType(e.target.value as ComponentType)}>
          <option value="tool">Tool</option>
          <option value="knowledge">Knowledge</option>
          <option value="skill">Skill</option>
          <option value="persona">Persona</option>
          <option value="memory">Memory</option>
          <option value="model">Model</option>
        </select>
        <label>Name:</label>
        <input
          type="text"
          value={def.name}
          onChange={(e) => setDef({ ...def, name: e.target.value })}
          placeholder="Component name"
        />
        {renderConfigEditor()}
        <button onClick={handleSave}>Save Component</button>
        {saved && <span className="saved-indicator">Saved!</span>}
      </div>
    </div>
  );
}