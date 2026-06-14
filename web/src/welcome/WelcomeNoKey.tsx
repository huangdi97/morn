interface WelcomeNoKeyProps {
  onDismiss: () => void;
}

export default function WelcomeNoKey({ onDismiss }: WelcomeNoKeyProps) {
  const handleGotKey = () => {
    localStorage.setItem("morn_welcomed", "true");
    onDismiss();
  };

  const handleProxy = () => {
    localStorage.setItem("morn_welcomed", "true");
    onDismiss();
  };

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
        欢迎使用 Morn
      </h1>
      <p style={{ color: "#8b949e", fontSize: "16px", margin: "0 0 32px 0" }}>
        你需要配置一个 AI 模型才能开始
      </p>
      <div style={{ display: "flex", gap: "12px", flexWrap: "wrap", justifyContent: "center" }}>
        <button
          onClick={handleGotKey}
          style={{
            background: "#1f6feb", color: "#fff", border: "none",
            borderRadius: "6px", padding: "12px 24px", fontSize: "15px",
            fontWeight: 600, cursor: "pointer",
          }}
        >
          🔑 我有 API Key → 打开设置页
        </button>
        <button
          onClick={handleProxy}
          style={{
            background: "#21262d", color: "#e6edf3", border: "1px solid #30363d",
            borderRadius: "6px", padding: "12px 24px", fontSize: "15px",
            fontWeight: 500, cursor: "pointer",
          }}
        >
          ⚡ 使用内置中转（推荐）
        </button>
      </div>
      <p style={{ color: "#8b949e", fontSize: "13px", marginTop: "24px" }}>
        或者先逛逛：🏪 Store · 📖 Studio · ⚙️ 设置
      </p>
    </div>
  );
}