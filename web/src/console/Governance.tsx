import { useState, useEffect } from "react";
import { api } from "../api";
import { useTranslation } from '../i18n';

interface ApiKeyInfo {
  id: string;
  name: string;
  provider: string;
  masked_key: string;
  last_used: string;
}

interface ApprovalItem {
  id: string;
  action: string;
  requester: string;
  reason: string;
  requested_at: string;
}

export default function Governance() {
  const { t } = useTranslation();
  const [keys, setKeys] = useState<ApiKeyInfo[]>([]);
  const [approvals, setApprovals] = useState<ApprovalItem[]>([]);
  const [threshold, setThreshold] = useState(50);

  useEffect(() => {
    api.getSystemStatus().then((res: any) => {
      if (res.dashboard?.api_keys) {
        setKeys(res.dashboard.api_keys);
      }
      if (res.dashboard?.approvals) {
        setApprovals(res.dashboard.approvals);
      }
      if (res.dashboard?.trust_threshold !== undefined) {
        setThreshold(res.dashboard.trust_threshold);
      }
    }).catch(() => {});
  }, []);

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.governance.title')}</h2>

      <div className="gov-card">
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>API Keys</div>
        {keys.length === 0 ? (
          <div style={{ color: "#8b949e" }}>{t('console.governance.no_data')}</div>
        ) : (
          keys.map(k => (
            <div key={k.id} style={{ display: "flex", justifyContent: "space-between", padding: "8px 0", borderBottom: "1px solid #21262d" }}>
              <div>
                <div style={{ color: "#e6edf3" }}>{k.name}</div>
                <div style={{ color: "#8b949e", fontSize: "12px" }}>{k.provider}</div>
              </div>
              <div style={{ textAlign: "right" }}>
                <div style={{ color: "#d29922", fontFamily: "monospace" }}>{k.masked_key}</div>
                <div style={{ color: "#8b949e", fontSize: "12px" }}>Last used: {k.last_used.slice(0, 10)}</div>
              </div>
            </div>
          ))
        )}
      </div>

      <div className="gov-card">
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>Approval Queue</div>
        {approvals.length === 0 ? (
          <div style={{ color: "#8b949e" }}>{t('console.governance.no_data')}</div>
        ) : (
          approvals.map(a => (
            <div key={a.id} style={{ padding: "8px 0", borderBottom: "1px solid #21262d" }}>
              <div style={{ color: "#f0883e", fontSize: "12px" }}>{a.action}</div>
              <div style={{ color: "#8b949e", fontSize: "12px", marginTop: "4px" }}>{a.reason}</div>
              <div style={{ display: "flex", gap: "8px", marginTop: "8px" }}>
                <button style={{ background: "#3fb950", color: "#fff", border: "none", padding: "4px 12px", borderRadius: "4px", cursor: "pointer" }}>Approve</button>
                <button style={{ background: "#f85149", color: "#fff", border: "none", padding: "4px 12px", borderRadius: "4px", cursor: "pointer" }}>Reject</button>
              </div>
            </div>
          ))
        )}
      </div>

      <div className="gov-card">
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>Trust Threshold</div>
        <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
          <input type="range" min="0" max="100" value={threshold} onChange={e => setThreshold(Number(e.target.value))} style={{ flex: 1 }} />
          <span style={{ color: "#e6edf3" }}>{threshold}%</span>
        </div>
      </div>
    </div>
  );
}
