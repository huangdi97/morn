import { useState } from "react";

const PROVIDERS = ["google", "github", "slack"];

export default function Connections() {
  const [connected, setConnected] = useState<Record<string, boolean>>({});

  const handleConnect = async (provider: string) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const url = await invoke<string>("oauth_authorize", { provider });
      window.open(url, "_blank");
      setConnected((prev) => ({ ...prev, [provider]: true }));
    } catch (e) {
      console.error(`Failed to connect ${provider}`, e);
    }
  };

  return (
    <div className="connections">
      <h2>Connections</h2>
      <div className="connections-list">
        {PROVIDERS.map((p) => (
          <div key={p} className="connection-item">
            <span className="connection-name">{p}</span>
            {connected[p] ? (
              <span className="connection-status connected">Connected</span>
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