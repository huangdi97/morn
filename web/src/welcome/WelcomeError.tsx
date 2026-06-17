interface WelcomeErrorProps {
  onDismiss: () => void;
}

export default function WelcomeError({ onDismiss }: WelcomeErrorProps) {
  return (
    <div className="welcome-page">
      <div className="welcome-emoji">⚠️</div>
      <h1 className="welcome-title" style={{ margin: "0 0 24px 0" }}>
        API 调用失败
      </h1>
      <div className="welcome-actions">
        <button
          onClick={onDismiss}
          className="welcome-btn-primary"
        >
          🔑 检查 API Key → 设置
        </button>
        <button
          onClick={onDismiss}
          className="welcome-btn-secondary"
        >
          🔄 切换模型 Provider
        </button>
      </div>
    </div>
  );
}