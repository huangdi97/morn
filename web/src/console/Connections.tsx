import { useEffect, useState } from "react";
import { useTranslation } from '../i18n';
import { EmptyState } from "../components/EmptyState";

export default function Connections() {
  const { t } = useTranslation();
  const [providers, setProviders] = useState<string[]>([]);
  const [connected, setConnected] = useState<Record<string, boolean>>({});
  const [isLoading, setIsLoading] = useState(true);

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
      setIsLoading(false);
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
      <h2>{t('console.connections.title')}</h2>
      {isLoading ? (
        <div className="skeleton-list">
          {[1,2,3].map(i => <div key={i} className="skeleton" />)}
        </div>
      ) : providers.length === 0 ? (
        <EmptyState icon="🔗" title="还没有配置渠道" description="连接第三方服务以扩展 Agent 的能力。" action={{ label: "配置渠道", onClick: () => {} }} />
      ) : (
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
      )}
    </div>
  );
}
