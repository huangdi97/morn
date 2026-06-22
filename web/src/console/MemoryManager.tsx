import { useEffect, useState } from "react";
import { useTranslation } from '../i18n';
import { EmptyState } from "../components/EmptyState";

interface Memory {
  id: string;
  type: "dialogue" | "knowledge" | "config" | "skill";
  content: string;
  created_at: string;
  source: string;
}

type MemoryFilter = "all" | "dialogue" | "knowledge" | "config" | "skill";

const MEMORY_TYPES: MemoryFilter[] = ["all", "dialogue", "knowledge", "config", "skill"];

export default function MemoryManager() {
  const { t } = useTranslation();
  const [memories, setMemories] = useState<Memory[]>([]);
  const [query, setQuery] = useState("");
  const [filter, setFilter] = useState<MemoryFilter>("all");
  const [selected, setSelected] = useState<Memory | null>(null);
  const [deleteTarget, setDeleteTarget] = useState<Memory | null>(null);

  useEffect(() => {
    loadMemories();
  }, []);

  const loadMemories = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const list = await invoke<Memory[]>("list_memories");
      setMemories(list);
    } catch (e) {
      console.error("Failed to list memories", e);
    }
  };

  const handleSearch = async () => {
    if (!query.trim()) {
      loadMemories();
      return;
    }
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const results = await invoke<Memory[]>("search_memories", { q: query });
      setMemories(results);
    } catch (e) {
      console.error("Failed to search memories", e);
    }
  };

  const handleDelete = async () => {
    if (!deleteTarget) return;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("delete_memory", { id: deleteTarget.id });
      setMemories(prev => prev.filter(m => m.id !== deleteTarget.id));
      if (selected?.id === deleteTarget.id) setSelected(null);
    } catch (e) {
      console.error("Failed to delete memory", e);
    }
    setDeleteTarget(null);
  };

  const filtered = memories.filter(m => filter === "all" || m.type === filter);

  const highlightMatch = (text: string) => {
    if (!query.trim()) return text;
    const idx = text.toLowerCase().indexOf(query.toLowerCase());
    if (idx === -1) return text;
    const before = text.slice(0, idx);
    const match = text.slice(idx, idx + query.length);
    const after = text.slice(idx + query.length);
    return (
      <>{before}<strong style={{ background: "var(--accent-brand)", color: "#fff", borderRadius: "2px", padding: "0 2px" }}>{match}</strong>{after}</>
    );
  };

  return (
    <div className="memory-manager">
      <h2>{t('console.memory.title')}</h2>
      <div className="memory-search" style={{ display: "flex", gap: "8px", marginBottom: "12px" }}>
        <input
          type="text"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleSearch()}
          placeholder="Search memories..."
          style={{ flex: 1 }}
        />
        <button onClick={handleSearch}>{t('console.memory.search')}</button>
      </div>
      <div className="memory-filters" style={{ display: "flex", gap: "6px", marginBottom: "12px", flexWrap: "wrap" }}>
        {MEMORY_TYPES.map(type => (
          <button
            key={type}
            onClick={() => setFilter(type)}
            style={{
              padding: "4px 12px", borderRadius: "14px", border: `1px solid var(--border-default)`,
              background: filter === type ? "var(--accent-brand)" : "transparent",
              color: filter === type ? "#fff" : "var(--text-secondary)",
              cursor: "pointer", fontSize: "12px",
            }}
          >
            {t(`console.memory.filter_${type}`)}
          </button>
        ))}
      </div>
      {filtered.length === 0 ? (
        <EmptyState icon="🧠" title="记忆为空" description="暂无记忆数据，与 Agent 对话会自动生成记忆。" />
      ) : (
        <table className="memory-table" style={{ width: "100%", borderCollapse: "collapse", fontSize: "13px" }}>
          <thead>
            <tr style={{ color: "var(--text-tertiary)", borderBottom: "1px solid var(--border-default)" }}>
              <th style={{ padding: "8px", textAlign: "left" }}>{t('console.memory.header')}</th>
              <th style={{ padding: "8px", textAlign: "left", width: "80px" }}>{t('console.memory.detail_source')}</th>
              <th style={{ padding: "8px", textAlign: "left", width: "100px" }}>{t('console.memory.detail_created')}</th>
              <th style={{ padding: "8px", width: "60px" }}></th>
            </tr>
          </thead>
          <tbody>
            {filtered.map(m => (
              <>
                <tr
                  key={m.id}
                  onClick={() => setSelected(selected?.id === m.id ? null : m)}
                  style={{
                    cursor: "pointer", borderBottom: "1px solid var(--border-subtle)",
                    background: selected?.id === m.id ? "var(--bg-surface-hover)" : "transparent",
                  }}
                >
                  <td style={{ padding: "8px" }}>
                    <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
                      <span style={{
                        fontSize: "10px", padding: "1px 6px", borderRadius: "8px",
                        background: m.type === "dialogue" ? "#58a6ff33" : m.type === "knowledge" ? "#3fb95033" : m.type === "config" ? "#d2992233" : "#bc8cff33",
                        color: m.type === "dialogue" ? "#58a6ff" : m.type === "knowledge" ? "#3fb950" : m.type === "config" ? "#d29922" : "#bc8cff",
                      }}>
                        {m.type}
                      </span>
                      <span style={{ color: "var(--text-primary)" }}>
                        {query.trim() ? highlightMatch(m.content) : m.content.length > 60 ? m.content.slice(0, 60) + "..." : m.content}
                      </span>
                    </div>
                  </td>
                  <td style={{ padding: "8px", color: "var(--text-tertiary)", fontSize: "12px" }}>{m.source}</td>
                  <td style={{ padding: "8px", color: "var(--text-tertiary)", fontSize: "12px" }}>{new Date(m.created_at).toLocaleDateString()}</td>
                  <td style={{ padding: "8px", textAlign: "center" }}>
                    <button
                      onClick={(e) => { e.stopPropagation(); setDeleteTarget(m); }}
                      style={{
                        padding: "2px 8px", fontSize: "11px", border: "1px solid var(--border-default)",
                        borderRadius: "4px", background: "transparent", color: "#f85149", cursor: "pointer",
                      }}
                    >
                      {t('console.memory.delete')}
                    </button>
                  </td>
                </tr>
                {selected?.id === m.id && (
                  <tr key={`${m.id}-detail`}>
                    <td colSpan={4} style={{ padding: "12px 16px", background: "var(--bg-page)", borderBottom: "1px solid var(--border-default)" }}>
                      <div style={{ color: "var(--text-primary)", fontWeight: 600, marginBottom: "6px" }}>{t('console.memory.detail_content')}</div>
                      <div style={{ color: "var(--text-secondary)", fontSize: "13px", lineHeight: "1.5", marginBottom: "12px", whiteSpace: "pre-wrap" }}>{m.content}</div>
                      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "8px", fontSize: "12px" }}>
                        <div>
                          <span style={{ color: "var(--text-tertiary)" }}>{t('console.memory.detail_source')}: </span>
                          <span style={{ color: "var(--text-primary)" }}>{m.source}</span>
                        </div>
                        <div>
                          <span style={{ color: "var(--text-tertiary)" }}>{t('console.memory.detail_created')}: </span>
                          <span style={{ color: "var(--text-primary)" }}>{new Date(m.created_at).toLocaleString()}</span>
                        </div>
                      </div>
                    </td>
                  </tr>
                )}
              </>
            ))}
          </tbody>
        </table>
      )}
      {deleteTarget && (
        <div style={{
          position: "fixed", top: 0, left: 0, right: 0, bottom: 0,
          background: "rgba(0,0,0,0.6)", zIndex: 1100,
          display: "flex", alignItems: "center", justifyContent: "center",
        }}>
          <div style={{
            background: "var(--bg-surface)", borderRadius: "var(--radius-xl)",
            border: "1px solid var(--border-default)", padding: "24px",
            maxWidth: "400px", width: "90%",
          }}>
            <h3 style={{ color: "var(--text-primary)", margin: "0 0 8px 0" }}>{t('console.memory.delete_confirm_title')}</h3>
            <p style={{ color: "var(--text-tertiary)", fontSize: "14px", margin: "0 0 20px 0" }}>{t('console.memory.delete_confirm_message')}</p>
            <div style={{ display: "flex", gap: "12px", justifyContent: "flex-end" }}>
              <button
                onClick={() => setDeleteTarget(null)}
                style={{ padding: "8px 20px", borderRadius: "6px", border: "1px solid var(--border-default)", background: "transparent", color: "var(--text-tertiary)", cursor: "pointer" }}
              >
                {t('console.memory.cancel')}
              </button>
              <button
                onClick={handleDelete}
                style={{ padding: "8px 20px", borderRadius: "6px", border: "none", background: "#f85149", color: "#fff", cursor: "pointer" }}
              >
                {t('console.memory.delete')}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}