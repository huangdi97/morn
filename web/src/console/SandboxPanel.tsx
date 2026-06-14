import { useEffect, useState } from "react";

export default function SandboxPanel() {
  const [code, setCode] = useState("");
  const [language, setLanguage] = useState("python");
  const [result, setResult] = useState("");
  const [status, setStatus] = useState("");

  useEffect(() => {
    (async () => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const s = await invoke<{available: boolean; max_memory_mb: number}>("sandbox_status");
        setStatus(`Sandbox available: ${s.available} | Max memory: ${s.max_memory_mb}MB`);
      } catch (e) {
        setStatus(`Sandbox unavailable: ${e}`);
      }
    })();
  }, []);

  const handleRun = async () => {
    if (!code.trim()) return;
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const res = await invoke<string>("run_in_sandbox", { code, language });
      setResult(res);
    } catch (e) {
      setResult(`Error: ${e}`);
    }
  };

  return (
    <div className="sandbox-panel">
      <h2>WASM Sandbox</h2>
      <p style={{ fontSize: "13px", color: "var(--text-secondary)", marginBottom: "12px" }}>{status}</p>
      <div style={{ marginBottom: "8px" }}>
        <select value={language} onChange={(e) => setLanguage(e.target.value)}>
          <option value="python">Python</option>
          <option value="js">JavaScript</option>
        </select>
      </div>
      <textarea
        value={code}
        onChange={(e) => setCode(e.target.value)}
        placeholder="Enter code..."
        rows={8}
        style={{ width: "100%", fontFamily: "monospace", marginBottom: "8px" }}
      />
      <button onClick={handleRun} disabled={!code.trim()}>Run</button>
      {result && (
        <pre style={{ marginTop: "12px", padding: "8px", background: "var(--bg-secondary)", borderRadius: "4px" }}>
          {result}
        </pre>
      )}
    </div>
  );
}
