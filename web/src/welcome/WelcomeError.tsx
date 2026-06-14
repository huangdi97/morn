interface WelcomeErrorProps {
  onDismiss: () => void;
}

export default function WelcomeError({ onDismiss }: WelcomeErrorProps) {
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
      <div style={{ fontSize: "56px", marginBottom: "16px" }}>⚠️</div>
      <h1 style={{ color: "#e6edf3", fontSize: "36px", fontWeight: 700, margin: "0 0 24px 0" }}>
        API 调用失败
      </h1>
      <div style={{ display: "flex", gap: "12px", flexWrap: "wrap", justifyContent: "center" }}>
        <button
          onClick={onDismiss}
          style={{
            background: "#1f6feb", color: "#fff", border: "none",
            borderRadius: "6px", padding: "12px 24px", fontSize: "15px",
            fontWeight: 600, cursor: "pointer",
          }}
        >
          🔑 检查 API Key → 设置
        </button>
        <button
          onClick={onDismiss}
          style={{
            background: "#21262d", color: "#e6edf3", border: "1px solid #30363d",
            borderRadius: "6px", padding: "12px 24px", fontSize: "15px",
            fontWeight: 500, cursor: "pointer",
          }}
        >
          🔄 切换模型 Provider
        </button>
      </div>
    </div>
  );
}