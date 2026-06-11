import { useState } from "react";

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

  const handleSave = () => {
    saveApiConfig(config);
    onClose();
  };

  return (
    <div className="settings-overlay" onClick={onClose}>
      <div className="settings-panel" onClick={(e) => e.stopPropagation()}>
        <div className="settings-header">
          <h2>Settings</h2>
          <button className="settings-close" onClick={onClose}>×</button>
        </div>

        <div className="settings-body">
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

        <div className="settings-footer">
          <button className="settings-btn settings-btn-primary" onClick={handleSave}>
            Save
          </button>
        </div>
      </div>
    </div>
  );
}