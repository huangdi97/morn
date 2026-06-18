import { useEffect, useState } from "react";
import { useTranslation } from '../i18n';

interface ProactiveRule {
  id: string;
  trigger: string;
  action: string;
  enabled: boolean;
}

export default function ProactivePanel() {
  const { t } = useTranslation();
  const [rules, setRules] = useState<ProactiveRule[]>([]);

  useEffect(() => {
    loadRules();
  }, []);

  const loadRules = async () => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      const list = await invoke<ProactiveRule[]>("list_proactive_rules");
      setRules(list);
    } catch (e) {
      console.error("Failed to load proactive rules", e);
    }
  };

  const handleToggle = async (ruleId: string, current: boolean) => {
    try {
      const { invoke } = await import("@tauri-apps/api/core");
      await invoke("toggle_proactive_rule", { ruleId, enabled: !current });
      setRules((prev) =>
        prev.map((r) => (r.id === ruleId ? { ...r, enabled: !current } : r))
      );
    } catch (e) {
      console.error("Failed to toggle rule", e);
    }
  };

  return (
    <div className="proactive-panel">
      <h2>{t('console.proactive.title')}</h2>
      <div className="proactive-list">
        {rules.map((rule) => (
          <div key={rule.id} className="proactive-item">
            <div className="proactive-info">
              <strong>{rule.id}</strong>
              <span style={{ fontSize: "13px", color: "var(--text-secondary)", marginLeft: "8px" }}>
                {rule.trigger}
              </span>
              <span style={{ fontSize: "12px", color: "var(--text-secondary)", marginLeft: "8px" }}>
                → {rule.action}
              </span>
            </div>
            <button
              className={`proactive-toggle ${rule.enabled ? "active" : ""}`}
              onClick={() => handleToggle(rule.id, rule.enabled)}
            >
              {rule.enabled ? "Enabled" : "Disabled"}
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}