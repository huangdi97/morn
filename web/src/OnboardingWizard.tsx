import { useState } from "react";
import { useTranslation } from './i18n';

interface OnboardingWizardProps {
  onComplete: () => void;
}

const PROVIDERS = ["sensenova", "openai", "ollama"];

export function OnboardingWizard({ onComplete }: OnboardingWizardProps) {
  const { t } = useTranslation();
  const [step, setStep] = useState(0);
  const [provider, setProvider] = useState(() => localStorage.getItem('morn_model_provider') || 'sensenova');
  const [apiKey, setApiKey] = useState(() => localStorage.getItem('morn_model_api_key') || '');
  const [theme, setTheme] = useState(() => localStorage.getItem('morn-theme') || 'cyber');

  const handleProviderChange = (val: string) => {
    setProvider(val);
    localStorage.setItem('morn_model_provider', val);
  };

  const handleApiKeyChange = (val: string) => {
    setApiKey(val);
    localStorage.setItem('morn_model_api_key', val);
  };

  const handleThemeChange = (val: string) => {
    setTheme(val);
    localStorage.setItem('morn-theme', val);
    window.dispatchEvent(new StorageEvent('storage', {
      key: 'morn-theme',
      newValue: val,
    }));
  };

  return (
    <div style={{
      position: "fixed", inset: 0, zIndex: 9999,
      background: "rgba(0,0,0,0.6)", display: "flex",
      alignItems: "center", justifyContent: "center",
    }}>
      <div style={{
        background: "var(--bg-card, #1a1b26)", borderRadius: 16,
        width: 480, maxWidth: "90vw", padding: 32,
        boxShadow: "0 8px 40px rgba(0,0,0,0.4)",
        color: "var(--text-primary, #c9d1d9)",
      }}>
        <div style={{ display: "flex", gap: 8, justifyContent: "center", marginBottom: 24 }}>
          {[0, 1, 2, 3].map(i => (
            <div key={i} style={{
              width: 10, height: 10, borderRadius: "50%",
              background: i === step ? "var(--accent, #58a6ff)" : "var(--border, #30363d)",
              transition: "background 0.2s",
            }} />
          ))}
        </div>

        {step === 0 && (
          <div style={{ textAlign: "center" }}>
            <div style={{ fontSize: 64, marginBottom: 16 }}>🌅</div>
            <h2 style={{ margin: "0 0 8px" }}>{t('onboarding.welcome_title')}</h2>
            <p style={{ color: "var(--text-secondary, #8b949e)", margin: 0 }}>
              {t('onboarding.welcome_desc')}
            </p>
          </div>
        )}

        {step === 1 && (
          <div>
            <h3 style={{ margin: "0 0 4px" }}>{t('onboarding.step_model')}</h3>
            <p style={{ color: "var(--text-secondary, #8b949e)", fontSize: 14, margin: "0 0 16px" }}>
              {t('onboarding.step_model_desc')}
            </p>
            <label style={{ display: "block", fontSize: 13, marginBottom: 6 }}>
              {t('onboarding.provider')}
            </label>
            <select
              value={provider}
              onChange={(e) => handleProviderChange(e.target.value)}
              style={{
                width: "100%", padding: "8px 12px", borderRadius: 8, border: "1px solid var(--border, #30363d)",
                background: "var(--bg-input, #0d1117)", color: "inherit", fontSize: 14, marginBottom: 16,
              }}
            >
              {PROVIDERS.map(p => <option key={p} value={p}>{p}</option>)}
            </select>
            <label style={{ display: "block", fontSize: 13, marginBottom: 6 }}>
              {t('onboarding.api_key')}
            </label>
            <input
              type="password"
              value={apiKey}
              onChange={(e) => handleApiKeyChange(e.target.value)}
              placeholder={t('settings.api_key_placeholder') || "sk-..."}
              style={{
                width: "100%", padding: "8px 12px", borderRadius: 8, border: "1px solid var(--border, #30363d)",
                background: "var(--bg-input, #0d1117)", color: "inherit", fontSize: 14, boxSizing: "border-box",
              }}
            />
          </div>
        )}

        {step === 2 && (
          <div>
            <h3 style={{ margin: "0 0 4px" }}>{t('onboarding.step_theme')}</h3>
            <p style={{ color: "var(--text-secondary, #8b949e)", fontSize: 14, margin: "0 0 16px" }}>
              {t('onboarding.step_theme_desc')}
            </p>
            <select
              value={theme}
              onChange={(e) => handleThemeChange(e.target.value)}
              style={{
                width: "100%", padding: "8px 12px", borderRadius: 8, border: "1px solid var(--border, #30363d)",
                background: "var(--bg-input, #0d1117)", color: "inherit", fontSize: 14,
              }}
            >
              <option value="cyber">cyber</option>
              <option value="dark">dark</option>
              <option value="light">light</option>
              <option value="glass">glass</option>
            </select>
            <p style={{ fontSize: 13, color: "var(--text-secondary, #8b949e)", marginTop: 8 }}>
              {t('settings.theme')}: {theme}
            </p>
          </div>
        )}

        {step === 3 && (
          <div style={{ textAlign: "center" }}>
            <div style={{ fontSize: 48, marginBottom: 12 }}>🚀</div>
            <h2 style={{ margin: "0 0 16px" }}>{t('onboarding.step_done')}</h2>
            <div style={{ display: "flex", flexDirection: "column", gap: 10, textAlign: "left" }}>
              <div style={{ background: "var(--bg-surface, #161b22)", borderRadius: 8, padding: "10px 14px", fontSize: 14 }}>
                💬 {t('onboarding.tip_workbench')}
              </div>
              <div style={{ background: "var(--bg-surface, #161b22)", borderRadius: 8, padding: "10px 14px", fontSize: 14 }}>
                🎨 {t('onboarding.tip_studio')}
              </div>
              <div style={{ background: "var(--bg-surface, #161b22)", borderRadius: 8, padding: "10px 14px", fontSize: 14 }}>
                🏪 {t('onboarding.tip_hub')}
              </div>
            </div>
          </div>
        )}

        <div style={{ display: "flex", justifyContent: "space-between", marginTop: 24 }}>
          <div>
            {step === 0 && <span />}
            {step > 0 && (
              <button
                onClick={() => setStep(s => s - 1)}
                style={{
                  padding: "8px 20px", borderRadius: 8, border: "1px solid var(--border, #30363d)",
                  background: "transparent", color: "inherit", cursor: "pointer", fontSize: 14,
                }}
              >
                {t('onboarding.back')}
              </button>
            )}
            {step === 1 && (
              <button
                onClick={() => { setStep(2); }}
                style={{
                  marginLeft: 8, padding: "8px 20px", borderRadius: 8, border: "none",
                  background: "transparent", color: "var(--text-secondary, #8b949e)", cursor: "pointer", fontSize: 13,
                }}
              >
                {t('onboarding.skip')}
              </button>
            )}
          </div>
          <button
            onClick={() => {
              if (step < 3) setStep(s => s + 1);
              else onComplete();
            }}
            style={{
              padding: "8px 24px", borderRadius: 8, border: "none",
              background: "var(--accent, #58a6ff)", color: "#fff", cursor: "pointer", fontSize: 14, fontWeight: 600,
            }}
          >
            {step < 3 ? t('onboarding.next') : t('onboarding.start')}
          </button>
        </div>
      </div>
    </div>
  );
}
