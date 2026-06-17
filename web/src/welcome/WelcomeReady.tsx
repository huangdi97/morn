interface WelcomeReadyProps {
  onSend: (text: string) => void;
  onDismiss?: () => void;
}

const EXAMPLES = [
  { emoji: "📄", text: "帮我写一份周报" },
  { emoji: "💻", text: "查一下电脑配置" },
  { emoji: "🔍", text: "搜索 AI Agent 最新消息" },
  { emoji: "📊", text: "分析这组数据" },
];

export default function WelcomeReady({ onSend, onDismiss }: WelcomeReadyProps) {
  return (
    <div className="welcome-page">
      <div className="welcome-emoji">👋</div>
      <h1 className="welcome-title">
        你好，我是 Morn
      </h1>
      <p className="welcome-desc">
        你的桌面 AI 系统已经就绪 ✅
      </p>
      <div className="welcome-examples">
        {EXAMPLES.map((ex) => (
          <button
            key={ex.text}
            onClick={() => onSend(ex.text)}
            className="welcome-example-btn"
          >
            <span>{ex.emoji}</span>
            <span>{ex.text}</span>
          </button>
        ))}
      </div>
      <div className="welcome-ready-footer">
        <p style={{ color: "var(--text-secondary)", fontSize: "13px", margin: 0 }}>
          或者去 Store 安装预置 Bot
        </p>
        {onDismiss && (
          <button
            onClick={onDismiss}
            className="welcome-btn-secondary"
          >
            🔑 配置 API Key
          </button>
        )}
      </div>
    </div>
  );
}
