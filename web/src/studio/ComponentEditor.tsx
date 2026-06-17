import { useState, useEffect } from "react";
import { api } from "../api";
import { ComponentSummary } from "./types";

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
  const [lastCreatedId, setLastCreatedId] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [components, setComponents] = useState<ComponentSummary[]>([]);
  const [editingId, setEditingId] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);

  const loadComponents = async () => {
    try {
      const list = await api.listComponents(null) as ComponentSummary[];
      setComponents(list);
    } catch (e: any) {
      setError(e.toString());
    }
  };

  useEffect(() => {
    loadComponents();
  }, []);

  const selectComponent = async (c: ComponentSummary) => {
    setSelectedId(c.id);
    setType(c.component_type as ComponentType);
    setDef({ name: c.name, type: c.component_type as ComponentType, config: "{}" });
    setEditingId(null);
    try {
      const detail = await api.getComponent(c.id);
      setDef({ name: c.name, type: c.component_type as ComponentType, config: detail.config_json ?? "{}" });
      setEditingId(c.id);
    } catch {
      setDef({ name: c.name, type: c.component_type as ComponentType, config: "{}" });
    }
  };

  const handleSave = async () => {
    try {
      setError(null);
      if (editingId) {
        await api.updateComponent(editingId, {
          name: def.name,
          componentType: def.type,
          configJson: def.config,
        });
      } else {
        const id = await api.createComponent({
          name: def.name,
          componentType: def.type,
          configJson: def.config,
        }) as string;
        setLastCreatedId(id);
      }
      setSaved(true);
      setDef({ name: "", type: "tool", config: "{}" });
      setEditingId(null);
      setSelectedId(null);
      await loadComponents();
      setTimeout(() => setSaved(false), 2000);
    } catch (e: any) {
      setError(e.toString());
    }
  };

  const handleDelete = async (id: string) => {
    try {
      setError(null);
      await api.deleteComponent(id);
      if (selectedId === id) {
        setDef({ name: "", type: "tool", config: "{}" });
        setEditingId(null);
        setSelectedId(null);
      }
      await loadComponents();
    } catch (e: any) {
      setError(e.toString());
    }
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
            <p>Skill Steps (one per line: tool_id → depends_on)</p>
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
    <div className="component-editor" style={{ display: "flex", gap: "16px" }}>
      <div className="component-list" style={{ width: "250px", flexShrink: 0 }}>
        <h3>Existing Components</h3>
        {components.length === 0 ? (
          <div style={{
            padding: "16px",
            textAlign: "center",
            color: "var(--text-secondary)",
            fontSize: "13px",
            border: "1px dashed var(--border)",
            borderRadius: "6px",
            background: "var(--bg-secondary)",
          }}>
            No components yet. Create one in the Agent Builder.
          </div>
        ) : components.map((c) => (
          <div
            key={c.id}
            className={`component-list-item${selectedId === c.id ? " selected" : ""}`}
            onClick={() => selectComponent(c)}
          >
            <span className="component-type-badge">{c.component_type}</span>
            <span className="component-name">{c.name}</span>
            <span className="component-status">{c.status}</span>
            <button
              className="component-delete-btn"
              onClick={(e) => { e.stopPropagation(); handleDelete(c.id); }}
            >
              Delete
            </button>
          </div>
        ))}
      </div>
      <div className="editor-form" style={{ flex: 1 }}>
        <h2>Component Editor</h2>
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
        <button onClick={handleSave}>
          {editingId ? "Update Component" : "Save Component"}
        </button>
        {saved && (
          <span className="saved-indicator">
            Saved!{editingId ? "" : ` ID: ${lastCreatedId}`}
          </span>
        )}
        {error && <span className="error-indicator">Error: {error}</span>}
      </div>
    </div>
  );
}
