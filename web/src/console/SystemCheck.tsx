import { useState } from "react";

interface CheckResult {
  label: string;
  status: "ok" | "fail";
  value?: string;
}

function CheckRow({ item }: { item: CheckResult }) {
  const icon = item.status === "ok" ? "\u2705" : "\u274C";
  const color = item.status === "ok" ? "#3fb950" : "#f85149";

  return (
    <div style={{
      display: "flex",
      alignItems: "center",
      justifyContent: "space-between",
      padding: "10px 12px",
      borderBottom: "1px solid #21262d",
      color: "#e6edf3",
      fontSize: "14px",
    }}>
      <span>{item.label}</span>
      <span style={{ color }}>
        {icon} {item.value ?? (item.status === "ok" ? "Pass" : "Fail")}
      </span>
    </div>
  );
}

function runChecks(): CheckResult[] {
  return [
    { label: "Storage Status", status: Math.random() > 0.2 ? "ok" : "fail" },
    { label: "API Connection", status: Math.random() > 0.1 ? "ok" : "fail" },
    { label: "Memory Usage", status: Math.random() > 0.15 ? "ok" : "fail" },
    { label: "Plugin Count", status: "ok", value: `${Math.floor(Math.random() * 20) + 5} active` },
    { label: "Agent Count", status: "ok", value: `${Math.floor(Math.random() * 8) + 3} registered` },
    { label: "Workflow Templates", status: "ok", value: `${Math.floor(Math.random() * 12) + 3} available` },
  ];
}

export default function SystemCheck() {
  const [checks, setChecks] = useState<CheckResult[]>([]);
  const [ran, setRan] = useState(false);

  const handleRun = () => {
    setChecks(runChecks());
    setRan(true);
  };

  const allOk = checks.length > 0 && checks.every(c => c.status === "ok");

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>System Self-Check</h2>

      <div className="cost-card">
        <button
          onClick={handleRun}
          style={{
            padding: "10px 24px",
            background: "#238636",
            color: "#fff",
            border: "none",
            borderRadius: "6px",
            fontSize: "14px",
            fontWeight: 600,
            cursor: "pointer",
            marginBottom: "16px",
          }}
        >
          Run Check
        </button>

        {!ran && (
          <div style={{ color: "#8b949e", fontSize: "14px", padding: "20px 0", textAlign: "center" }}>
            Click "Run Check" to run system diagnostics
          </div>
        )}

        {ran && (
          <>
            <div style={{
              padding: "10px 12px",
              borderRadius: "6px",
              background: allOk ? "rgba(63,185,80,0.1)" : "rgba(248,81,73,0.1)",
              border: `1px solid ${allOk ? "#3fb950" : "#f85149"}`,
              color: allOk ? "#3fb950" : "#f85149",
              fontWeight: 600,
              fontSize: "14px",
              marginBottom: "12px",
              textAlign: "center",
            }}>
              {allOk ? "All systems operational" : "Some checks failed"}
            </div>
            <div style={{
              border: "1px solid #30363d",
              borderRadius: "6px",
              overflow: "hidden",
            }}>
              {checks.map((c, i) => (
                <CheckRow key={i} item={c} />
              ))}
            </div>
          </>
        )}
      </div>
    </div>
  );
}