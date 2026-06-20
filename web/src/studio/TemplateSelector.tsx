import { useState, useEffect } from "react";
import { api } from "../api";
import { useTranslation } from '../i18n';

interface Template {
  id: string;
  name: string;
  icon: string;
  description: string;
  persona: string;
  model: string;
  tools: string[];
  knowledge: string[];
  skills: string[];
}

const FALLBACK_TEMPLATES: Template[] = [
  { id: "research-assistant", name: "研究助手", icon: "🔬", description: "多源信息检索、知识整合与摘要生成", persona: "researcher", model: "deepseek-chat", tools: ["web_search", "read_file"], knowledge: ["docs", "data_sources"], skills: ["summarization", "report_generation"] },
  { id: "data-analyst", name: "数据分析师", icon: "📊", description: "获取行情数据、计算技术指标、生成图表", persona: "analyst", model: "deepseek-chat", tools: ["get_kline", "calc_macd", "chart", "exec_python"], knowledge: ["docs", "data_sources"], skills: ["report_generation"] },
  { id: "writing-assistant", name: "写作助手", icon: "✍️", description: "翻译、语法检查、格式润色与风格优化", persona: "writer", model: "deepseek-chat", tools: ["web_search", "read_file", "write_file"], knowledge: ["glossary"], skills: ["translation", "grammar_check", "format", "style", "proofread"] },
  { id: "coding-helper", name: "编码助手", icon: "💻", description: "代码审查、调试、格式化和测试", persona: "coder", model: "deepseek-chat", tools: ["read_file", "write_file", "exec_python"], knowledge: ["docs"], skills: ["code_review", "debug", "format", "test"] },
  { id: "translation-agent", name: "翻译 Agent", icon: "🌐", description: "多语言翻译、校对和专业术语管理", persona: "translator", model: "deepseek-chat", tools: ["web_search", "read_file"], knowledge: ["glossary"], skills: ["translation", "proofread"] },
  { id: "general-assistant", name: "通用助手", icon: "🤖", description: "混合工具集的通用助手，适合日常问答", persona: "assistant", model: "deepseek-chat", tools: ["web_search", "read_file", "get_time", "calc"], knowledge: ["docs"], skills: [] },
];

interface TemplateSelectorProps {
  onSelect?: (template: Template) => void;
}

export function TemplateSelector({ onSelect }: TemplateSelectorProps) {
  const { t } = useTranslation();
  const [selected, setSelected] = useState<string | null>(null);
  const [templates, setTemplates] = useState<Template[]>(FALLBACK_TEMPLATES);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.listAgentTemplates().then((list: Template[]) => {
      if (list.length > 0) {
        setTemplates(list);
      }
    }).catch(() => {}).finally(() => setLoading(false));
  }, []);

  const componentTags = (t: Template): string[] => {
    const tags: string[] = [];
    tags.push(`🧑 ${t.persona}`);
    tags.push(`🤖 ${t.model}`);
    if (t.tools.length > 0) tags.push(`🔧 ${t.tools.length} tools`);
    if (t.skills.length > 0) tags.push(`⚡ ${t.skills.length} skills`);
    return tags;
  };

  return (
    <div className="template-selector">
      <h2>{t('studio.teams.select_template')}</h2>
      {loading && <p style={{ color: "var(--text-secondary)" }}>{t('template_selector.loading')}</p>}
      <div className="template-grid" style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(240px, 1fr))", gap: "12px" }}>
        {templates.map((tmpl) => (
          <div
            key={tmpl.id}
            className={`template-card ${selected === tmpl.id ? "selected" : ""}`}
            onClick={() => {
              setSelected(tmpl.id);
              onSelect?.(tmpl);
            }}
            style={{
              background: selected === tmpl.id ? "var(--bg-tertiary)" : "var(--bg-secondary)",
              border: selected === tmpl.id ? "2px solid var(--accent)" : "1px solid var(--border)",
              borderRadius: "8px",
              padding: "16px",
              cursor: "pointer",
              transition: "all 0.15s ease",
            }}
          >
            <div style={{ fontSize: "28px", marginBottom: "8px" }}>{tmpl.icon}</div>
            <div style={{ fontWeight: 600, color: "var(--text-primary)", marginBottom: "4px", fontSize: "15px" }}>{t(`template_selector.${tmpl.id}`)}</div>
            <div style={{ fontSize: "13px", color: "var(--text-secondary)", marginBottom: "10px", lineHeight: "1.4" }}>{t(`template_selector.${tmpl.id}_desc`)}</div>
            <div style={{ display: "flex", flexWrap: "wrap", gap: "4px", marginBottom: "12px" }}>
              {componentTags(tmpl).map((tag, i) => (
                <span key={i} style={{
                  fontSize: "11px", padding: "2px 6px", borderRadius: "4px",
                  background: "var(--bg-tertiary)", color: "var(--text-secondary)",
                  border: "1px solid var(--border)",
                }}>
                  {tag}
                </span>
              ))}
            </div>
            <button
              onClick={(e) => { e.stopPropagation(); onSelect?.(tmpl); }}
              style={{
                width: "100%", padding: "6px 12px", borderRadius: "6px",
                background: "var(--accent)", color: "#fff", border: "none",
                cursor: "pointer", fontSize: "13px", fontWeight: 500,
              }}
            >
              {t('template_selector.use_template')}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}
