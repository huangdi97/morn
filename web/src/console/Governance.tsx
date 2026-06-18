import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from '../i18n';

interface GovernancePolicy {
  id: string;
  name: string;
  status: string;
}

interface AuditLogEntry {
  id: string;
  action: string;
  actor: string;
  timestamp: string;
  detail: string;
}

export default function Governance() {
  const { t } = useTranslation();
  const [policies, setPolicies] = useState<GovernancePolicy[]>([]);
  const [auditLogs, setAuditLogs] = useState<AuditLogEntry[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    Promise.all([
      invoke<GovernancePolicy[]>("get_governance_policies"),
      invoke<AuditLogEntry[]>("get_audit_log"),
    ])
      .then(([policies, logs]) => {
        setPolicies(policies);
        setAuditLogs(logs);
      })
      .catch(() => {})
      .finally(() => setLoading(false));
  }, []);

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.governance.title')}</h2>

      <div className="gov-card">
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>Governance Policies</div>
        {loading ? (
          <div style={{ color: "#8b949e" }}>Loading...</div>
        ) : policies.length === 0 ? (
          <div style={{ color: "#8b949e" }}>配置你的第一条治理策略</div>
        ) : (
          policies.map(p => (
            <div key={p.id} style={{ display: "flex", justifyContent: "space-between", padding: "8px 0", borderBottom: "1px solid #21262d" }}>
              <div>
                <div style={{ color: "#e6edf3" }}>{p.name}</div>
              </div>
              <div style={{ textAlign: "right" }}>
                <div style={{ color: p.status === "active" ? "#3fb950" : "#d29922" }}>{p.status}</div>
              </div>
            </div>
          ))
        )}
      </div>

      <div className="gov-card">
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>Audit Log</div>
        {loading ? (
          <div style={{ color: "#8b949e" }}>Loading...</div>
        ) : auditLogs.length === 0 ? (
          <div style={{ color: "#8b949e" }}>暂无审计日志</div>
        ) : (
          auditLogs.map(log => (
            <div key={log.id} style={{ padding: "8px 0", borderBottom: "1px solid #21262d" }}>
              <div style={{ color: "#e6edf3", fontSize: "13px" }}>{log.action}</div>
              <div style={{ color: "#8b949e", fontSize: "12px", marginTop: "4px" }}>{log.detail}</div>
              <div style={{ color: "#8b949e", fontSize: "11px", marginTop: "4px" }}>{log.actor} @ {log.timestamp}</div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}