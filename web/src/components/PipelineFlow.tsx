import { useEffect, useRef, useState } from "react";
import "../styles/pipeline.css";

interface ExecutionEvent {
  id: string;
  action: string;
  status: string;
  agent_id: string;
  latency_ms: number;
  error_msg: string;
  created_at: string;
}

interface PipelineFlowProps {
  logs: ExecutionEvent[];
  visible: boolean;
  onRetry?: (eventId: string) => void;
}

function formatTime(ts: string) {
  if (!ts) return "";
  const d = new Date(ts);
  return d.toLocaleTimeString();
}

function formatLatency(ms: number) {
  if (ms == null) return "";
  return ms >= 1000 ? `${(ms / 1000).toFixed(1)}s` : `${ms}ms`;
}

function statusIcon(status: string) {
  switch (status) {
    case "completed":
    case "success":
      return "\u2705";
    case "failed":
    case "error":
      return "\u274C";
    case "in_progress":
    case "running":
      return "\u23F3";
    default:
      return "\u2B1C";
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

function isRunning(status: string) {
  return status === "in_progress" || status === "running";
}

export default function PipelineFlow({ logs, visible, onRetry }: PipelineFlowProps) {
  const [newIds, setNewIds] = useState<Set<string>>(new Set());
  const [expandedId, setExpandedId] = useState<string | null>(null);
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

  const deduped = new Map<string, ExecutionEvent>();
  for (const log of logs) {
    deduped.set(log.action, log);
  }
  const nodes = Array.from(deduped.values());

  return (
    <div className="pipeline-flow">
      {nodes.map((node, i) => (
        <div key={node.id} className="pipeline-node">
          <div
            className={`pipeline-card${newIds.has(node.id) ? " card-new" : ""}`}
            style={{ borderLeftColor: statusColor(node.status) }}
            onClick={() => setExpandedId(expandedId === node.id ? null : node.id)}
          >
            <div className="pipeline-card-header">
              <span className="pipeline-icon">{statusIcon(node.status)}</span>
              <span className="pipeline-action">{node.action}</span>
              <span className="pipeline-latency" style={{ color: statusColor(node.status) }}>
                {formatLatency(node.latency_ms)}
              </span>
            </div>
            {node.error_msg && (
              <div className="pipeline-error">{node.error_msg}</div>
            )}
            {node.status === "failed" && onRetry && (
              <button
                className="pipeline-retry-btn"
                onClick={(e) => { e.stopPropagation(); onRetry(node.id); }}
              >
                ⟳ 重试
              </button>
            )}
          </div>
          {expandedId === node.id && (
            <div className="pipeline-detail">
              <div className="pipeline-detail-row">
                <span className="pipeline-detail-label">Agent:</span>
                <span>{node.agent_id}</span>
              </div>
              <div className="pipeline-detail-row">
                <span className="pipeline-detail-label">Action:</span>
                <span>{node.action}</span>
              </div>
              <div className="pipeline-detail-row">
                <span className="pipeline-detail-label">Duration:</span>
                <span>{formatLatency(node.latency_ms)}</span>
              </div>
              <div className="pipeline-detail-row">
                <span className="pipeline-detail-label">Time:</span>
                <span>{formatTime(node.created_at)}</span>
              </div>
              {node.error_msg && (
                <div className="pipeline-detail-error">{node.error_msg}</div>
              )}
            </div>
          )}
          {i < nodes.length - 1 && (
            <div className="pipeline-arrow">
              {isRunning(node.status) ? (
                <div className="pipeline-arrow-flow" />
              ) : (
                <div className={`pipeline-arrow-line${node.status === "completed" || node.status === "success" ? " completed" : ""}`} />
              )}
              <div className="pipeline-arrow-head" />
            </div>
          )}
        </div>
      ))}
    </div>
  );
}