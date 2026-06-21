import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from '../i18n';

interface DailyCallStat {
  date: string;
  count: number;
}

interface DailyTokenStat {
  date: string;
  tokens: number;
  cost: number;
}

interface AgentStat {
  agent_id: string;
  calls: number;
  avg_latency: number;
}

interface ErrorRateStat {
  date: string;
  errors: number;
  total: number;
}

interface LatencyStat {
  date: string;
  avg_latency: number;
}

interface AnalyticsData {
  daily_calls: DailyCallStat[];
  daily_tokens: DailyTokenStat[];
  top_agents: AgentStat[];
  error_rates: ErrorRateStat[];
  avg_latency: LatencyStat[];
  active_users: number;
  total_executions: number;
}

function BarChart({ data, color, labelKey, valueKey, height = 100 }: {
  data: any[];
  color: string;
  labelKey: string;
  valueKey: string;
  height?: number;
}) {
  const max = Math.max(...data.map(d => d[valueKey]), 1);
  return (
    <div style={{ display: "flex", alignItems: "flex-end", gap: "2px", height: `${height}px`, paddingTop: "8px" }}>
      {data.map((d, i) => {
        const pct = (d[valueKey] / max) * 100;
        return (
          <div
            key={i}
            title={`${d[labelKey]}: ${d[valueKey]}`}
            style={{
              flex: 1,
              height: `${Math.max(pct, 2)}%`,
              background: color,
              borderRadius: "2px 2px 0 0",
              minWidth: "4px",
            }}
          />
        );
      })}
    </div>
  );
}

