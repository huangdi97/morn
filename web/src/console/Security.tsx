import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from '../i18n';

interface SecurityStatus {
  constitution_status: string;
  intercepted_count: number;
  dual_llm_status: string;
  audit_log: SecurityLog[];
}

interface SecurityLog {
  timestamp: string;
  event_type: string;
  detail: string;
  severity: string;
}

export default function Security() {
  const { t } = useTranslation();
  const [constitutionStatus, setConstitutionStatus] = useState("ACTIVE");
  const [interceptedCount, setInterceptedCount] = useState("0");
  const [dualLlmStatus, setDualLlmStatus] = useState("ENABLED");
  const [logs, setLogs] = useState<SecurityLog[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    invoke<SecurityStatus>("get_security_status")
      .then((res) => {
        setConstitutionStatus(res.constitution_status);
        setInterceptedCount(String(res.intercepted_count));
        setDualLlmStatus(res.dual_llm_status);
        setLogs(res.audit_log);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  const getSeverityColor = (sev: string) => {
    switch (sev) {
      case "high": return "#f85149";
      case "warning": return "#d29922";
      case "info": return "#58a6ff";
      default: return "#8b949e";
    }
  };

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>Security</h2>

      <div style={{ display: "grid", gridTemplateColumns: "repeat(3, 1fr)", gap: "12px", marginBottom: "16px" }}>
        <div className="sec-card">
          <div style={{ color: "#3fb950", fontSize: "13px" }}>Constitution Status</div>
          <div style={{ fontSize: "20px", fontWeight: "bold", color: "#3fb950", marginTop: "8px" }}>{constitutionStatus}</div>
        </div>
        <div className="sec-card">
          <div style={{ color: "#d29922", fontSize: "13px" }}>Intercepted Today</div>
          <div style={{ fontSize: "20px", fontWeight: "bold", color: "#d29922", marginTop: "8px" }}>{interceptedCount}</div>
        </div>
        <div className="sec-card">
          <div style={{ color: "#58a6ff", fontSize: "13px" }}>Dual-LLM Status</div>
          <div style={{ fontSize: "20px", fontWeight: "bold", color: "#58a6ff", marginTop: "8px" }}>{dualLlmStatus}</div>
        </div>
      </div>

      <div className="sec-card">
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>Audit Log</div>
        {loading ? (
          <div style={{ color: "#8b949e" }}>Loading...</div>
        ) : logs.length === 0 ? (
          <div style={{ color: "#8b949e" }}>{t('console.security.no_data')}</div>
        ) : (
          logs.map((log, i) => (
            <div key={i} style={{ display: "flex", gap: "12px", padding: "8px 0", borderBottom: "1px solid #21262d", fontSize: "13px" }}>
              <div style={{ color: "#8b949e", minWidth: "160px", fontSize: "12px" }}>{log.timestamp.slice(0, 19).replace("T", " ")}</div>
              <div style={{ color: getSeverityColor(log.severity), minWidth: "80px", textTransform: "uppercase", fontSize: "11px" }}>{log.severity}</div>
              <div style={{ color: "#e6edf3", flex: 1 }}>{log.detail}</div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}