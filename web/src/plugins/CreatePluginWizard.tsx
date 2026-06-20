import { useState } from 'react';
import { useTranslation } from '../i18n';

interface PluginEntry {
  name: string;
  version: string;
  description: string;
  author: string | null;
  plugin_type: string;
  status: string;
}

export default function CreatePluginWizard() {
  const { t } = useTranslation();
  const [description, setDescription] = useState('');
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<PluginEntry | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleCreate = async () => {
    if (!description.trim()) return;
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const plugin = await invoke<PluginEntry>('create_plugin_from_spec', {
        description: description.trim(),
      });
      setResult(plugin);
    } catch (e: any) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="create-plugin-wizard" style={{ padding: "16px" }}>
      <hgroup>
        <h2 style={{ color: "#e6edf3", marginBottom: "8px" }}>
          {t('console.plugins.create_title') || 'Create Plugin'}
        </h2>
        <p style={{ color: "#8b949e", marginBottom: "16px", fontSize: "14px" }}>
          {t('console.plugins.create_desc') || 'Describe the plugin you want in natural language'}
        </p>
      </hgroup>
      <textarea
        value={description}
        onChange={(e) => setDescription(e.target.value)}
        placeholder={t('console.plugins.create_placeholder') || 'e.g., A weather plugin that fetches forecast from an API'}
        rows={4}
        style={{
          width: "100%",
          padding: "8px",
          border: "1px solid #30363d",
          borderRadius: "4px",
          background: "#0d1117",
          color: "#e6edf3",
          resize: "vertical",
          boxSizing: "border-box",
        }}
      />
      <button
        onClick={handleCreate}
        disabled={loading || !description.trim()}
        style={{
          marginTop: "8px",
          padding: "8px 16px",
          background: loading ? "#21262d" : "#238636",
          color: "#fff",
          border: "none",
          borderRadius: "4px",
          cursor: loading || !description.trim() ? "not-allowed" : "pointer",
          opacity: loading || !description.trim() ? 0.6 : 1,
        }}
      >
        {loading
          ? (t('console.plugins.generating') || 'Generating...')
          : (t('console.plugins.create_btn') || 'Generate & Install')}
      </button>
      {error && (
        <div style={{ color: "#f85149", marginTop: "8px", fontSize: "13px" }}>
          {error}
        </div>
      )}
      {result && (
        <div style={{
          marginTop: "12px",
          padding: "12px",
          background: "#0f2d14",
          border: "1px solid #238636",
          borderRadius: "4px",
        }}>
          <div style={{ color: "#3fb950", fontWeight: 600, fontSize: "14px", marginBottom: "4px" }}>
            ✅ {result.name} installed
          </div>
          <div style={{ color: "#8b949e", fontSize: "13px" }}>{result.description}</div>
          <div style={{ color: "#8b949e", fontSize: "12px", marginTop: "4px" }}>
            Type: {result.plugin_type} | Status: {result.status}
          </div>
        </div>
      )}
    </div>
  );
}
