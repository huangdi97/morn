import { useEffect, useState } from "react";

export default function Connections() {
  const [providers, setProviders] = useState<string[]>([]);
  const [connected, setConnected] = useState<Record<string, boolean>>({});

  useEffect(() => {
    (async () => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const list = await invoke<string[]>("oauth_list_providers");
        setProviders(list);
        const saved = localStorage.getItem("oauth_tokens");
        if (saved) {
          const tokens = JSON.parse(saved) as Record<string, string>;
          const conn: Record<string, boolean> = {};
          for (const p of list) {
            conn[p] = !!tokens[p];
          }
          setConnected(conn);
        }
      } catch (e) {
        console.error("Failed to load providers", e);
      }
    })();
  }, []);

  const handleConnect = async (provider: string) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const url = await invoke<string>("oauth_authorize", { provider });
      window.open(url, "_blank");
      const tokens = JSON.parse(localStorage.getItem("oauth_tokens") || "{}");
      tokens[provider] = "pending";
      localStorage.setItem("oauth_tokens", JSON.stringify(tokens));
      setConnected((prev) => ({ ...prev, [provider]: true }));
    } catch (e) {
      console.error(`Failed to connect ${provider}`, e);
    }
  };

  return (
    <div className="connections">
      <h2>Connections</h2>
      <div className="connections-list">
        {providers.map((p) => (
          <div key={p} className="connection-item">
            <span className="connection-name">{p}</span>
            {connected[p] ? (
              <span className="connection-status connected">✅ Connected</span>
            ) : (
              <button
                className="connection-btn"
                onClick={() => handleConnect(p)}
              >
                Connect
              </button>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}
