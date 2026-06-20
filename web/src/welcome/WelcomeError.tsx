import { useTranslation } from "../i18n";

interface WelcomeErrorProps {
  onDismiss: () => void;
}

export default function WelcomeError({ onDismiss }: WelcomeErrorProps) {
  const { t } = useTranslation();

  return (
    <div className="welcome-page">
      <div className="welcome-emoji">⚠️</div>
      <h1 className="welcome-title" style={{ margin: "0 0 24px 0" }}>
        {t('welcome_error.title')}
      </h1>
      <div className="welcome-actions">
        <button
          onClick={onDismiss}
          className="welcome-btn-primary"
        >
          🔑 {t('welcome_error.check_key')}
        </button>
        <button
          onClick={onDismiss}
          className="welcome-btn-secondary"
        >
          🔄 {t('welcome_error.switch_provider')}
        </button>
      </div>
    </div>
  );
}