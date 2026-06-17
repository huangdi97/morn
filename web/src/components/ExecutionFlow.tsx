import { useEffect, useRef, useState } from "react";
import "../styles/execution.css";

interface ExecutionEvent {
  id: string;
  action: string;
  status: string;
  agent_id: string;
  latency_ms: number;
  error_msg: string;
  created_at: string;
}

interface ExecutionFlowProps {
  logs: ExecutionEvent[];
  visible: boolean;
}

function formatTime(ts: string) {
  try {
    const d = new Date(ts);
    return d.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit", second: "2-digit" });
  } catch {
    return ts;
  }
}

function statusIcon(status: string) {
  switch (status) {
    case "completed":
    case "success":
      return "\u{1F7E2}";
    case "failed":
    case "error":
      return "\u{1F534}";
    case "in_progress":
    case "running":
      return "\u23F3";
    default:
      return "\u26AA";
  }
}

function statusLabel(status: string) {
  switch (status) {
    case "completed":
    case "success":
      return "Completed";
    case "failed":
    case "error":
      return "Failed";
    case "in_progress":
    case "running":
      return "In Progress";
    default:
      return status;
  }
}

function statusColor(status: string) {
  switch (status) {
    case "completed":
    case "success":
      return "#4ade80";
    case "failed":
    case "error":
      return "#ef4444";
    case "in_progress":
    case "running":
      return "#facc15";
    default:
      return "#9ca3af";
  }
}

export default function ExecutionFlow({ logs, visible }: ExecutionFlowProps) {
  const [newIds, setNewIds] = useState<Set<string>>(new Set());
  const prevLengthRef = useRef(0);

  useEffect(() => {
    if (logs.length > prevLengthRef.current) {
      const added = logs.slice(0, logs.length - prevLengthRef.current).map(l => l.id);
      setNewIds(new Set(added));
      const t = setTimeout(() => setNewIds(new Set()), 500);
      prevLengthRef.current = logs.length;
      return () => clearTimeout(t);
    }
    prevLengthRef.current = logs.length;
  }, [logs]);

  if (!visible || logs.length === 0) return null;

  const display = logs.slice(0, 5);

  return (
    <div className="execution-flow">
      {display.map((log) => (
        <div
          key={log.id}
          className={`execution-entry${newIds.has(log.id) ? " entry-new" : ""}`}
        >
          <span className="execution-time">{formatTime(log.created_at)}</span>
          <span className="execution-agent">{log.agent_id || "system"}</span>
          <span className="execution-action">{log.action}</span>
          <span
            className="execution-status"
            style={{ color: statusColor(log.status) }}
          >
            <span
              className="execution-dot"
              style={{ background: statusColor(log.status) }}
            />
            {statusIcon(log.status)} {statusLabel(log.status)}
          </span>
          {log.latency_ms != null && (
            <span className="execution-duration">
              {log.latency_ms >= 1000
                ? `${(log.latency_ms / 1000).toFixed(1)}s`
                : `${log.latency_ms}ms`}
            </span>
          )}
        </div>
      ))}
    </div>
  );
}