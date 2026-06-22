import { useState, useEffect } from "react";
import { useTranslation } from '../i18n';

const FIRST_LAUNCH_KEY = "morn_first_launch";

const milestones = [
  { day: 1, emoji: "✅" },
  { day: 3, emoji: "⬜" },
  { day: 7, emoji: "⬜" },
  { day: 14, emoji: "⬜" },
  { day: 30, emoji: "⬜" },
];

const TIP_COUNT = 13;

function getCurrentDay(): number {
  const stored = localStorage.getItem(FIRST_LAUNCH_KEY);
  if (!stored) return 1;
  const firstLaunch = parseInt(stored, 10);
  if (isNaN(firstLaunch)) return 1;
  const now = Date.now();
  const diffMs = now - firstLaunch;
  const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));
  return Math.min(Math.max(diffDays + 1, 1), 30);
}

export default function UserJourney({ onNavigate }: { onNavigate?: (tab: string) => void }) {
  const [day, setDay] = useState(1);
  const [tipIndex, setTipIndex] = useState(0);

  useEffect(() => {
    const stored = localStorage.getItem(FIRST_LAUNCH_KEY);
    if (!stored) {
      localStorage.setItem(FIRST_LAUNCH_KEY, String(Date.now()));
    }
    setDay(getCurrentDay());
  }, []);

  useEffect(() => {
    const interval = setInterval(() => {
      setTipIndex((prev) => (prev + 1) % TIP_COUNT);
    }, 8000);
    return () => clearInterval(interval);
  }, []);

  const progressPercent = Math.round((day / 30) * 100);
  const currentMilestones = milestones.map((m) => ({
    ...m,
    emoji: day >= m.day ? "✅" : "⬜",
  }));
  const { t } = useTranslation();
  const message =
    [1, 3, 7, 14, 30].includes(day)
      ? t(`console.journey.motivation_${day}`)
      : day > 30
        ? t('console.journey.motivation_30')
        : t('console.journey.motivation_default');

  const actionButtons: { key: string; tab: string }[] = [];
  if (day >= 3) {
    actionButtons.push(
      { key: 'console.journey.action_day3_create_agent', tab: 'studio' },
      { key: 'console.journey.action_day3_team_topology', tab: 'topology' },
    );
  }
  if (day >= 7) {
    actionButtons.push(
      { key: 'console.journey.action_day7_publish', tab: 'hub' },
      { key: 'console.journey.action_day7_bot_store', tab: 'marketplace' },
    );
  }
  if (day >= 14) {
    actionButtons.push(
      { key: 'console.journey.action_day14_manage_workflow', tab: 'workflow' },
      { key: 'console.journey.action_day14_efficiency_report', tab: 'analytics' },
    );
  }
  if (day >= 30) {
    actionButtons.push(
      { key: 'console.journey.action_day30_full_stats', tab: 'analytics' },
      { key: 'console.journey.action_day30_share', tab: 'earnings' },
    );
  }

  return (
    <div>
      <h2 style={{ color: "var(--text-primary)", marginBottom: "16px" }}>{t('console.journey.title')}</h2>

      <div style={{ background: "var(--bg-surface)", borderRadius: "var(--radius-xl)", border: "1px solid var(--border-default)", padding: "24px", marginBottom: "16px" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "8px" }}>
          <span style={{ color: "var(--text-primary)", fontWeight: 600 }}>{t('console.journey.day_of_30', { day })}</span>
          <span style={{ color: "var(--text-tertiary)", fontSize: "13px" }}>{t('console.journey.percent_complete', { percent: progressPercent })}</span>
        </div>
        <div style={{ width: "100%", height: "8px", background: "var(--bg-page)", borderRadius: "4px", overflow: "hidden" }}>
          <div style={{ width: `${progressPercent}%`, height: "100%", background: "var(--accent-brand)", borderRadius: "4px", transition: "width 0.5s ease" }} />
        </div>
      </div>

      <div style={{ background: "var(--bg-surface)", borderRadius: "var(--radius-xl)", border: "1px solid var(--border-default)", padding: "24px", marginBottom: "16px" }}>
        <div style={{ color: "var(--text-primary)", fontWeight: 600, marginBottom: "16px" }}>{t('console.journey.milestones_title')}</div>
        <div style={{ display: "grid", gap: "12px" }}>
          {currentMilestones.map((m) => (
            <div key={m.day} style={{ display: "flex", alignItems: "center", gap: "10px", opacity: day >= m.day ? 1 : 0.5 }}>
              <span style={{ fontSize: "16px" }}>{m.emoji}</span>
              <span style={{ color: "var(--text-primary)", fontSize: "14px" }}>{t(`console.journey.milestone_${m.day}`)}</span>
            </div>
          ))}
        </div>
      </div>

      <div style={{ background: "var(--bg-surface)", borderRadius: "var(--radius-xl)", border: "1px solid var(--border-default)", padding: "24px", marginBottom: "16px" }}>
        <div style={{ color: "var(--text-primary)", fontWeight: 600, marginBottom: "8px" }}>{t('console.journey.motivation_title')}</div>
        <p style={{ color: "var(--text-secondary)", fontSize: "14px", lineHeight: "1.6", margin: 0 }}>{message}</p>
      </div>

      {actionButtons.length > 0 && (
        <div style={{ background: "var(--bg-surface)", borderRadius: "var(--radius-xl)", border: "1px solid var(--border-default)", padding: "24px", marginBottom: "16px" }}>
          <div style={{ display: "flex", flexWrap: "wrap", gap: "10px" }}>
            {actionButtons.map((btn) => (
              <button
                key={btn.key}
                onClick={() => onNavigate?.(btn.tab)}
                style={{
                  padding: "10px 18px",
                  borderRadius: "var(--radius-lg)",
                  border: "1px solid var(--accent-brand)",
                  background: "var(--bg-page)",
                  color: "var(--accent-brand)",
                  cursor: "pointer",
                  fontSize: "13px",
                  fontWeight: 500,
                  transition: "background 0.2s, color 0.2s",
                }}
                onMouseEnter={(e) => { e.currentTarget.style.background = "var(--accent-brand)"; e.currentTarget.style.color = "#fff"; }}
                onMouseLeave={(e) => { e.currentTarget.style.background = "var(--bg-page)"; e.currentTarget.style.color = "var(--accent-brand)"; }}
              >
                {t(btn.key)}
              </button>
            ))}
          </div>
        </div>
      )}

      <div style={{ background: "var(--bg-surface)", borderRadius: "var(--radius-xl)", border: "1px solid var(--border-default)", padding: "24px" }}>
        <div style={{ color: "var(--text-primary)", fontWeight: 600, marginBottom: "8px" }}>{t('console.journey.tips_title')}</div>
        <p style={{ color: "var(--accent-brand)", fontSize: "14px", margin: 0 }}>{t(`console.journey.tip_${tipIndex}`)}</p>
      </div>
    </div>
  );
}
