import { useEffect, useState } from "react";
import { useTranslation } from '../i18n';

interface DeviceRecord {
  id: string;
  name: string;
  last_seen: string;
  public_key: string;
}

interface SyncStatus {
  pending_events: number;
  device_id: string;
  server_url: string;
  engine_initialized: boolean;
}

export default function SyncPanel() {
  const { t } = useTranslation();
  const [status, setStatus] = useState<SyncStatus | null>(null);
  const [devices, setDevices] = useState<DeviceRecord[]>([]);
  const [syncing, setSyncing] = useState(false);
  const [serverUrl, setServerUrl] = useState("http://localhost:3000");
  const [lastSyncResult, setLastSyncResult] = useState<string>("");

  const load = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const [statusData, deviceList] = await Promise.all([
        invoke<SyncStatus>("get_sync_status"),
        invoke<DeviceRecord[]>("list_sync_devices"),
      ]);
      setStatus(statusData);
      setDevices(deviceList);
      setServerUrl(statusData.server_url);
    } catch (e) {
      console.error("Failed to load sync status", e);
    }
  };

  useEffect(() => {
    load();
  }, []);

  const handleSync = async () => {
    setSyncing(true);
    setLastSyncResult("");
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const result = await invoke<string>("sync_now");
      setLastSyncResult(result);
      await load();
    } catch (e) {
      console.error("Sync failed", e);
      setLastSyncResult(`Error: ${e}`);
    } finally {
      setSyncing(false);
    }
  };

  const handleSaveServerUrl = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("set_sync_server_url", { url: serverUrl });
    } catch (e) {
      console.error("Failed to save server URL", e);
    }
  };

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>Device Sync</h2>
      <div className="cost-card">
        {status && (
          <div style={{ marginBottom: "16px", color: "#8b949e", fontSize: "14px" }}>
            <div style={{ marginBottom: "8px" }}>
              <span>Device ID: </span>
              <span style={{ color: "#e6edf3", fontFamily: "monospace" }}>
                {status.device_id.slice(0, 8)}...
              </span>
            </div>
            <div style={{ marginBottom: "8px" }}>
              <span>{t('console.sync.pending_events')}: </span>
              <span style={{ color: "#e6edf3", fontWeight: 600 }}>
                {status.pending_events}
              </span>
            </div>
            {lastSyncResult && (
              <div style={{ marginBottom: "8px", color: "#58a6ff", fontSize: "12px" }}>
                {lastSyncResult}
              </div>
            )}
          </div>
        )}

        {devices.length > 0 && (
          <div style={{ marginBottom: "16px" }}>
            <div style={{ color: "#8b949e", fontSize: "14px", marginBottom: "8px" }}>
              Registered Devices
            </div>
            {devices.map((d) => (
              <div
                key={d.id}
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  padding: "4px 0",
                  borderBottom: "1px solid #21262d",
                  fontSize: "13px",
                }}
              >
                <span style={{ color: "#e6edf3" }}>{d.name}</span>
                <span style={{ color: "#8b949e", fontFamily: "monospace", fontSize: "11px" }}>
                  {d.id.slice(0, 8)}...
                </span>
              </div>
            ))}
          </div>
        )}

        <div style={{ marginBottom: "16px" }}>
          <div style={{ color: "#8b949e", fontSize: "14px", marginBottom: "4px" }}>
            Sync Server URL
          </div>
          <div style={{ display: "flex", gap: "8px" }}>
            <input
              type="text"
              value={serverUrl}
              onChange={(e) => setServerUrl(e.target.value)}
              style={{
                flex: 1,
                padding: "6px 10px",
                background: "#0d1117",
                border: "1px solid #30363d",
                borderRadius: "6px",
                color: "#e6edf3",
                fontSize: "13px",
              }}
            />
            <button
              onClick={handleSaveServerUrl}
              style={{
                padding: "6px 12px",
                background: "#21262d",
                color: "#e6edf3",
                border: "1px solid #30363d",
                borderRadius: "6px",
                fontSize: "12px",
                cursor: "pointer",
              }}
            >
              Save
            </button>
          </div>
        </div>

        <button
          onClick={handleSync}
          disabled={syncing}
          style={{
            padding: "10px 24px",
            background: syncing ? "#484f58" : "#238636",
            color: "#fff",
            border: "none",
            borderRadius: "6px",
            fontSize: "14px",
            fontWeight: 600,
            cursor: syncing ? "not-allowed" : "pointer",
          }}
        >
          {syncing ? t('console.sync.syncing') : t('console.sync.sync_now')}
        </button>
      </div>
    </div>
  );
}