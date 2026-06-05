import { useState } from "react";

interface SecurityLog {
  timestamp: string;
  event_type: string;
  detail: string;
  severity: string;
}

const cardStyle: React.CSSProperties = {
  background: "#161b22",
  borderRadius: "8px",
  padding: "16px",
  border: "1px solid #30363d",
  marginBottom: "12px",
};

export default function Security() {
  const [logs] = useState<SecurityLog[]>([
    { timestamp: "2024-01-15T10:30:00Z", event_type: "auth", detail: "User authenticated successfully", severity: "info" },
    { timestamp: "2024-01-15T10:29:00Z", event_type: "policy_check", detail: "L1 policy enforced: format_disk blocked", severity: "warning" },
    { timestamp: "2024-01-15T10:28:00Z", event_type: "dual_llm", detail: "Injection attempt detected: 'ignore previous instructions'", severity: "high" },
    { timestamp: "2024-01-15T10:27:00Z", event_type: "approval", detail: "execute_shell approved by user", severity: "info" },
  ]);

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
        <div style={cardStyle}>
          <div style={{ color: "#3fb950", fontSize: "13px" }}>Constitution Status</div>
          <div style={{ fontSize: "20px", fontWeight: "bold", color: "#3fb950", marginTop: "8px" }}>ACTIVE</div>
        </div>
        <div style={cardStyle}>
          <div style={{ color: "#d29922", fontSize: "13px" }}>Intercepted Today</div>
          <div style={{ fontSize: "20px", fontWeight: "bold", color: "#d29922", marginTop: "8px" }}>3</div>
        </div>
        <div style={cardStyle}>
          <div style={{ color: "#58a6ff", fontSize: "13px" }}>Dual-LLM Status</div>
          <div style={{ fontSize: "20px", fontWeight: "bold", color: "#58a6ff", marginTop: "8px" }}>ENABLED</div>
        </div>
      </div>

      <div style={cardStyle}>
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>Audit Log</div>
        {logs.map((log, i) => (
          <div key={i} style={{ display: "flex", gap: "12px", padding: "8px 0", borderBottom: "1px solid #21262d", fontSize: "13px" }}>
            <div style={{ color: "#8b949e", minWidth: "160px", fontSize: "12px" }}>{log.timestamp.slice(0, 19).replace("T", " ")}</div>
            <div style={{ color: getSeverityColor(log.severity), minWidth: "80px", textTransform: "uppercase", fontSize: "11px" }}>{log.severity}</div>
            <div style={{ color: "#e6edf3", flex: 1 }}>{log.detail}</div>
          </div>
        ))}
      </div>
    </div>
  );
}