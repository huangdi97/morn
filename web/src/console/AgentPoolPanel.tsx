import { useState, useEffect, useCallback } from "react";
import { useTranslation } from '../i18n';
import { api } from "../api";

interface AgentTask {
  id: string;
  name: string;
  agent_id: string;
  status: "queued" | "running" | "completed" | "failed";
  progress: number;
  started_at?: string;
  completed_at?: string;
  duration_ms?: number;
  tokens_used?: number;
  cost?: number;
  result?: string;
  error?: string;
}

interface AgentPoolSummary {
  total_agents: number;
  running: number;
  queued: number;
  completed: number;
  failed: number;
  tasks: AgentTask[];
}

const statusColors: Record<string, string> = {
  queued: "#8b949e",
  running: "#3b82f6",
  completed: "#22c55e",
  failed: "#ef4444",
};

const statusIcons: Record<string, string> = {
  queued: "○",
  running: "●",
  completed: "✓",
  failed: "✕",
};

function formatDuration(ms?: number): string {
  if (!ms) return "--";
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${Math.floor(ms / 60000)}m ${Math.floor((ms % 60000) / 1000)}s`;
}

function formatTime(iso?: string): string {
  if (!iso) return "--";
  return iso.slice(11, 19);
}

export default function AgentPoolPanel() {
  const { t } = useTranslation();
  const [poolStatus, setPoolStatus] = useState<AgentPoolSummary | null>(null);
  const [expandedId, setExpandedId] = useState<string | null>(null);
  const [detailCache, setDetailCache] = useState<Record<string, any>>({});
  const [loadingDetail, setLoadingDetail] = useState<string | null>(null);

  const fetchStatus = useCallback(() => {
    api.getAgentPoolStatus().then((data: AgentPoolSummary) => {
      if (data && data.tasks) {
        setPoolStatus(data);
      }
    }).catch(() => {});
  }, []);

  useEffect(() => {
    fetchStatus();
    const interval = setInterval(fetchStatus, 3000);
    return () => clearInterval(interval);
  }, [fetchStatus]);

  const toggleExpand = useCallback(async (taskId: string) => {
    if (expandedId === taskId) {
      setExpandedId(null);
      return;
    }
    setExpandedId(taskId);
    if (!detailCache[taskId]) {
      setLoadingDetail(taskId);
      try {
        const detail = await api.getExecutionDetails(taskId);
        setDetailCache((prev) => ({ ...prev, [taskId]: detail }));
      } catch {
        setDetailCache((prev) => ({ ...prev, [taskId]: null }));
      }
      setLoadingDetail(null);
    }
  }, [expandedId, detailCache]);

  const summary = poolStatus || { total_agents: 0, running: 0, queued: 0, completed: 0, failed: 0, tasks: [] };

  return (
    <div style={{ background: "#161b22", border: "1px solid #30363d", borderRadius: "8px", overflow: "hidden" }}>
      <div style={{ padding: "14px 16px", borderBottom: "1px solid #30363d" }}>
        <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: "10px" }}>
          <h3 style={{ color: "#e6edf3", margin: 0, fontSize: "14px", fontWeight: 600 }}>
            {t('console.agent_pool.title')}
          </h3>
          <span style={{ color: "#8b949e", fontSize: "12px" }}>
            {summary.tasks.length} {t('console.agent_pool.tasks')}
          </span>
        </div>
        <div style={{ display: "flex", gap: "16px", flexWrap: "wrap" }}>
          <StatBadge label={t('console.agent_pool.running')} value={summary.running} color="#3b82f6" />
          <StatBadge label={t('console.agent_pool.queued')} value={summary.queued} color="#8b949e" />
          <StatBadge label={t('console.agent_pool.completed')} value={summary.completed} color="#22c55e" />
          <StatBadge label={t('console.agent_pool.failed')} value={summary.failed} color="#ef4444" />
        </div>
      </div>

      <div style={{ maxHeight: "400px", overflowY: "auto" }}>
        {summary.tasks.length === 0 ? (
          <div style={{ padding: "24px", textAlign: "center", color: "#8b949e", fontSize: "13px" }}>
            {t('console.agent_pool.no_tasks')}
          </div>
        ) : (
          summary.tasks.map((task) => {
            const isExpanded = expandedId === task.id;
            const isDetailLoading = loadingDetail === task.id;
            const detail = detailCache[task.id];

            return (
              <div key={task.id}
                style={{ borderBottom: "1px solid #21262d" }}>
                <div
                  onClick={() => toggleExpand(task.id)}
                  style={{
                    display: "flex",
                    alignItems: "center",
                    gap: "10px",
                    padding: "10px 16px",
                    cursor: "pointer",
                    background: isExpanded ? "#1c2128" : "transparent",
                    transition: "background 0.1s",
                  }}
                >
                  <span style={{ color: statusColors[task.status], fontSize: "14px" }}>{statusIcons[task.status]}</span>
                  <div style={{ flex: 1, minWidth: 0 }}>
                    <div style={{ color: "#e6edf3", fontSize: "13px", fontWeight: 500, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{task.name}</div>
                    <div style={{ color: "#8b949e", fontSize: "11px" }}>{task.agent_id}</div>
                  </div>
                  {task.status === "running" && (
                    <div style={{ width: "60px" }}>
                      <div style={{ height: "4px", background: "#21262d", borderRadius: "2px", overflow: "hidden" }}>
                        <div style={{ height: "100%", width: `${task.progress}%`, background: "#3b82f6", borderRadius: "2px", transition: "width 0.3s" }} />
                      </div>
                    </div>
                  )}
                  <span style={{ color: "#6b7280", fontSize: "11px", minWidth: "50px", textAlign: "right" }}>
                    {task.status === "running" ? "..." : formatDuration(task.duration_ms)}
                  </span>
                  <span style={{ color: "#6b7280", fontSize: "11px", transform: isExpanded ? "rotate(180deg)" : "rotate(0deg)", transition: "transform 0.15s" }}>▾</span>
                </div>

                {isExpanded && (
                  <div style={{ padding: "0 16px 12px 42px", background: "#1c2128" }}>
                    {isDetailLoading ? (
                      <div style={{ color: "#8b949e", fontSize: "12px", padding: "8px 0" }}>Loading...</div>
                    ) : detail ? (
                      <div style={{ display: "flex", flexDirection: "column", gap: "6px", fontSize: "12px", color: "#8b949e" }}>
                        <div style={{ display: "flex", justifyContent: "space-between" }}>
                          <span>{t('console.agent_pool.start_time')}</span>
                          <span style={{ color: "#e6edf3" }}>{formatTime(task.started_at)}</span>
                        </div>
                        <div style={{ display: "flex", justifyContent: "space-between" }}>
                          <span>{t('console.agent_pool.duration')}</span>
                          <span style={{ color: "#e6edf3" }}>{formatDuration(task.duration_ms)}</span>
                        </div>
                        <div style={{ display: "flex", justifyContent: "space-between" }}>
                          <span>{t('console.agent_pool.tokens')}</span>
                          <span style={{ color: "#e6edf3" }}>{task.tokens_used ?? detail.tokens_used ?? "--"}</span>
                        </div>
                        <div style={{ display: "flex", justifyContent: "space-between" }}>
                          <span>{t('console.agent_pool.cost')}</span>
                          <span style={{ color: "#e6edf3" }}>${(task.cost ?? detail.cost ?? 0).toFixed(6)}</span>
                        </div>
                        {detail.result && (
                          <div>
                            <div style={{ color: "#8b949e", marginBottom: "4px" }}>{t('console.agent_pool.result')}</div>
                            <pre style={{ color: "#e6edf3", background: "#0d1117", borderRadius: "4px", padding: "8px", fontSize: "11px", maxHeight: "120px", overflow: "auto", margin: 0, whiteSpace: "pre-wrap", wordBreak: "break-all" }}>
                              {typeof detail.result === "string" ? detail.result : JSON.stringify(detail.result, null, 2)}
                            </pre>
                          </div>
                        )}
                        {detail.error && (
                          <div style={{ color: "#ef4444", background: "#2d1b1b", borderRadius: "4px", padding: "8px", fontSize: "11px" }}>
                            {detail.error}
                          </div>
                        )}
                      </div>
                    ) : (
                      <div style={{ color: "#8b949e", fontSize: "11px", padding: "4px 0", display: "flex", flexDirection: "column", gap: "4px" }}>
                        <span>{t('console.agent_pool.start_time')}: {formatTime(task.started_at)}</span>
                        <span>{t('console.agent_pool.duration')}: {formatDuration(task.duration_ms)}</span>
                        {task.tokens_used != null && <span>Tokens: {task.tokens_used}</span>}
                      </div>
                    )}
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}

function StatBadge({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <div style={{ display: "flex", alignItems: "center", gap: "6px" }}>
      <span style={{ width: "8px", height: "8px", borderRadius: "50%", background: color, display: "inline-block" }} />
      <span style={{ color: "#8b949e", fontSize: "12px" }}>{label}</span>
      <span style={{ color: "#e6edf3", fontSize: "13px", fontWeight: 600 }}>{value}</span>
    </div>
  );
}