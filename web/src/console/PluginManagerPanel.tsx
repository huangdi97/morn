import { useState, useEffect } from "react";
import { useTranslation } from '../i18n';

interface PluginEntry {
  name: string;
  version: string;
  description: string;
  author: string | null;
  plugin_type: string;
  status: string;
}

const cardStyle: React.CSSProperties = {
  background: "#161b22",
  borderRadius: "8px",
  padding: "16px",
  border: "1px solid #30363d",
};

const statusColor: Record<string, string> = {
  active: "#3fb950",
  loaded: "#d29922",
  discovered: "#8b949e",
  error: "#f85149",
};

const statusLabel: Record<string, string> = {
  active: "console.plugins.status_active",
  loaded: "console.plugins.status_loaded",
  discovered: "console.plugins.status_discovered",
  error: "console.plugins.status_error",
};

export default function PluginManagerPanel() {
  const { t } = useTranslation();
  const [plugins, setPlugins] = useState<PluginEntry[]>([]);
  const [loading, setLoading] = useState(true);
  const [toggling, setToggling] = useState<Record<string, boolean>>({});

  const fetchPlugins = async () => {
    setLoading(true);
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const result = await invoke<PluginEntry[]>("list_plugins");
      setPlugins(result);
    } catch {
      setPlugins([]);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchPlugins();
  }, []);

  const handleToggle = async (name: string, enable: boolean) => {
    setToggling((prev) => ({ ...prev, [name]: true }));
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("toggle_plugin", { name, enable });
      await fetchPlugins();
    } catch {
      await fetchPlugins();
    } finally {
      setToggling((prev) => ({ ...prev, [name]: false }));
    }
  };

  const typeLabel = (pluginType: string) => t(`console.plugins.type_${pluginType}` as any) || pluginType;

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>
        {t('console.plugins.title')}
        <span style={{ color: "#8b949e", fontSize: "14px", fontWeight: "normal", marginLeft: "8px" }}>
          ({plugins.length})
        </span>
      </h2>

      {loading && plugins.length === 0 ? (
        <div style={{ color: "#8b949e" }}>{t('console.plugins.loading')}</div>
      ) : plugins.length === 0 ? (
        <div style={cardStyle}>
          <div style={{ color: "#8b949e", textAlign: "center", padding: "24px" }}>
            {t('console.plugins.no_plugins')}
          </div>
        </div>
      ) : (
        <div style={{ display: "flex", flexDirection: "column", gap: "8px" }}>
          {plugins.map((plugin) => {
            const isActive = plugin.status === "active";
            return (
              <div key={plugin.name} style={cardStyle}>
                <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                  <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
                    <div
                      style={{
                        width: "10px",
                        height: "10px",
                        borderRadius: "50%",
                        background: statusColor[plugin.status] || "#8b949e",
                        flexShrink: 0,
                      }}
                    />
                    <div>
                      <div style={{ color: "#e6edf3", fontWeight: 600, fontSize: "14px" }}>
                        {plugin.name}
                      </div>
                      <div style={{ color: "#8b949e", fontSize: "12px", marginTop: "2px" }}>
                        v{plugin.version} &middot; {typeLabel(plugin.plugin_type)}
                        {plugin.author && <span> &middot; {plugin.author}</span>}
                      </div>
                    </div>
                  </div>
                  <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
                    <span
                      style={{
                        ...badgeStyle,
                        background: `${statusColor[plugin.status] || "#8b949e"}22`,
                        color: statusColor[plugin.status] || "#8b949e",
                        border: `1px solid ${statusColor[plugin.status] || "#8b949e"}44`,
                      }}
                    >
                      {t(statusLabel[plugin.status] || "console.plugins.status_discovered")}
                    </span>
                    <label style={{ position: "relative", display: "inline-block", width: "36px", height: "20px", cursor: toggling[plugin.name] ? "not-allowed" : "pointer", opacity: toggling[plugin.name] ? 0.5 : 1 }}>
                      <input
                        type="checkbox"
                        checked={isActive}
                        disabled={toggling[plugin.name]}
                        onChange={() => handleToggle(plugin.name, !isActive)}
                        style={{ opacity: 0, width: 0, height: 0 }}
                      />
                      <span
                        style={{
                          position: "absolute",
                          inset: 0,
                          background: isActive ? "#3fb950" : "#21262d",
                          borderRadius: "20px",
                          transition: "background 0.2s",
                        }}
                      >
                        <span
                          style={{
                            position: "absolute",
                            top: "2px",
                            left: isActive ? "18px" : "2px",
                            width: "16px",
                            height: "16px",
                            background: "#fff",
                            borderRadius: "50%",
                            transition: "left 0.2s",
                          }}
                        />
                      </span>
                    </label>
                  </div>
                </div>
                {plugin.description && (
                  <div style={{ color: "#8b949e", fontSize: "12px", marginTop: "8px", paddingLeft: "22px" }}>
                    {plugin.description}
                  </div>
                )}
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
}

const badgeStyle: React.CSSProperties = {
  display: "inline-block",
  padding: "2px 10px",
  borderRadius: "12px",
  fontSize: "12px",
  fontWeight: 600,
};
