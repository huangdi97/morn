import { useState } from "react";
import { useTranslation } from '../i18n';

export default function RoiCalculator() {
  const { t } = useTranslation();
  const [hours, setHours] = useState(160);
  const [rate, setRate] = useState(50);
  const [agents, setAgents] = useState(3);

  const HOURS_SAVED_PER_AGENT = 4;
  const WORKING_DAYS = 22;

  const timeSaved = agents * HOURS_SAVED_PER_AGENT;
  const costSaved = agents * HOURS_SAVED_PER_AGENT * rate * WORKING_DAYS;
  const maxPossible = 10 * HOURS_SAVED_PER_AGENT * rate * WORKING_DAYS;

  const percentage = maxPossible > 0 ? Math.min((costSaved / maxPossible) * 100, 100) : 0;

  const inputStyle: React.CSSProperties = {
    width: "100%",
    padding: "8px 12px",
    background: "#161b22",
    border: "1px solid #30363d",
    borderRadius: "6px",
    color: "#e6edf3",
    fontSize: "14px",
    boxSizing: "border-box",
    marginTop: "4px",
  };

  const labelStyle: React.CSSProperties = {
    color: "#8b949e",
    fontSize: "13px",
    fontWeight: 500,
  };

  return (
    <div>
      <h2 style={{ color: "#e6edf3", marginBottom: "16px" }}>{t('console.roi.title')}</h2>

      <div className="cost-card">
        <div style={{ marginBottom: "16px" }}>
          <label style={labelStyle}>{t('console.roi.operating_hours')}</label>
          <input
            type="number"
            min={1}
            value={hours}
            onChange={e => setHours(Math.max(1, Number(e.target.value)))}
            style={inputStyle}
          />
        </div>
        <div style={{ marginBottom: "16px" }}>
          <label style={labelStyle}>{t('console.roi.hourly_rate')}</label>
          <input
            type="number"
            min={1}
            value={rate}
            onChange={e => setRate(Math.max(1, Number(e.target.value)))}
            style={inputStyle}
          />
        </div>
        <div style={{ marginBottom: "16px" }}>
          <label style={labelStyle}>{t('console.roi.agents_deployed')}</label>
          <input
            type="number"
            min={1}
            max={10}
            value={agents}
            onChange={e => setAgents(Math.max(1, Math.min(10, Number(e.target.value))))}
            style={inputStyle}
          />
        </div>
      </div>

      <div className="cost-card">
        <div style={{ color: "#8b949e", fontSize: "13px", marginBottom: "8px" }}>{t('console.roi.time_saved_per_day')}</div>
        <div style={{ fontSize: "28px", fontWeight: "bold", color: "#58a6ff" }}>{timeSaved} hrs</div>
      </div>

      <div className="cost-card">
        <div style={{ color: "#8b949e", fontSize: "13px", marginBottom: "8px" }}>{t('console.roi.monthly_cost_saved')}</div>
        <div style={{ fontSize: "28px", fontWeight: "bold", color: "#3fb950" }}>
          ¥{costSaved.toLocaleString()}/month
        </div>
        <div style={{ color: "#8b949e", fontSize: "13px", marginTop: "4px" }}>
          {t('console.roi.with_agents', { agents })}
        </div>
      </div>

      <div className="cost-card">
        <div style={{ color: "#e6edf3", fontWeight: "bold", marginBottom: "8px" }}>{t('console.roi.savings_potential')}</div>
        <div style={{
          width: "100%",
          height: "24px",
          background: "#21262d",
          borderRadius: "12px",
          overflow: "hidden",
        }}>
          <div style={{
            width: `${percentage}%`,
            height: "100%",
            background: "linear-gradient(90deg, #3fb950, #58a6ff)",
            borderRadius: "12px",
            transition: "width 0.3s ease",
            display: "flex",
            alignItems: "center",
            justifyContent: "flex-end",
            paddingRight: "8px",
            boxSizing: "border-box",
            minWidth: percentage > 0 ? "24px" : "0",
          }}>
            {percentage > 0 && (
              <span style={{ fontSize: "11px", fontWeight: "bold", color: "#fff" }}>
                {percentage.toFixed(1)}%
              </span>
            )}
          </div>
        </div>
        <div style={{ color: "#8b949e", fontSize: "12px", marginTop: "6px", textAlign: "right" }}>
          {t('console.roi.vs_max')}
        </div>
      </div>
    </div>
  );
}