export default function AnalyticsPanel() {
  const { t } = useTranslation();
  const [data, setData] = useState<AnalyticsData | null>(null);
  const [days, setDays] = useState(30);

  useEffect(() => {
    invoke<AnalyticsData>("get_analytics_data", { days }).then(setData).catch(console.error);
  }, [days]);

  if (!data) {
    return (
      <div>
        <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.analytics.title')}</h2>
        <div style={{ color: "#8b949e" }}>Loading...</div>
      </div>
    );
  }

  const totalErrors = data.error_rates.reduce((s, r) => s + r.errors, 0);
  const totalCalls = data.error_rates.reduce((s, r) => s + r.total, 0);
  const errorRate = totalCalls > 0 ? ((totalErrors / totalCalls) * 100).toFixed(1) : "0.0";
  const avgLatencyAll = data.avg_latency.length > 0
    ? (data.avg_latency.reduce((s, r) => s + r.avg_latency, 0) / data.avg_latency.length).toFixed(0)
    : "0";

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.analytics.title')}</h2>
      <div style={{ marginBottom: "16px", display: "flex", gap: "8px", alignItems: "center" }}>
        <span style={{ color: "#8b949e", fontSize: "14px" }}>Period:</span>
        {[7, 14, 30, 90].map(d => (
          <button
            key={d}
            onClick={() => setDays(d)}
            style={{
              padding: "4px 12px",
              borderRadius: "6px",
              border: "1px solid #30363d",
              background: days === d ? "#1f6feb" : "#21262d",
              color: "#e6edf3",
              cursor: "pointer",
              fontSize: "13px",
            }}
          >{d}d</button>
        ))}
      </div>

      <div style={{ display: "grid", gridTemplateColumns: "repeat(4, 1fr)", gap: "12px", marginBottom: "16px" }}>
        <div className="cost-card" style={{ padding: "16px" }}>
          <div style={{ color: "#8b949e", fontSize: "12px" }}>Active Agents</div>
          <div style={{ color: "#e6edf3", fontSize: "28px", fontWeight: 600 }}>{data.active_users}</div>
        </div>
        <div className="cost-card" style={{ padding: "16px" }}>
          <div style={{ color: "#8b949e", fontSize: "12px" }}>Total Executions</div>
          <div style={{ color: "#e6edf3", fontSize: "28px", fontWeight: 600 }}>{data.total_executions}</div>
        </div>
        <div className="cost-card" style={{ padding: "16px" }}>
          <div style={{ color: "#8b949e", fontSize: "12px" }}>Error Rate</div>
          <div style={{ color: "#e6edf3", fontSize: "28px", fontWeight: 600 }}>{errorRate}%</div>
        </div>
        <div className="cost-card" style={{ padding: "16px" }}>
          <div style={{ color: "#8b949e", fontSize: "12px" }}>Avg Latency</div>
          <div style={{ color: "#e6edf3", fontSize: "28px", fontWeight: 600 }}>{avgLatencyAll}ms</div>
        </div>
      </div>

      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "16px" }}>
        <div className="cost-card" style={{ padding: "16px" }}>
          <div style={{ color: "#8b949e", fontSize: "14px", marginBottom: "8px" }}>Daily Calls</div>
          {data.daily_calls.length > 0 ? (
            <BarChart data={data.daily_calls} color="#58a6ff" labelKey="date" valueKey="count" height={120} />
          ) : (
            <div style={{ color: "#484f58", fontSize: "13px", padding: "20px 0", textAlign: "center" }}>No data</div>
          )}
        </div>
        <div className="cost-card" style={{ padding: "16px" }}>
          <div style={{ color: "#8b949e", fontSize: "14px", marginBottom: "8px" }}>Token Consumption</div>
          {data.daily_tokens.length > 0 ? (
            <BarChart data={data.daily_tokens} color="#3fb950" labelKey="date" valueKey="tokens" height={120} />
          ) : (
            <div style={{ color: "#484f58", fontSize: "13px", padding: "20px 0", textAlign: "center" }}>No data</div>
          )}
        </div>
        <div className="cost-card" style={{ padding: "16px" }}>
          <div style={{ color: "#8b949e", fontSize: "14px", marginBottom: "8px" }}>Error Rate Trend</div>
          {data.error_rates.length > 0 ? (
            <BarChart data={data.error_rates} color="#f85149" labelKey="date" valueKey="errors" height={120} />
          ) : (
            <div style={{ color: "#484f58", fontSize: "13px", padding: "20px 0", textAlign: "center" }}>No data</div>
          )}
        </div>
        <div className="cost-card" style={{ padding: "16px" }}>
          <div style={{ color: "#8b949e", fontSize: "14px", marginBottom: "8px" }}>Avg Latency Trend</div>
          {data.avg_latency.length > 0 ? (
            <BarChart data={data.avg_latency} color="#d2a8ff" labelKey="date" valueKey="avg_latency" height={120} />
          ) : (
            <div style={{ color: "#484f58", fontSize: "13px", padding: "20px 0", textAlign: "center" }}>No data</div>
          )}
        </div>
      </div>

      <div className="cost-card" style={{ padding: "16px", marginTop: "16px" }}>
        <div style={{ color: "#8b949e", fontSize: "14px", marginBottom: "8px" }}>Top Agents</div>
        {data.top_agents.length > 0 ? (
          <table style={{ width: "100%", borderCollapse: "collapse", color: "#e6edf3", fontSize: "13px" }}>
            <thead>
              <tr style={{ borderBottom: "1px solid #30363d", textAlign: "left", color: "#8b949e" }}>
                <th style={{ padding: "6px 8px" }}>Agent</th>
                <th style={{ padding: "6px 8px" }}>Calls</th>
                <th style={{ padding: "6px 8px" }}>Avg Latency</th>
              </tr>
            </thead>
            <tbody>
              {data.top_agents.map((a) => (
                <tr key={a.agent_id} style={{ borderBottom: "1px solid #21262d" }}>
                  <td style={{ padding: "6px 8px" }}>{a.agent_id}</td>
                  <td style={{ padding: "6px 8px" }}>{a.calls}</td>
                  <td style={{ padding: "6px 8px" }}>{a.avg_latency.toFixed(0)}ms</td>
                </tr>
              ))}
            </tbody>
          </table>
        ) : (
          <div style={{ color: "#484f58", fontSize: "13px", padding: "20px 0", textAlign: "center" }}>No data</div>
        )}
      </div>
    </div>
  );
}