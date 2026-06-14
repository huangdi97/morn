import { t, useLocale } from "../i18n";

interface WelcomeGuideProps {
  onDismiss: () => void;
}

const steps = [
  { icon: "🎭", titleKey: "welcome.step1", desc: "Browse the Bot Store to find pre-built AI agents tailored to your needs — from coding to research." },
  { icon: "📦", titleKey: "welcome.step2", desc: "One-click install any bot. Free agents are ready immediately; paid ones unlock after purchase." },
  { icon: "💬", titleKey: "welcome.step3", desc: "Open the Workbench and begin interacting with your AI. Use quick actions or type anything." },
];

export default function WelcomeGuide({ onDismiss }: WelcomeGuideProps) {
  const [locale, setLocale] = useLocale();

  const handleGetStarted = () => {
    localStorage.setItem("morn_welcomed", "true");
    onDismiss();
  };

  const toggleLocale = () => {
    setLocale(locale === "en" ? "zh" : "en");
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
      <div style={{ position: "absolute", top: "20px", right: "20px" }}>
        <button
          onClick={toggleLocale}
          style={{
            background: "#21262d",
            color: "#e6edf3",
            border: "1px solid #30363d",
            borderRadius: "6px",
            padding: "6px 14px",
            fontSize: "13px",
            cursor: "pointer",
          }}
        >
          {locale === "en" ? "中文" : "English"}
        </button>
      </div>
      <div style={{ textAlign: "center", marginBottom: "48px" }}>
        <div style={{ fontSize: "56px", marginBottom: "16px" }}>🚀</div>
        <h1 style={{ color: "#e6edf3", fontSize: "36px", fontWeight: 700, margin: "0 0 8px 0" }}>
          {t("welcome.title", locale)}
        </h1>
        <p style={{ color: "#8b949e", fontSize: "16px", margin: 0 }}>
          {t("welcome.subtitle", locale)}
        </p>
      </div>

      <div style={{
        display: "flex",
        gap: "16px",
        flexWrap: "wrap",
        justifyContent: "center",
        maxWidth: "800px",
        marginBottom: "40px",
      }}>
        {steps.map((step, i) => (
          <div key={i} style={{
            background: "#161b22",
            border: "1px solid #30363d",
            borderRadius: "8px",
            padding: "24px",
            flex: "1 1 220px",
            minWidth: "200px",
            textAlign: "center",
          }}>
            <div style={{ fontSize: "40px", marginBottom: "12px" }}>{step.icon}</div>
            <div style={{ color: "#e6edf3", fontWeight: 600, fontSize: "15px", marginBottom: "8px" }}>
              {t(step.titleKey, locale)}
            </div>
            <div style={{ color: "#8b949e", fontSize: "13px", lineHeight: 1.5 }}>
              {step.desc}
            </div>
          </div>
        ))}
      </div>

      <button
        onClick={handleGetStarted}
        style={{
          background: "#1f6feb",
          color: "#fff",
          border: "none",
          borderRadius: "6px",
          padding: "12px 36px",
          fontSize: "16px",
          fontWeight: 600,
          cursor: "pointer",
        }}
      >
        {t("welcome.get_started", locale)}
      </button>
    </div>
  );
}