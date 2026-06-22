import { useState, useEffect } from "react";
import { api, ProviderInfo } from "../api";
import { useTranslation } from "../i18n";

const PROVIDER_ICONS: Record<string, string> = {
  github: "🐙",
  google: "🔵",
  slack: "💬",
  notion: "📝",
};

function getIcon(name: string): string {
  return PROVIDER_ICONS[name] || "🔗";
}

export default function OAuthPanel() {
  const { t } = useTranslation();
  const [providers, setProviders] = useState<ProviderInfo[]>([]);
  const [loading, setLoading] = useState(true);
  const [configuring, setConfiguring] = useState<string | null>(null);
  const [clientId, setClientId] = useState("");
  const [clientSecret, setClientSecret] = useState("");
  const [authUrl, setAuthUrl] = useState("");
  const [callbackCode, setCallbackCode] = useState("");
  const [step, setStep] = useState<"config" | "authorize" | "done">("config");
  const [saving, setSaving] = useState(false);
  const [message, setMessage] = useState("");

  const loadProviders = async () => {
    try {
      const list = await api.oauthListProviders();
      setProviders(list);
    } catch (e) {
      console.error("Failed to load OAuth providers", e);
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    loadProviders();
  }, []);

  const openConfig = (p: ProviderInfo) => {
    setConfiguring(p.name);
    setClientId("");
    setClientSecret("");
    setCallbackCode("");
    setAuthUrl("");
    setStep("config");
    setMessage("");
  };

  const handleSave = async () => {
    if (!configuring) return;
    setSaving(true);
    setMessage("");
    try {
      await api.oauthSaveConfig(configuring, clientId, clientSecret);
      setMessage(t("console.oauth.save_success"));
      const url = await api.oauthAuthorize(configuring);
      setAuthUrl(url);
      setStep("authorize");
    } catch (e: any) {
      setMessage(`Error: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const handleSubmitCode = async () => {
    if (!configuring || !callbackCode.trim()) return;
    setSaving(true);
    setMessage("");
    try {
      await api.oauthCallback(configuring, callbackCode.trim());
      setMessage(t("console.oauth.auth_success"));
      setStep("done");
      loadProviders();
    } catch (e: any) {
      setMessage(`Error: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const handleDisconnect = async (name: string) => {
    try {
      await api.oauthSaveConfig(name, "", "");
      loadProviders();
    } catch (e) {
      console.error("Failed to disconnect", e);
    }
  };

  const closeModal = () => {
    setConfiguring(null);
    setStep("config");
  };

  if (loading) {
    return (
      <div className="oauth-panel" style={{ padding: "24px" }}>
        <h2>{t("console.oauth.title")}</h2>
        <p style={{ color: "var(--text-secondary)" }}>{t("template_selector.loading")}</p>
      </div>
    );
  }

  return (
    <div className="oauth-panel" style={{ padding: "24px" }}>
      <h2>{t("console.oauth.title")}</h2>
      <div style={{ display: "grid", gridTemplateColumns: "repeat(auto-fill, minmax(280px, 1fr))", gap: "16px", marginTop: "16px" }}>
        {providers.map((p) => {
          const icon = getIcon(p.name);
          const configured = p.has_client_id;
          const connected = p.has_token;
          return (
            <div
              key={p.name}
              style={{
                background: "var(--bg-surface)",
                border: "1px solid var(--border-default)",
                borderRadius: "var(--radius-xl)",
                padding: "20px",
                display: "flex",
                flexDirection: "column",
                gap: "12px",
              }}
            >
              <div style={{ display: "flex", alignItems: "center", gap: "12px" }}>
                <span style={{ fontSize: "28px" }}>{icon}</span>
                <span style={{ fontSize: "16px", fontWeight: 600, color: "var(--text-primary)" }}>
                  {p.name.charAt(0).toUpperCase() + p.name.slice(1)}
                </span>
              </div>
              <div style={{ fontSize: "13px", display: "flex", gap: "8px", alignItems: "center" }}>
                {connected ? (
                  <span style={{ color: "var(--color-success, #22c55e)" }}>✅ {t("console.oauth.configured")}</span>
                ) : configured ? (
                  <span style={{ color: "var(--text-secondary)" }}>{t("console.oauth.not_configured")}</span>
                ) : (
                  <span style={{ color: "var(--text-secondary)" }}>{t("console.oauth.not_configured")}</span>
                )}
              </div>
              <div style={{ display: "flex", gap: "8px" }}>
                {connected ? (
                  <button
                    onClick={() => handleDisconnect(p.name)}
                    style={{
                      padding: "8px 16px",
                      border: "1px solid var(--border-default)",
                      borderRadius: "var(--radius-md)",
                      background: "var(--bg-surface)",
                      color: "var(--text-primary)",
                      cursor: "pointer",
                      fontSize: "13px",
                    }}
                  >
                    {t("console.oauth.disconnect")}
                  </button>
                ) : (
                  <button
                    onClick={() => openConfig(p)}
                    style={{
                      padding: "8px 16px",
                      border: "1px solid var(--border-default)",
                      borderRadius: "var(--radius-md)",
                      background: "var(--bg-surface)",
                      color: "var(--text-primary)",
                      cursor: "pointer",
                      fontSize: "13px",
                    }}
                  >
                    {t("console.oauth.configure")}
                  </button>
                )}
              </div>
            </div>
          );
        })}
      </div>

      {configuring && (
        <div
          style={{
            position: "fixed",
            inset: 0,
            background: "rgba(0,0,0,0.5)",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            zIndex: 1000,
          }}
          onClick={closeModal}
        >
          <div
            style={{
              background: "var(--bg-surface)",
              border: "1px solid var(--border-default)",
              borderRadius: "var(--radius-xl)",
              padding: "28px",
              width: "420px",
              maxWidth: "90vw",
              maxHeight: "80vh",
              overflowY: "auto",
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 style={{ margin: "0 0 20px 0", color: "var(--text-primary)" }}>
              {getIcon(configuring)} {configuring.charAt(0).toUpperCase() + configuring.slice(1)}
            </h3>

            {step === "config" && (
              <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
                <label style={{ fontSize: "13px", color: "var(--text-secondary)" }}>
                  {t("console.oauth.client_id")}
                </label>
                <input
                  value={clientId}
                  onChange={(e) => setClientId(e.target.value)}
                  style={{
                    padding: "10px 12px",
                    border: "1px solid var(--border-default)",
                    borderRadius: "var(--radius-md)",
                    background: "var(--bg-input)",
                    color: "var(--text-primary)",
                    fontSize: "14px",
                  }}
                />
                <label style={{ fontSize: "13px", color: "var(--text-secondary)" }}>
                  {t("console.oauth.client_secret")}
                </label>
                <input
                  type="password"
                  value={clientSecret}
                  onChange={(e) => setClientSecret(e.target.value)}
                  style={{
                    padding: "10px 12px",
                    border: "1px solid var(--border-default)",
                    borderRadius: "var(--radius-md)",
                    background: "var(--bg-input)",
                    color: "var(--text-primary)",
                    fontSize: "14px",
                  }}
                />
                <button
                  onClick={handleSave}
                  disabled={saving || !clientId || !clientSecret}
                  style={{
                    padding: "10px 20px",
                    border: "none",
                    borderRadius: "var(--radius-md)",
                    background: saving ? "var(--text-secondary)" : "var(--color-primary, #3b82f6)",
                    color: "#fff",
                    cursor: saving || !clientId || !clientSecret ? "not-allowed" : "pointer",
                    fontSize: "14px",
                    fontWeight: 600,
                    marginTop: "8px",
                  }}
                >
                  {saving ? t("settings.saving") : t("console.oauth.save")}
                </button>
              </div>
            )}

            {step === "authorize" && (
              <div style={{ display: "flex", flexDirection: "column", gap: "12px" }}>
                <p style={{ fontSize: "14px", color: "var(--text-secondary)", lineHeight: 1.5 }}>
                  {t("console.oauth.authorize")}
                </p>
                {authUrl && (
                  <a
                    href={authUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    style={{
                      display: "block",
                      padding: "10px 16px",
                      background: "var(--bg-input)",
                      border: "1px solid var(--border-default)",
                      borderRadius: "var(--radius-md)",
                      color: "var(--color-primary, #3b82f6)",
                      textDecoration: "none",
                      fontSize: "13px",
                      wordBreak: "break-all",
                      textAlign: "center",
                    }}
                  >
                    {t("console.oauth.authorize")} ↗
                  </a>
                )}
                <label style={{ fontSize: "13px", color: "var(--text-secondary)", marginTop: "8px" }}>
                  {t("console.oauth.callback_code")}
                </label>
                <input
                  value={callbackCode}
                  onChange={(e) => setCallbackCode(e.target.value)}
                  placeholder="..."
                  style={{
                    padding: "10px 12px",
                    border: "1px solid var(--border-default)",
                    borderRadius: "var(--radius-md)",
                    background: "var(--bg-input)",
                    color: "var(--text-primary)",
                    fontSize: "14px",
                  }}
                />
                <button
                  onClick={handleSubmitCode}
                  disabled={saving || !callbackCode.trim()}
                  style={{
                    padding: "10px 20px",
                    border: "none",
                    borderRadius: "var(--radius-md)",
                    background: saving ? "var(--text-secondary)" : "var(--color-primary, #3b82f6)",
                    color: "#fff",
                    cursor: saving || !callbackCode.trim() ? "not-allowed" : "pointer",
                    fontSize: "14px",
                    fontWeight: 600,
                    marginTop: "8px",
                  }}
                >
                  {saving ? t("settings.saving") : t("console.oauth.submit_code")}
                </button>
              </div>
            )}

            {step === "done" && (
              <div style={{ textAlign: "center", padding: "16px 0" }}>
                <p style={{ fontSize: "14px", color: "var(--color-success, #22c55e)" }}>{message}</p>
                <button
                  onClick={closeModal}
                  style={{
                    padding: "10px 20px",
                    border: "1px solid var(--border-default)",
                    borderRadius: "var(--radius-md)",
                    background: "var(--bg-surface)",
                    color: "var(--text-primary)",
                    cursor: "pointer",
                    fontSize: "14px",
                    marginTop: "12px",
                  }}
                >
                  Close
                </button>
              </div>
            )}

            {message && step !== "done" && (
              <p style={{ fontSize: "13px", color: message.startsWith("Error") ? "var(--color-error, #ef4444)" : "var(--color-success, #22c55e)", marginTop: "8px" }}>
                {message}
              </p>
            )}
          </div>
        </div>
      )}
    </div>
  );
}