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

const TIP_COUNT = 5;

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

export default function UserJourney() {
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

      <div style={{ background: "var(--bg-surface)", borderRadius: "var(--radius-xl)", border: "1px solid var(--border-default)", padding: "24px" }}>
        <div style={{ color: "var(--text-primary)", fontWeight: 600, marginBottom: "8px" }}>{t('console.journey.tips_title')}</div>
        <p style={{ color: "var(--accent-brand)", fontSize: "14px", margin: 0 }}>{t(`console.journey.tip_${tipIndex}`)}</p>
      </div>
    </div>
  );
}
