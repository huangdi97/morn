import { useState, useEffect } from "react";

const FIRST_LAUNCH_KEY = "morn_first_launch";

const milestones = [
  { day: 1, label: "Created your first Agent", emoji: "✅" },
  { day: 3, label: "Built a team", emoji: "⬜" },
  { day: 7, label: "Published to Hub", emoji: "⬜" },
  { day: 14, label: "Automated a workflow", emoji: "⬜" },
  { day: 30, label: "Running a one-person company", emoji: "⬜" },
];

const tips = [
  "Try creating a custom Agent persona",
  "Explore the Bot Store for pre-built agents",
  "Use the Canvas to design a workflow",
  "Publish your Agent to the Hub",
  "Set up a proactive agent with timer trigger",
];

const motivationalMessages: Record<number, string> = {
  1: "Every journey begins with a single step. Great start!",
  3: "Building your team — collaboration is key!",
  7: "Publishing to the Hub opens up new possibilities!",
  14: "Automation is the superpower of efficiency!",
  30: "You're running a one-person company — incredible!",
};

const defaultMessage = "Keep going, every day counts!";

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
      setTipIndex((prev) => (prev + 1) % tips.length);
    }, 8000);
    return () => clearInterval(interval);
  }, []);

  const progressPercent = Math.round((day / 30) * 100);
  const currentMilestones = milestones.map((m) => ({
    ...m,
    emoji: day >= m.day ? "✅" : "⬜",
  }));
  const message = motivationalMessages[day] || (day > 30 ? motivationalMessages[30] : defaultMessage);

  return (
    <div>
      <h2 style={{ color: "var(--text-primary)", marginBottom: "16px" }}>Your Journey</h2>

      <div style={{ background: "var(--bg-surface)", borderRadius: "var(--radius-xl)", border: "1px solid var(--border-default)", padding: "24px", marginBottom: "16px" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "8px" }}>
          <span style={{ color: "var(--text-primary)", fontWeight: 600 }}>Day {day} of 30</span>
          <span style={{ color: "var(--text-tertiary)", fontSize: "13px" }}>{progressPercent}% complete</span>
        </div>
        <div style={{ width: "100%", height: "8px", background: "var(--bg-page)", borderRadius: "4px", overflow: "hidden" }}>
          <div style={{ width: `${progressPercent}%`, height: "100%", background: "var(--accent-brand)", borderRadius: "4px", transition: "width 0.5s ease" }} />
        </div>
      </div>

      <div style={{ background: "var(--bg-surface)", borderRadius: "var(--radius-xl)", border: "1px solid var(--border-default)", padding: "24px", marginBottom: "16px" }}>
        <div style={{ color: "var(--text-primary)", fontWeight: 600, marginBottom: "16px" }}>Milestones</div>
        <div style={{ display: "grid", gap: "12px" }}>
          {currentMilestones.map((m) => (
            <div key={m.day} style={{ display: "flex", alignItems: "center", gap: "10px", opacity: day >= m.day ? 1 : 0.5 }}>
              <span style={{ fontSize: "16px" }}>{m.emoji}</span>
              <span style={{ color: "var(--text-primary)", fontSize: "14px" }}>{m.label}</span>
            </div>
          ))}
        </div>
      </div>

      <div style={{ background: "var(--bg-surface)", borderRadius: "var(--radius-xl)", border: "1px solid var(--border-default)", padding: "24px", marginBottom: "16px" }}>
        <div style={{ color: "var(--text-primary)", fontWeight: 600, marginBottom: "8px" }}>Motivation</div>
        <p style={{ color: "var(--text-secondary)", fontSize: "14px", lineHeight: "1.6", margin: 0 }}>{message}</p>
      </div>

      <div style={{ background: "var(--bg-surface)", borderRadius: "var(--radius-xl)", border: "1px solid var(--border-default)", padding: "24px" }}>
        <div style={{ color: "var(--text-primary)", fontWeight: 600, marginBottom: "8px" }}>Tips for Today</div>
        <p style={{ color: "var(--accent-brand)", fontSize: "14px", margin: 0 }}>{tips[tipIndex]}</p>
      </div>
    </div>
  );
}
