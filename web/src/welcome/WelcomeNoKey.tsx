interface WelcomeNoKeyProps {
  onDismiss: () => void;
  onReady: () => void;
}

export default function WelcomeNoKey({ onDismiss, onReady }: WelcomeNoKeyProps) {
  const handleGotKey = () => {
    localStorage.setItem("morn_welcomed", "true");
    onDismiss();
  };

  const handleNoKey = () => {
    // Set an empty key so WelcomeGuide transitions to "ready" state,
    // which shows the WelcomeReady screen with a "配置 Key" option.
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
        欢迎使用 Morn
      </h1>
      <p className="welcome-desc">
        你需要配置一个 AI 模型才能开始
      </p>
      <div className="welcome-actions">
        <button
          onClick={handleGotKey}
          className="welcome-btn-primary"
        >
          🔑 我有 API Key → 打开设置页
        </button>
        <button
          onClick={handleNoKey}
          className="welcome-btn-secondary"
        >
          ⚡ 先逛逛，稍后配置
        </button>
      </div>
      <p className="welcome-footer">
        或者先逛逛：🏪 Store · 📖 Studio · ⚙️ 设置
      </p>
    </div>
  );
}
