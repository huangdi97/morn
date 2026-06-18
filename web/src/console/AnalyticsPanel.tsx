import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from '../i18n';

export default function AnalyticsPanel() {
  const { t } = useTranslation();
  const [usage, setUsage] = useState("");
  const [perf, setPerf] = useState("");

  useEffect(() => {
    invoke<string>("get_usage_stats").then(setUsage).catch(console.error);
    invoke<string>("get_performance_metrics").then(setPerf).catch(console.error);
  }, []);

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.analytics.title')}</h2>
      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "16px" }}>
        <div className="cost-card">
          <div style={{ color: "#8b949e", fontSize: "14px", marginBottom: "8px" }}>Usage Stats</div>
          <pre style={{ color: "#e6edf3", fontSize: "14px", margin: 0, whiteSpace: "pre-wrap" }}>
            {usage || "Loading..."}
          </pre>
        </div>
        <div className="cost-card">
          <div style={{ color: "#8b949e", fontSize: "14px", marginBottom: "8px" }}>Performance Metrics</div>
          <pre style={{ color: "#e6edf3", fontSize: "14px", margin: 0, whiteSpace: "pre-wrap" }}>
            {perf || "Loading..."}
          </pre>
        </div>
      </div>
    </div>
  );
}