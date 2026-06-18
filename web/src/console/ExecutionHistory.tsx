import { useState, useEffect } from "react";
import { useTranslation } from '../i18n';
import { api } from "../api";
import "../styles/execution.css";

interface LogEntry {
  id: string;
  action: string;
  status: string;
  agent_id: string;
  latency_ms: number;
  error_msg: string;
  created_at: string;
}

type Filter = "All" | "Running" | "Completed" | "Failed";

const statusFilter: Record<Filter, (s: string) => boolean> = {
  All: () => true,
  Running: (s) => s === "running",
  Completed: (s) => s === "completed" || s === "success",
  Failed: (s) => s === "failed" || s === "error",
};

const statusDot = (status: string) => {
  switch (status) {
    case "running":
      return "#3b82f6";
    case "completed":
    case "success":
      return "#22c55e";
    case "failed":
    case "error":
      return "#ef4444";
    default:
      return "#6b7280";
  }
};

export default function ExecutionHistory() {
  const { t } = useTranslation();
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [filter, setFilter] = useState<Filter>("All");

  useEffect(() => {
    const fetchLogs = () => {
      api.getRecentLogs().then((data: LogEntry[]) => {
        if (Array.isArray(data)) {
          setLogs(data.slice(0, 20));
        }
      }).catch(() => {});
    };
    fetchLogs();
    const interval = setInterval(fetchLogs, 5000);
    return () => clearInterval(interval);
  }, []);

  const filtered = logs.filter((l) => statusFilter[filter](l.status));

  const filters: Filter[] = ["All", "Running", "Completed", "Failed"];

  return (
    <div className="dashboard-card execution-history-card">
      <div className="execution-history-header">
        <span className="execution-history-title">{t('console.execution_history.title')}</span>
        <div className="execution-history-filters">
          {filters.map((f) => (
            <button
              key={f}
              className={`eh-filter-btn ${filter === f ? "active" : ""}`}
              onClick={() => setFilter(f)}
            >
{t(`console.execution_history.filter_${f.toLowerCase()}`)}
            </button>
          ))}
        </div>
      </div>
      <div className="execution-history-table">
        <div className="eh-row eh-header">
          <div className="eh-cell eh-time">{t('console.execution_history.time')}</div>
          <div className="eh-cell eh-agent">{t('console.execution_history.agent')}</div>
          <div className="eh-cell eh-action">{t('console.execution_history.action')}</div>
          <div className="eh-cell eh-status">{t('console.execution_history.status')}</div>
          <div className="eh-cell eh-dur">{t('console.execution_history.duration')}</div>
        </div>
        {filtered.length === 0 ? (
          <div className="eh-empty">{t('console.execution_history.no_logs')}</div>
        ) : (
          filtered.map((log) => (
            <div key={log.id} className="eh-row">
              <div className="eh-cell eh-time">
                {log.created_at?.slice(11, 19) ?? "--"}
              </div>
              <div className="eh-cell eh-agent">{log.agent_id}</div>
              <div className="eh-cell eh-action">{log.action}</div>
              <div className="eh-cell eh-status">
                <span
                  className="eh-dot"
                  style={{ background: statusDot(log.status) }}
                />
                {log.status}
              </div>
              <div className="eh-cell eh-dur">{log.latency_ms}ms</div>
            </div>
          ))
        )}
      </div>
    </div>
  );
}