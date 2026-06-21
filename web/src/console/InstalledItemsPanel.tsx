import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTranslation } from '../i18n';

interface InstalledItem {
  id: string;
  item_type: string;
  name: string;
  description: string;
  enabled: boolean;
  installed_at: string;
}

export function InstalledItemsPanel() {
  const { t } = useTranslation();
  const [items, setItems] = useState<InstalledItem[]>([]);
  const [loading, setLoading] = useState(true);

  const loadItems = async () => {
    try {
      const data = await invoke<InstalledItem[]>("list_installed_items");
      setItems(data);
    } catch (e) {
      console.error("Failed to load installed items", e);
    } finally {
      setLoading(false);
    }
  };

  const handleToggle = async (id: string) => {
    await invoke("toggle_installed_item", { id });
    loadItems();
  };

  const handleUninstall = async (id: string) => {
    if (!confirm(t('lifecycle.confirm_uninstall'))) return;
    await invoke("uninstall_installed_item", { id });
    loadItems();
  };

  useEffect(() => { loadItems(); }, []);

  const groups = items.reduce((acc, item) => {
    (acc[item.item_type] = acc[item.item_type] || []).push(item);
    return acc;
  }, {} as Record<string, InstalledItem[]>);

  const groupOrder = ["system_plugin", "agent", "team_template", "tool", "skill", "personality", "knowledge"];

  return (
    <div className="installed-items-panel">
      {loading ? <p>{t('common.loading')}</p> : (
        groupOrder.map(type => groups[type]?.length > 0 && (
          <div key={type} className="item-group">
            <h3>{t(`lifecycle.type_${type}`)} ({groups[type].length})</h3>
            {groups[type].map(item => (
              <div key={item.id} className="item-row">
                <div className="item-info">
                  <span className="item-name">{item.name}</span>
                  <span className="item-desc">{item.description}</span>
                </div>
                <div className="item-actions">
                  <button onClick={() => handleToggle(item.id)}>
                    {item.enabled ? t('lifecycle.disable') : t('lifecycle.enable')}
                  </button>
                  {item.item_type !== "system_plugin" && (
                    <button className="danger" onClick={() => handleUninstall(item.id)}>
                      {t('lifecycle.uninstall')}
                    </button>
                  )}
                </div>
              </div>
            ))}
          </div>
        ))
      )}
    </div>
  );
}
