import { useEffect, useState } from "react";

interface Metrics {
  success_rate: number;
  avg_latency_ms: number;
  p50_latency_ms: number;
  p95_latency_ms: number;
  sla_rate: number;
  total_executions: number;
}

interface Execution {
  id: string;
  agent_id: string;
  task_id: string;
  action: string;
  status: string;
  latency_ms: number | null;
  error_msg: string | null;
  created_at: string;
}

function MetricCard({
  label,
  value,
  unit,
}: {
  label: string;
  value: string;
  unit?: string;
}) {
  return (
    <div
      style={{
        background: "#161b22",
        border: "1px solid #30363d",
        borderRadius: "8px",
        padding: "16px",
        flex: "1 1 200px",
      }}
    >
      <div style={{ color: "#8b949e", fontSize: "13px", marginBottom: "8px" }}>
        {label}
      </div>
      <div style={{ color: "#e6edf3", fontSize: "22px", fontWeight: 700 }}>
        {value}
        {unit && (
          <span style={{ fontSize: "14px", fontWeight: 400, color: "#8b949e", marginLeft: "4px" }}>
            {unit}
          </span>
        )}
      </div>
    </div>
  );
}

export default function ReliabilityPanel() {
  const [metrics, setMetrics] = useState<Metrics | null>(null);
  const [recent, setRecent] = useState<Execution[]>([]);

  useEffect(() => {
    const load = async () => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const data = await invoke<{ metrics: Metrics; recent: Execution[] }>(
          "get_reliability_metrics"
        );
        setMetrics(data.metrics);
        setRecent(data.recent);
      } catch (e) {
        console.error("Failed to load reliability metrics", e);
      }
    };
    load();
  }, []);

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>Reliability Metrics</h2>
      <div
        style={{
          display: "flex",
          flexWrap: "wrap",
          gap: "12px",
          marginBottom: "24px",
        }}
      >
        <MetricCard
          label="Success Rate (24h)"
          value={metrics ? metrics.success_rate.toFixed(1) : "..."}
          unit="%"
        />
        <MetricCard
          label="Avg Latency"
          value={metrics ? metrics.avg_latency_ms.toFixed(0) : "..."}
          unit="ms"
        />
        <MetricCard
          label="P95 Latency"
          value={metrics ? metrics.p95_latency_ms.toFixed(0) : "..."}
          unit="ms"
        />
        <MetricCard
          label="SLA Compliance"
          value={metrics ? metrics.sla_rate.toFixed(1) : "..."}
          unit="%"
        />
      </div>
      <h3 style={{ color: "#e6edf3", marginBottom: "12px", fontSize: "15px" }}>
        Recent Executions
      </h3>
      <div
        style={{
          border: "1px solid #30363d",
          borderRadius: "6px",
          overflow: "hidden",
        }}
      >
        {recent.map((exec) => (
          <div
            key={exec.id}
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              padding: "10px 12px",
              borderBottom: "1px solid #21262d",
              color: "#e6edf3",
              fontSize: "13px",
            }}
          >
            <span style={{ flex: 2 }}>{exec.action}</span>
            <span
              style={{
                flex: 1,
                color: exec.status === "success" ? "#3fb950" : "#f85149",
              }}
            >
              {exec.status}
            </span>
            <span style={{ flex: 1, color: "#8b949e" }}>
              {exec.latency_ms != null ? `${exec.latency_ms}ms` : "-"}
            </span>
            <span style={{ flex: 1, color: "#8b949e", fontSize: "12px" }}>
              {exec.created_at}
            </span>
          </div>
        ))}
        {recent.length === 0 && (
          <div
            style={{
              color: "#8b949e",
              fontSize: "14px",
              padding: "20px",
              textAlign: "center",
            }}
          >
            No executions found
          </div>
        )}
      </div>
    </div>
  );
}
