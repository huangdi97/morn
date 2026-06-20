import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from '../i18n';

interface ProactiveRule {
  id: string;
  name: string;
  trigger_type: string;
  trigger_config: string;
  action: string;
  enabled: boolean;
  last_triggered_at: number | null;
  created_at: number;
  updated_at: number;
}

export default function ProactivePanel() {
  const { t } = useTranslation();
  const [rules, setRules] = useState<ProactiveRule[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [name, setName] = useState("");
  const [triggerType, setTriggerType] = useState("timer");
  const [triggerConfig, setTriggerConfig] = useState("60");
  const [action, setAction] = useState("");

  useEffect(() => {
    loadRules();
  }, []);

  const loadRules = async () => {
    try {
      const list = await invoke<ProactiveRule[]>("list_proactive_rules");
      setRules(list);
    } catch (e) {
      console.error("Failed to load proactive rules", e);
    }
  };

  const handleToggle = async (ruleId: string, current: boolean) => {
    try {
      await invoke("toggle_proactive_rule", { ruleId, enabled: !current });
      setRules((prev) =>
        prev.map((r) => (r.id === ruleId ? { ...r, enabled: !current } : r))
      );
    } catch (e) {
      console.error("Failed to toggle rule", e);
    }
  };

  const handleDelete = async (ruleId: string) => {
    try {
      await invoke("delete_proactive_rule", { ruleId });
      setRules((prev) => prev.filter((r) => r.id !== ruleId));
    } catch (e) {
      console.error("Failed to delete rule", e);
    }
  };

  const handleCreate = async () => {
    if (!name.trim() || !triggerConfig.trim()) return;
    try {
      await invoke("create_proactive_rule", {
        name: name.trim(),
        triggerType,
        triggerConfig: triggerConfig.trim(),
        action: action.trim() || name.trim(),
      });
      setName("");
      setTriggerType("timer");
      setTriggerConfig("60");
      setAction("");
      setShowForm(false);
      await loadRules();
    } catch (e) {
      console.error("Failed to create rule", e);
    }
  };

  const formatTrigger = (rule: ProactiveRule) => {
    if (rule.trigger_type === "timer") {
      return `Timer (every ${rule.trigger_config}s)`;
    }
    return `Event (${rule.trigger_config})`;
  };

  const formatTime = (ts: number | null) => {
    if (!ts) return "—";
    return new Date(ts * 1000).toLocaleString();
  };

  return (
    <div className="proactive-panel">
      <div style={{ display: "flex", alignItems: "center", gap: "12px", marginBottom: "16px" }}>
        <h2 style={{ margin: 0 }}>{t('console.proactive.title')}</h2>
        <button className="btn btn-primary" onClick={() => setShowForm(!showForm)}>
          {showForm ? "Cancel" : "New Rule"}
        </button>
      </div>

      {showForm && (
        <div style={{ background: "var(--bg-secondary)", padding: "16px", borderRadius: "8px", marginBottom: "16px" }}>
          <div style={{ display: "flex", flexDirection: "column", gap: "10px" }}>
            <input
              placeholder="Rule name"
              value={name}
              onChange={(e) => setName(e.target.value)}
              style={{ padding: "8px", borderRadius: "4px", border: "1px solid var(--border-color, #ccc)" }}
            />
            <select
              value={triggerType}
              onChange={(e) => setTriggerType(e.target.value)}
              style={{ padding: "8px", borderRadius: "4px", border: "1px solid var(--border-color, #ccc)" }}
            >
              <option value="timer">Timer</option>
              <option value="event">Event</option>
            </select>
            <input
              placeholder={triggerType === "timer" ? "Interval (seconds)" : "Event type name"}
              value={triggerConfig}
              onChange={(e) => setTriggerConfig(e.target.value)}
              style={{ padding: "8px", borderRadius: "4px", border: "1px solid var(--border-color, #ccc)" }}
            />
            <input
              placeholder="Action description"
              value={action}
              onChange={(e) => setAction(e.target.value)}
              style={{ padding: "8px", borderRadius: "4px", border: "1px solid var(--border-color, #ccc)" }}
            />
            <button className="btn btn-primary" onClick={handleCreate} disabled={!name.trim() || !triggerConfig.trim()}>
              Create
            </button>
          </div>
        </div>
      )}

      <div className="proactive-list">
        {rules.length === 0 && (
          <p style={{ color: "var(--text-secondary)", textAlign: "center", padding: "24px" }}>
            No rules defined. Create one to get started.
          </p>
        )}
        {rules.map((rule) => (
          <div key={rule.id} className="proactive-item" style={{
            display: "flex",
            alignItems: "center",
            justifyContent: "space-between",
            padding: "12px",
            borderBottom: "1px solid var(--border-color, #eee)",
          }}>
            <div className="proactive-info" style={{ flex: 1 }}>
              <div>
                <strong>{rule.name}</strong>
                <span style={{ fontSize: "12px", color: "var(--text-secondary)", marginLeft: "8px" }}>
                  {rule.id}
                </span>
              </div>
              <div style={{ fontSize: "13px", color: "var(--text-secondary)", marginTop: "4px" }}>
                {formatTrigger(rule)} → {rule.action}
              </div>
              <div style={{ fontSize: "12px", color: "var(--text-secondary)", marginTop: "2px" }}>
                Last triggered: {formatTime(rule.last_triggered_at)}
              </div>
            </div>
            <div style={{ display: "flex", alignItems: "center", gap: "8px" }}>
              <button
                className={`proactive-toggle ${rule.enabled ? "active" : ""}`}
                onClick={() => handleToggle(rule.id, rule.enabled)}
                style={{
                  padding: "4px 12px",
                  borderRadius: "4px",
                  border: "1px solid var(--border-color, #ccc)",
                  cursor: "pointer",
                  background: rule.enabled ? "var(--accent, #4f8cff)" : "transparent",
                  color: rule.enabled ? "#fff" : "var(--text-primary, #000)",
                }}
              >
                {rule.enabled ? "Enabled" : "Disabled"}
              </button>
              <button
                onClick={() => handleDelete(rule.id)}
                style={{
                  padding: "4px 8px",
                  borderRadius: "4px",
                  border: "1px solid #e74c3c",
                  cursor: "pointer",
                  background: "transparent",
                  color: "#e74c3c",
                  fontSize: "13px",
                }}
                title="Delete rule"
              >
                Del
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
