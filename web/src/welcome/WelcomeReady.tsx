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
    <div style={{
      minHeight: "100vh",
      background: "#0d1117",
      display: "flex",
      flexDirection: "column",
      alignItems: "center",
      justifyContent: "center",
      padding: "40px 24px",
      fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif",
    }}>
      <div style={{ fontSize: "56px", marginBottom: "16px" }}>👋</div>
      <h1 style={{ color: "#e6edf3", fontSize: "36px", fontWeight: 700, margin: "0 0 8px 0" }}>
        你好，我是 Morn
      </h1>
      <p style={{ color: "#8b949e", fontSize: "16px", margin: "0 0 32px 0" }}>
        你的桌面 AI 系统已经就绪 ✅
      </p>
      <div style={{ display: "flex", gap: "8px", flexWrap: "wrap", justifyContent: "center", maxWidth: "500px" }}>
        {EXAMPLES.map((ex) => (
          <button
            key={ex.text}
            onClick={() => onSend(ex.text)}
            style={{
              display: "flex", alignItems: "center", gap: "6px",
              padding: "8px 16px", borderRadius: "20px",
              border: "1px solid #30363d", background: "#161b22",
              color: "#e6edf3", cursor: "pointer", fontSize: "14px",
              transition: "all 0.15s ease",
            }}
            onMouseEnter={(e) => { e.currentTarget.style.background = "#1f6feb"; e.currentTarget.style.color = "#fff"; }}
            onMouseLeave={(e) => { e.currentTarget.style.background = "#161b22"; e.currentTarget.style.color = "#e6edf3"; }}
          >
            <span>{ex.emoji}</span>
            <span>{ex.text}</span>
          </button>
        ))}
      </div>
      <div style={{ display: "flex", gap: "12px", marginTop: "32px", alignItems: "center" }}>
        <p style={{ color: "#8b949e", fontSize: "13px", margin: 0 }}>
          或者去 Store 安装预置 Bot
        </p>
        {onDismiss && (
          <button
            onClick={onDismiss}
            style={{
              background: "#21262d", color: "#e6edf3", border: "1px solid #30363d",
              borderRadius: "6px", padding: "8px 16px", fontSize: "13px",
              fontWeight: 500, cursor: "pointer",
            }}
          >
            🔑 配置 API Key
          </button>
        )}
      </div>
    </div>
  );
}
