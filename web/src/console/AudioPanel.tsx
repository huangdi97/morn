import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from '../i18n';

export default function AudioPanel() {
  const { t } = useTranslation();
  const [devices, setDevices] = useState<string[]>([]);
  const [path, setPath] = useState("");
  const [result, setResult] = useState("");

  useEffect(() => {
    invoke<string[]>("list_audio_devices").then(setDevices).catch(console.error);
  }, []);

  const handleTranscribe = async () => {
    try {
      const text = await invoke<string>("transcribe_audio", { path });
      setResult(text);
    } catch (e: any) {
      setResult(`Error: ${e}`);
    }
  };

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.audio.title')}</h2>
      <div className="cost-card">
        <div style={{ marginBottom: "12px" }}>
          <label style={{ color: "#8b949e", fontSize: "14px", display: "block", marginBottom: "4px" }}>Audio Devices</label>
          <ul style={{ color: "#e6edf3", fontSize: "14px", margin: 0, paddingLeft: "20px" }}>
            {devices.map((d, i) => <li key={i}>{d}</li>)}
          </ul>
        </div>
        <div style={{ marginBottom: "12px" }}>
          <label style={{ color: "#8b949e", fontSize: "14px", display: "block", marginBottom: "4px" }}>File Path</label>
          <input
            value={path}
            onChange={e => setPath(e.target.value)}
            placeholder="/path/to/audio.wav"
            style={{
              width: "100%",
              padding: "8px 12px",
              background: "#0d1117",
              border: "1px solid #30363d",
              borderRadius: "6px",
              color: "#e6edf3",
              fontSize: "14px",
              boxSizing: "border-box",
            }}
          />
        </div>
        <button
          onClick={handleTranscribe}
          disabled={!path.trim()}
          style={{
            padding: "10px 24px",
            background: "#238636",
            color: "#fff",
            border: "none",
            borderRadius: "6px",
            fontSize: "14px",
            fontWeight: 600,
            cursor: "pointer",
            opacity: path.trim() ? 1 : 0.5,
          }}
        >
          Transcribe
        </button>
        {result && (
          <div style={{ marginTop: "12px", padding: "10px", background: "#161b22", borderRadius: "6px", color: "#e6edf3", fontSize: "14px" }}>
            {result}
          </div>
        )}
      </div>
    </div>
  );
}