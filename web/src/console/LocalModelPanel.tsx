import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from '../i18n';

export default function LocalModelPanel() {
  const { t } = useTranslation();
  const [models, setModels] = useState<string[]>([]);
  const [downloadName, setDownloadName] = useState("");

  useEffect(() => {
    invoke<string[]>("list_local_models").then(setModels).catch(console.error);
  }, []);

  const handleDownload = async () => {
    if (!downloadName.trim()) return;
    try {
      const msg = await invoke<string>("download_model", { name: downloadName });
      alert(msg);
      const updated = await invoke<string[]>("list_local_models");
      setModels(updated);
    } catch (e: any) {
      alert(`Error: ${e}`);
    }
  };

  const handleDelete = async (name: string) => {
    try {
      const msg = await invoke<string>("delete_local_model", { name });
      alert(msg);
      const updated = await invoke<string[]>("list_local_models");
      setModels(updated);
    } catch (e: any) {
      alert(`Error: ${e}`);
    }
  };

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.local_models.title')}</h2>
      <div className="cost-card">
        <div style={{ marginBottom: "12px" }}>
          <label style={{ color: "#8b949e", fontSize: "14px", display: "block", marginBottom: "4px" }}>Installed Models</label>
          {models.length === 0 ? (
            <p style={{ color: "#8b949e", fontSize: "14px" }}>No models installed</p>
          ) : (
            <ul style={{ color: "#e6edf3", fontSize: "14px", margin: 0, paddingLeft: "20px" }}>
              {models.map((m, i) => (
                <li key={i} style={{ marginBottom: "6px", display: "flex", alignItems: "center", gap: "8px" }}>
                  <span>{m}</span>
                  <button
                    onClick={() => handleDelete(m)}
                    style={{
                      padding: "4px 12px",
                      background: "#da3633",
                      color: "#fff",
                      border: "none",
                      borderRadius: "4px",
                      fontSize: "12px",
                      cursor: "pointer",
                    }}
                  >
                    Delete
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
        <div style={{ marginBottom: "12px" }}>
          <label style={{ color: "#8b949e", fontSize: "14px", display: "block", marginBottom: "4px" }}>Download Model</label>
          <div style={{ display: "flex", gap: "8px" }}>
            <input
              value={downloadName}
              onChange={e => setDownloadName(e.target.value)}
              placeholder="model-name"
              style={{
                flex: 1,
                padding: "8px 12px",
                background: "#0d1117",
                border: "1px solid #30363d",
                borderRadius: "6px",
                color: "#e6edf3",
                fontSize: "14px",
              }}
            />
            <button
              onClick={handleDownload}
              disabled={!downloadName.trim()}
              style={{
                padding: "8px 20px",
                background: "#238636",
                color: "#fff",
                border: "none",
                borderRadius: "6px",
                fontSize: "14px",
                fontWeight: 600,
                cursor: "pointer",
                opacity: downloadName.trim() ? 1 : 0.5,
              }}
            >
              Download
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}