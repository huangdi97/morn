import { useTranslation } from "../i18n";

interface WelcomeReadyProps {
  onSend: (text: string) => void;
  onDismiss?: () => void;
}

const EXAMPLES = [
  { emoji: "📄", textKey: "welcome_ready.example_report" },
  { emoji: "💻", textKey: "welcome_ready.example_pc" },
  { emoji: "🔍", textKey: "welcome_ready.example_search" },
  { emoji: "📊", textKey: "welcome_ready.example_analyze" },
];

export default function WelcomeReady({ onSend, onDismiss }: WelcomeReadyProps) {
  const { t } = useTranslation();

  return (
    <div className="welcome-page">
      <div className="welcome-emoji">👋</div>
      <h1 className="welcome-title">
        {t('welcome_ready.title')}
      </h1>
      <p className="welcome-desc">
        {t('welcome_ready.desc')}
      </p>
      <div className="welcome-examples">
        {EXAMPLES.map((ex) => (
          <button
            key={ex.textKey}
            onClick={() => onSend(t(ex.textKey))}
            className="welcome-example-btn"
          >
            <span>{ex.emoji}</span>
            <span>{t(ex.textKey)}</span>
          </button>
        ))}
      </div>
      <div className="welcome-ready-footer">
        <p style={{ color: "var(--text-secondary)", fontSize: "13px", margin: 0 }}>
          {t('welcome_ready.footer')}
        </p>
        {onDismiss && (
          <button
            onClick={onDismiss}
            className="welcome-btn-secondary"
          >
            🔑 {t('welcome_ready.configure_key')}
          </button>
        )}
      </div>
    </div>
  );
}
