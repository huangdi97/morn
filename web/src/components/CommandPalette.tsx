import React, { useState, useEffect, useRef, useCallback } from 'react';
import { api } from '../api';

interface CommandItem {
  id: string;
  label: string;
  icon: string;
  category: string;
  action: () => void;
}

interface CommandPaletteProps {
  views: Array<{ key: string; label: string; icon: string }>;
  onNavigate: (view: string) => void;
  onClose: () => void;
}

export const CommandPalette: React.FC<CommandPaletteProps> = ({ views, onNavigate, onClose }) => {
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<CommandItem[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const listRef = useRef<HTMLDivElement>(null);

  const getCommands = useCallback(async (q: string): Promise<CommandItem[]> => {
    const items: CommandItem[] = [];

    const viewCommands = views.map(v => ({
      id: `view-${v.key}`,
      label: v.label,
      icon: v.icon,
      category: '视图',
      action: () => { onNavigate(v.key); onClose(); },
    }));
    items.push(...viewCommands);

    try {
      const agents = await api.listPresetPersonas();
      if (Array.isArray(agents)) {
        for (const agent of agents) {
          items.push({
            id: `agent-${agent.id || agent.name}`,
            label: `查看 Agent: ${agent.name || agent.id}`,
            icon: '🤖',
            category: 'Agent',
            action: () => { onNavigate('console'); onClose(); },
          });
        }
      }
    } catch {}

    try {
      const templates = await api.listPresetPersonas();
      if (Array.isArray(templates)) {
        for (const tmpl of templates) {
          items.push({
            id: `template-${tmpl.id}`,
            label: `模板: ${tmpl.name}`,
            icon: '📋',
            category: '模板',
            action: () => { onNavigate('studio'); onClose(); },
          });
        }
      }
    } catch {}

    items.push(
      { id: 'settings', label: '打开设置', icon: '⚙️', category: '设置', action: () => { onClose(); document.querySelector<HTMLButtonElement>('.settings-btn')?.click(); } },
    );

    if (!q.trim()) return items;
    const lower = q.toLowerCase();
    return items.filter(i =>
      i.label.toLowerCase().includes(lower) ||
      i.category.toLowerCase().includes(lower)
    );
  }, [views, onNavigate, onClose]);

  useEffect(() => {
    getCommands(query).then(setResults);
  }, [query, getCommands]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [results.length]);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex(prev => Math.min(prev + 1, results.length - 1));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex(prev => Math.max(prev - 1, 0));
    } else if (e.key === 'Enter' && results[selectedIndex]) {
      e.preventDefault();
      results[selectedIndex].action();
    } else if (e.key === 'Escape') {
      onClose();
    }
  };

  useEffect(() => {
    const el = listRef.current;
    if (!el) return;
    const selected = el.children[selectedIndex] as HTMLElement;
    if (selected) {
      selected.scrollIntoView({ block: 'nearest' });
    }
  }, [selectedIndex]);

  const grouped = results.reduce<Record<string, CommandItem[]>>((acc, item) => {
    (acc[item.category] = acc[item.category] || []).push(item);
    return acc;
  }, {});

  let idx = 0;

  return (
    <div className="command-palette-overlay" onClick={onClose}>
      <div className="command-palette" onClick={e => e.stopPropagation()}>
        <div className="command-palette-input-wrap">
          <span className="command-palette-search-icon">🔍</span>
          <input
            ref={inputRef}
            type="text"
            value={query}
            onChange={e => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="搜索命令或视图..."
            className="command-palette-input"
          />
        </div>
        {results.length > 0 && (
          <div className="command-palette-results" ref={listRef}>
            {Object.entries(grouped).map(([category, items]) => (
              <div key={category}>
                <div className="command-palette-category">{category}</div>
                {items.map(item => {
                  const currentIndex = idx++;
                  return (
                    <div
                      key={item.id}
                      className={`command-palette-item ${currentIndex === selectedIndex ? 'selected' : ''}`}
                      onClick={item.action}
                      onMouseEnter={() => setSelectedIndex(currentIndex)}
                    >
                      <span className="command-palette-item-icon">{item.icon}</span>
                      <span className="command-palette-item-label">{item.label}</span>
                    </div>
                  );
                })}
              </div>
            ))}
          </div>
        )}
        <div className="command-palette-footer">
          按 ↑↓ 导航 · ↵ 确认 · Esc 关闭
        </div>
      </div>
    </div>
  );
};
