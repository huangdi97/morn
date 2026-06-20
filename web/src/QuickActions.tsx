import { useTranslation } from "./i18n";

interface QuickActionsProps {
  onSend: (text: string) => void;
}

export function QuickActions({ onSend }: QuickActionsProps) {
  const { t } = useTranslation();

  const ACTIONS = [
    { emoji: "📊", labelKey: "quick_actions.analysis", promptKey: "quick_actions.analysis_prompt" },
    { emoji: "📝", labelKey: "quick_actions.writing", promptKey: "quick_actions.writing_prompt" },
    { emoji: "🔍", labelKey: "quick_actions.research", promptKey: "quick_actions.research_prompt" },
    { emoji: "💻", labelKey: "quick_actions.coding", promptKey: "quick_actions.coding_prompt" },
    { emoji: "🌐", labelKey: "quick_actions.translate", promptKey: "quick_actions.translate_prompt" },
  ];

  return (
    <div style={{
      display: "flex", gap: "8px", padding: "8px 16px",
      flexWrap: "wrap", justifyContent: "center",
    }}>
      {ACTIONS.map((action) => (
        <button
          key={action.labelKey}
          onClick={() => onSend(t(action.promptKey))}
          className="quick-action-btn"
        >
          <span>{action.emoji}</span>
          <span>{t(action.labelKey)}</span>
        </button>
      ))}
    </div>
  );
}