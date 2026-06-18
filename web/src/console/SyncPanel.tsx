import { useEffect, useState } from "react";
import { useTranslation } from '../i18n';

export default function SyncPanel() {
  const { t } = useTranslation();
  const [pending, setPending] = useState<number | null>(null);
  const [lastSync, setLastSync] = useState<string>("");
  const [syncing, setSyncing] = useState(false);

  const load = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const raw = await invoke<string>("sync_now");
      const ts = Number(raw);
      if (ts > 0) {
        const d = new Date(ts * 1000);
        setLastSync(d.toLocaleString());
      } else {
        setLastSync("Never");
      }
      setPending(0);
    } catch (e) {
      console.error("Failed to load sync status", e);
    }
  };

  useEffect(() => {
    load();
  }, []);

  const handleSync = async () => {
    setSyncing(true);
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("sync_now");
      setLastSync(new Date().toLocaleString());
      setPending(0);
    } catch (e) {
      console.error("Sync failed", e);
    } finally {
      setSyncing(false);
    }
  };

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>Device Sync</h2>
      <div className="cost-card">
        <div style={{ marginBottom: "16px" }}>
          <div style={{ color: "#8b949e", fontSize: "14px", marginBottom: "4px" }}>
            {t('console.sync.last_sync')}
          </div>
          <div style={{ color: "#e6edf3", fontSize: "16px", fontWeight: 600 }}>
            {lastSync || "Loading..."}
          </div>
        </div>
        <div style={{ marginBottom: "16px" }}>
          <div style={{ color: "#8b949e", fontSize: "14px", marginBottom: "4px" }}>
            {t('console.sync.pending_events')}
          </div>
          <div style={{ color: "#e6edf3", fontSize: "16px", fontWeight: 600 }}>
            {pending !== null ? pending : "..."}
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
