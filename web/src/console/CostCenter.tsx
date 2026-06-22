import { useState, useEffect } from "react";
import { useTranslation } from '../i18n';
import { EmptyState } from "../components/EmptyState";

interface CostBreakdown {
  name: string;
  cost: number;
  calls: number;
  percentage: number;
}

interface DailyCost {
  date: string;
  cost: number;
}

interface DailyCostRow {
  date: string;
  agent_id: string;
  model: string;
  token_count: number;
  cost_usd: number;
  call_count: number;
}

interface CostSummary {
  total: number;
  calls: number;
  tokens: number;
  by_date: DailyCostRow[];
}

const tableStyle: React.CSSProperties = {
  width: "100%",
  borderCollapse: "collapse",
  fontSize: "13px",
};

export default function CostCenter() {
  const { t } = useTranslation();
  const [byAgent, setByAgent] = useState<CostBreakdown[]>([]);
  const [daily, setDaily] = useState<DailyCost[]>([]);
  const [totalCost, setTotalCost] = useState("0.00");

  useEffect(() => {
    const load = async () => {
      try {
        const { invoke } = await import("@tauri-apps/api/core");
        const summary = await invoke<CostSummary>("get_cost_summary");
        setTotalCost(summary.total.toFixed(2));

        const details = await invoke<DailyCostRow[]>("get_cost_details", { agentId: null });
        const agentMap = new Map<string, { cost: number; calls: number }>();
        const dateMap = new Map<string, number>();
        for (const row of details) {
          const prev = agentMap.get(row.agent_id) || { cost: 0, calls: 0 };
          agentMap.set(row.agent_id, {
            cost: prev.cost + row.cost_usd,
            calls: prev.calls + row.call_count,
          });
          dateMap.set(row.date, (dateMap.get(row.date) || 0) + row.cost_usd);
        }
        const total = summary.total || 0.01;
        setByAgent(
          Array.from(agentMap.entries()).map(([name, v]) => ({
            name,
            cost: v.cost,
            calls: v.calls,
            percentage: (v.cost / total) * 100,
          }))
        );
        setDaily(
          Array.from(dateMap.entries())
            .map(([date, cost]) => ({ date, cost }))
            .sort((a, b) => a.date.localeCompare(b.date))
        );
      } catch {
        console.error("Failed to load cost data");
      }
    };
    load();
  }, []);

  const maxDaily = Math.max(...daily.map(d => d.cost), 0.01);

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.cost_center.title')}</h2>

      <div className="cost-card">
        <div style={{ color: "#8b949e", fontSize: "13px", marginBottom: "8px" }}>{t('console.cost_center.total_cost')}</div>
        <div style={{ fontSize: "28px", fontWeight: "bold", color: "#f85149" }}>¥{totalCost}</div>
      </div>

      {byAgent.length === 0 && daily.length === 0 ? (
        <EmptyState icon="💰" title="还没有费用数据" description="费用数据将在执行 Agent 任务后自动生成。" />
      ) : (
      <>
      <div className="cost-card">
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>By Agent</div>
        {byAgent.length === 0 ? (
          <div style={{ color: "#8b949e" }}>暂无按 Agent 的费用数据</div>
        ) : (
          <table style={tableStyle}>
            <thead>
              <tr style={{ color: "#8b949e", borderBottom: "1px solid #30363d" }}>
                <td style={{ padding: "6px 8px" }}>Agent</td><td style={{ padding: "6px 8px" }}>Cost</td><td style={{ padding: "6px 8px" }}>Calls</td><td style={{ padding: "6px 8px" }}>%</td>
              </tr>
            </thead>
            <tbody>
              {byAgent.map(a => (
                <tr key={a.name} style={{ borderBottom: "1px solid #21262d", color: "#e6edf3" }}>
                  <td style={{ padding: "6px 8px" }}>{a.name}</td>
                  <td style={{ padding: "6px 8px" }}>¥{a.cost.toFixed(2)}</td>
                  <td style={{ padding: "6px 8px" }}>{a.calls}</td>
                  <td style={{ padding: "6px 8px" }}>{a.percentage.toFixed(1)}%</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      <div className="cost-card">
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>Daily Trend</div>
        {daily.length === 0 ? (
          <div style={{ color: "#8b949e" }}>暂无每日费用数据</div>
        ) : (
          <div style={{ display: "flex", alignItems: "flex-end", gap: "8px", height: "80px" }}>
            {daily.map(d => (
              <div key={d.date} style={{ flex: 1, display: "flex", flexDirection: "column", alignItems: "center" }}>
                <div style={{ height: `${(d.cost / maxDaily) * 70}px`, width: "100%", background: "#58a6ff", borderRadius: "4px 4px 0 0", minHeight: "4px" }} />
                <div style={{ color: "#8b949e", fontSize: "10px", marginTop: "4px" }}>{d.date}</div>
                <div style={{ color: "#e6edf3", fontSize: "10px" }}>¥{d.cost.toFixed(2)}</div>
              </div>
            ))}
          </div>
        )}
      </div>
      </>
      )}
    </div>
  );
}
