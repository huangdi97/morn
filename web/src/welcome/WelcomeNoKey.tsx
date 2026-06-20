import { useTranslation } from "../i18n";

interface WelcomeNoKeyProps {
  onDismiss: () => void;
  onReady: () => void;
}

export default function WelcomeNoKey({ onDismiss, onReady }: WelcomeNoKeyProps) {
  const { t } = useTranslation();

  const handleGotKey = () => {
    localStorage.setItem("morn_welcomed", "true");
    onDismiss();
  };

  const handleNoKey = () => {
    localStorage.setItem("morn_api_config", JSON.stringify({
      mode: "local",
      apiKey: "",
    }));
    onReady();
  };

  return (
    <div className="welcome-page">
      <div className="welcome-emoji">👋</div>
      <h1 className="welcome-title">
        {t('welcome_no_key.title')}
      </h1>
      <p className="welcome-desc">
        {t('welcome_no_key.desc')}
      </p>
      <div className="welcome-actions">
        <button
          onClick={handleGotKey}
          className="welcome-btn-primary"
        >
          🔑 {t('welcome_no_key.got_key')}
        </button>
        <button
          onClick={handleNoKey}
          className="welcome-btn-secondary"
        >
          ⚡ {t('welcome_no_key.browse_first')}
        </button>
      </div>
      <p className="welcome-footer">
        {t('welcome_no_key.footer')}
      </p>
    </div>
  );
}
