import { useEffect, useRef, useState } from "react";

export type ApiConfig = {
  mode: "local" | "remote";
  serverUrl: string;
  apiKey: string;
};

const STORAGE_KEY = "morn_api_config";

const defaultConfig: ApiConfig = {
  mode: "local",
  serverUrl: "http://localhost:3000",
  apiKey: "",
};

export function getApiConfig(): ApiConfig {
  try {
    const saved = localStorage.getItem(STORAGE_KEY);
    if (saved) {
      return { ...defaultConfig, ...JSON.parse(saved) };
    }
  } catch {}
  return defaultConfig;
}

function saveApiConfig(config: ApiConfig) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(config));
}

interface SettingsProps {
  onClose: () => void;
}

export function Settings({ onClose }: SettingsProps) {
  const [config, setConfig] = useState<ApiConfig>(getApiConfig);
  const [themes, setThemes] = useState<string[]>([]);
  const [selectedTheme, setSelectedTheme] = useState(localStorage.getItem('morn-theme') || '');
  const styleRef = useRef<HTMLStyleElement | null>(null);
  const [syncTime, setSyncTime] = useState("");
  const [telegramEnabled, setTelegramEnabled] = useState(() => localStorage.getItem('morn_telegram_enabled') === 'true');
  const [telegramToken, setTelegramToken] = useState(() => localStorage.getItem('morn_telegram_token') || '');
  const [notifyAgentComplete, setNotifyAgentComplete] = useState(() => localStorage.getItem('morn_notify_agent_complete') === 'true');
  const [notifySecurityAlert, setNotifySecurityAlert] = useState(() => localStorage.getItem('morn_notify_security_alert') === 'true');
  const [notifyUpdateAvailable, setNotifyUpdateAvailable] = useState(() => localStorage.getItem('morn_notify_update_available') === 'true');
  const [provider, setProvider] = useState(() => localStorage.getItem('morn_model_provider') || 'sensenova');
  const [modelApiKey, setModelApiKey] = useState(() => localStorage.getItem('morn_model_api_key') || '');

  useEffect(() => {
    (async () => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const list = await invoke<string[]>("list_themes");
        setThemes(list);
        const saved = localStorage.getItem('morn-theme');
        if (saved && list.includes(saved)) {
          setSelectedTheme(saved);
          const css = await invoke<string>("apply_theme", { name: saved });
          injectCss(css);
        }
      } catch (e) {
        console.error("Failed to load themes", e);
      }
    })();
  }, []);

  const injectCss = (css: string) => {
    if (styleRef.current) {
      styleRef.current.remove();
    }
    const style = document.createElement("style");
    style.textContent = css;
    document.head.appendChild(style);
    styleRef.current = style;
  };

  const handleThemeChange = async (name: string) => {
    setSelectedTheme(name);
    localStorage.setItem('morn-theme', name);
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const css = await invoke<string>("apply_theme", { name });
      injectCss(css);
    } catch (e) {
      console.error("Failed to apply theme", e);
    }
    window.dispatchEvent(new StorageEvent('storage', {
      key: 'morn-theme',
      newValue: name,
    }));
  };

  const handleSave = () => {
    saveApiConfig(config);
    onClose();
  };

  const handleExport = async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    try {
      const data = await invoke<string>("export_mornpack");
      const blob = new Blob([data], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `mornpack-${new Date().toISOString().slice(0, 10)}.mornpack`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      console.error("Export failed", e);
    }
  };

  const handleImport = async () => {
    const input = document.createElement("input");
    input.type = "file";
    input.accept = ".mornpack,.json";
    input.onchange = async () => {
      const file = input.files?.[0];
      if (!file) return;
      const text = await file.text();
      const { invoke } = await import("@tauri-apps/api/core");
      try {
        const count = await invoke<number>("import_mornpack", { data: text });
        alert(`Imported ${count} agent(s)`);
      } catch (e) {
        console.error("Import failed", e);
        alert("Import failed: " + e);
      }
    };
    input.click();
  };

  const handleSyncNow = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const ts = await invoke<string>("sync_now");
      setSyncTime(ts);
    } catch (e) {
      console.error("Sync failed", e);
    }
  };

  const handleTelegramToggle = (enabled: boolean) => {
    setTelegramEnabled(enabled);
    localStorage.setItem('morn_telegram_enabled', String(enabled));
  };

  const handleTelegramToken = (token: string) => {
    setTelegramToken(token);
    localStorage.setItem('morn_telegram_token', token);
  };

  const handleTestNotification = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke<string>("test_notification");
    } catch (e) {
      console.error("Notification test failed", e);
    }
  };

  const handleSaveModelConfig = () => {
    localStorage.setItem('morn_model_provider', provider);
    localStorage.setItem('morn_model_api_key', modelApiKey);
  };

  const handleNotifyChange = (key: string, value: boolean) => {
    localStorage.setItem(key, String(value));
    switch (key) {
      case 'morn_notify_agent_complete': setNotifyAgentComplete(value); break;
      case 'morn_notify_security_alert': setNotifySecurityAlert(value); break;
      case 'morn_notify_update_available': setNotifyUpdateAvailable(value); break;
    }
  };

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-panel" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>Settings</h2>
          <button className="settings-close" onClick={onClose}>×</button>
        </div>

        <div className="settings-body">
          <div className="settings-section">
            <label className="settings-label">Theme</label>
            <select
              className="settings-select"
              value={selectedTheme}
              onChange={(e) => handleThemeChange(e.target.value)}
            >
              {themes.map((t) => (
                <option key={t} value={t}>{t}</option>
              ))}
            </select>
          </div>
          <label className="settings-label">Mode</label>
          <div className="settings-radio-group">
            <label className="settings-radio">
              <input
                type="radio"
                name="mode"
                checked={config.mode === "local"}
                onChange={() => setConfig({ ...config, mode: "local" })}
              />
              Local Mode
            </label>
            <label className="settings-radio">
              <input
                type="radio"
                name="mode"
                checked={config.mode === "remote"}
                onChange={() => setConfig({ ...config, mode: "remote" })}
              />
              Remote Mode
            </label>
          </div>

          {config.mode === "remote" && (
            <>
              <label className="settings-label">Server URL</label>
              <input
                className="settings-input"
                type="text"
                value={config.serverUrl}
                onChange={(e) => setConfig({ ...config, serverUrl: e.target.value })}
                placeholder="http://localhost:3000"
              />

              <label className="settings-label">API Key</label>
              <input
                className="settings-input"
                type="password"
                value={config.apiKey}
                onChange={(e) => setConfig({ ...config, apiKey: e.target.value })}
                placeholder="Enter your API key"
              />
            </>
          )}
        </div>

        <div className="settings-section settings-backup">
          <label className="settings-label">Backup</label>
          <div className="settings-btn-group">
            <button className="settings-btn" onClick={handleExport}>Export .mornpack</button>
            <button className="settings-btn" onClick={handleImport}>Import .mornpack</button>
          </div>
        </div>

        <div className="settings-section">
          <label className="settings-label">Model Configuration</label>
          <select
            className="settings-select"
            value={provider}
            onChange={(e) => setProvider(e.target.value)}
            style={{ marginBottom: "8px" }}
          >
            <option value="sensenova">sensenova</option>
            <option value="deepseek">deepseek</option>
            <option value="openai">openai</option>
            <option value="local">local</option>
          </select>
          <input
            className="settings-input"
            type="password"
            value={modelApiKey}
            onChange={(e) => setModelApiKey(e.target.value)}
            placeholder="API Key"
            style={{ marginBottom: "8px" }}
          />
          <button className="settings-btn" onClick={handleSaveModelConfig}>
            Save Model Config
          </button>
        </div>

        <div className="settings-section">
          <label className="settings-label">Sync</label>
          <div style={{ fontSize: "14px", color: "var(--text-primary)", marginBottom: "8px" }}>
            Devices
          </div>
          <div style={{ fontSize: "13px", color: "var(--text-secondary)", marginBottom: "12px", paddingLeft: "8px" }}>
            • This device (current)
          </div>
          <button className="settings-btn" onClick={handleSyncNow}>Sync Now</button>
          {syncTime && (
            <div style={{ fontSize: "12px", color: "var(--text-secondary)", marginTop: "6px" }}>
              Synced at {new Date(syncTime).toLocaleTimeString()}
            </div>
          )}
        </div>

        <div className="settings-section">
          <label className="settings-label">Notifications</label>
          <div style={{ display: "flex", flexDirection: "column", gap: "8px", marginTop: "8px" }}>
            <label className="settings-checkbox">
              <input
                type="checkbox"
                checked={telegramEnabled}
                onChange={(e) => handleTelegramToggle(e.target.checked)}
              />
              Telegram
            </label>
            {telegramEnabled && (
              <input
                className="settings-input"
                type="password"
                value={telegramToken}
                onChange={(e) => handleTelegramToken(e.target.value)}
                placeholder="Telegram bot token"
                style={{ marginLeft: "24px", width: "calc(100% - 24px)" }}
              />
            )}
            <button className="settings-btn" onClick={handleTestNotification} style={{ alignSelf: "flex-start" }}>
              Test Notification
            </button>
            <label className="settings-checkbox">
              <input
                type="checkbox"
                checked={notifyAgentComplete}
                onChange={(e) => handleNotifyChange('morn_notify_agent_complete', e.target.checked)}
              />
              Agent Complete
            </label>
            <label className="settings-checkbox">
              <input
                type="checkbox"
                checked={notifySecurityAlert}
                onChange={(e) => handleNotifyChange('morn_notify_security_alert', e.target.checked)}
              />
              Security Alert
            </label>
            <label className="settings-checkbox">
              <input
                type="checkbox"
                checked={notifyUpdateAvailable}
                onChange={(e) => handleNotifyChange('morn_notify_update_available', e.target.checked)}
              />
              Update Available
            </label>
          </div>
        </div>

        <div className="settings-footer">
          <button className="settings-btn settings-btn-primary" onClick={handleSave}>
            Save
          </button>
        </div>
      </div>
    </div>
  );
}