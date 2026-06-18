import { useState, useEffect } from "react";
import { useTranslation } from '../i18n';

const invoke = (cmd: string, args?: Record<string, unknown>) => {
  if ((window as any).__TAURI__) {
    return (window as any).__TAURI__.invoke(cmd, args);
  }
  return fetch(`/api/${cmd}`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(args || {}),
  }).then((r) => r.json());
};

interface MCPServer {
  name: string;
  url: string;
  tools: { name: string; description: string }[];
}

export function McpManager() {
  const { t } = useTranslation();
  const [servers, setServers] = useState<MCPServer[]>([]);
  const [loading, setLoading] = useState(true);
  const [name, setName] = useState("");
  const [url, setUrl] = useState("");
  const [message, setMessage] = useState("");

  const load = async () => {
    try {
      setLoading(true);
      const result: MCPServer[] = await invoke("mcp_list_servers");
      setServers(result);
    } catch (e) {
      setMessage(`Error: ${e}`);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => { load(); }, []);

  const handleConnect = async () => {
    try {
      setMessage("");
      await invoke("mcp_connect", { name, url });
      setMessage(`Connected to '${name}'`);
      setName("");
      setUrl("");
      await load();
    } catch (e) {
      setMessage(`Error: ${e}`);
    }
  };

  const handleDisconnect = async (n: string) => {
    try {
      await invoke("mcp_disconnect", { name: n });
      setMessage(`Disconnected '${n}'`);
      await load();
    } catch (e) {
      setMessage(`Error: ${e}`);
    }
  };

  return (
    <div className="mcp-manager">
      <h2>{t('studio.mcp.title')}</h2>

      <div style={{ display: "flex", gap: "24px" }}>
        {/* List */}
        <div style={{ flex: 1 }}>
          <h3>{t('studio.mcp.connected', { count: servers.length })}</h3>
          {loading ? (
            <p>{t('studio.mcp.loading')}</p>
          ) : servers.length === 0 ? (
            <p style={{ color: "var(--text-secondary)", fontSize: "13px" }}>No MCP servers connected.</p>
          ) : (
            <table style={{ width: "100%", borderCollapse: "collapse" }}>
              <thead>
                <tr>
                  <th>{t('studio.mcp.name_header')}</th>
                  <th>URL</th>
                  <th>Tools</th>
                  <th></th>
                </tr>
              </thead>
              <tbody>
                {servers.map((s) => (
                  <tr key={s.name}>
                    <td>{s.name}</td>
                    <td style={{ fontSize: "12px" }}>{s.url}</td>
                    <td>{s.tools.length}</td>
                    <td>
                      <button onClick={() => handleDisconnect(s.name)}>Disconnect</button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>

        {/* Connect Form */}
        <div style={{ width: "320px" }}>
          <h3>{t('studio.mcp.connect')}</h3>
          <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
            <input placeholder="Server name" value={name} onChange={(e) => setName(e.target.value)} />
            <input placeholder="URL (e.g. https://mcp.example.com/sse)" value={url} onChange={(e) => setUrl(e.target.value)} />
            <button onClick={handleConnect}>Connect</button>
          </div>
          {message && <p style={{ marginTop: "8px", fontSize: "13px" }}>{message}</p>}
        </div>
      </div>
    </div>
  );
}
