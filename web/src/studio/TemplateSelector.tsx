import { useState } from "react";

interface Template {
  id: string;
  name: string;
  description: string;
  persona: string;
  tools: string[];
  icon: string;
}

const TEMPLATES: Template[] = [
  { id: "research", name: "Researcher", description: "Rigorous multi-source research agent", persona: "researcher", tools: ["web_search", "read_file", "exec_python"], icon: "🔬" },
  { id: "analyst", name: "Analyst", description: "Data-driven analysis and insights", persona: "analyst", tools: ["web_search", "read_file", "calc"], icon: "📊" },
  { id: "coder", name: "Coder", description: "Code generation and review", persona: "coder", tools: ["read_file", "write_file", "exec_python"], icon: "💻" },
  { id: "writer", name: "Writer", description: "Content creation and editing", persona: "writer", tools: ["web_search", "read_file"], icon: "✍️" },
  { id: "assistant", name: "Assistant", description: "General purpose helper", persona: "assistant", tools: ["web_search", "read_file", "get_time", "calc"], icon: "🤖" },
  { id: "translator", name: "Translator", description: "Multi-language translation", persona: "translator", tools: ["web_search"], icon: "🌐" },
  { id: "reviewer", name: "Reviewer", description: "Code and document review specialist", persona: "reviewer", tools: ["read_file"], icon: "👁️" },
];

export function TemplateSelector({ onSelect }: { onSelect?: (template: Template) => void }) {
  const [selected, setSelected] = useState<string | null>(null);

  return (
    <div className="template-selector">
      <h2>Choose a Template</h2>
      <div className="template-grid">
        {TEMPLATES.map((t) => (
          <div
            key={t.id}
            className={`template-card ${selected === t.id ? "selected" : ""}`}
            onClick={() => {
              setSelected(t.id);
              onSelect?.(t);
            }}
          >
            <div className="template-icon">{t.icon}</div>
            <div className="template-name">{t.name}</div>
            <div className="template-desc">{t.description}</div>
            <div className="template-tools">{t.tools.join(", ")}</div>
          </div>
        ))}
      </div>
    </div>
  );
}