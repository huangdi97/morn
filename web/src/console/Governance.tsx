import { useState } from "react";

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

const cardStyle: React.CSSProperties = {
  background: "#161b22",
  borderRadius: "8px",
  padding: "16px",
  border: "1px solid #30363d",
  marginBottom: "12px",
};

export default function Governance() {
  const [keys] = useState<ApiKeyInfo[]>([
    { id: "key-1", name: "DeepSeek Production", provider: "deepseek", masked_key: "sk-****-abcd", last_used: "2024-01-15T10:30:00Z" },
  ]);

  const [approvals] = useState<ApprovalItem[]>([
    { id: "app-1", action: "execute_shell: ls /root", requester: "Research Agent", reason: "Need to access config files", requested_at: "2024-01-15T10:30:00Z" },
  ]);

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>Governance</h2>

      <div style={cardStyle}>
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>API Keys</div>
        {keys.map(k => (
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
        ))}
      </div>

      <div style={cardStyle}>
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>Approval Queue</div>
        {approvals.length === 0 ? (
          <div style={{ color: "#8b949e" }}>No pending approvals</div>
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

      <div style={cardStyle}>
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>Trust Threshold</div>
        <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
          <input type="range" min="0" max="100" defaultValue={50} style={{ flex: 1 }} />
          <span style={{ color: "#e6edf3" }}>50%</span>
        </div>
      </div>
    </div>
  );
}