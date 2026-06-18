import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from '../i18n';

export default function CostPanel() {
  const { t } = useTranslation();
  const [summary, setSummary] = useState("");

  useEffect(() => {
    invoke<string>("get_cost_summary").then(setSummary).catch(console.error);
  }, []);

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.cost_tracking.title')}</h2>
      <div className="cost-card">
        <div style={{ color: "#8b949e", fontSize: "14px", marginBottom: "8px" }}>Summary</div>
        <div style={{ color: "#e6edf3", fontSize: "14px" }}>{summary || "Loading..."}</div>
      </div>
    </div>
  );
}