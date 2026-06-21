import { useState, useEffect, useCallback } from "react";
import { useTranslation } from './i18n';

interface GuideStep {
  target: string;
  title: string;
  description: string;
  placement: 'top' | 'bottom' | 'left' | 'right';
}

interface FeatureGuideProps {
  onComplete: () => void;
  onSkip: () => void;
}

const STEPS: GuideStep[] = [
  { target: "workbench", title: "guide.workbench", description: "guide.workbench_desc", placement: "bottom" },
  { target: "studio", title: "guide.studio", description: "guide.studio_desc", placement: "bottom" },
  { target: "hub", title: "guide.hub", description: "guide.hub_desc", placement: "bottom" },
  { target: "console", title: "guide.console", description: "guide.console_desc", placement: "bottom" },
];

export function FeatureGuide({ onComplete, onSkip }: FeatureGuideProps) {
  const { t } = useTranslation();
  const [step, setStep] = useState(0);
  const [rect, setRect] = useState<DOMRect | null>(null);

  const current = STEPS[step];

  const updatePosition = useCallback(() => {
    const el = document.querySelector<HTMLElement>(`[data-guide-target="${current.target}"]`);
    if (el) {
      setRect(el.getBoundingClientRect());
    }
  }, [current.target]);

  useEffect(() => {
    updatePosition();
    window.addEventListener("scroll", updatePosition);
    window.addEventListener("resize", updatePosition);
    return () => {
      window.removeEventListener("scroll", updatePosition);
      window.removeEventListener("resize", updatePosition);
    };
  }, [updatePosition]);

  const handleNext = () => {
    if (step < STEPS.length - 1) {
      setStep(s => s + 1);
    } else {
      onComplete();
    }
  };

  const tooltipStyle: React.CSSProperties = {
    position: "fixed",
    zIndex: 10001,
    background: "var(--bg-card, #1a1b26)",
    border: "1px solid var(--accent, #58a6ff)",
    borderRadius: 12,
    padding: "16px 20px",
    width: 300,
    boxShadow: "0 4px 24px rgba(0,0,0,0.3)",
    color: "var(--text-primary, #c9d1d9)",
  };

  if (rect) {
    switch (current.placement) {
      case "bottom":
        tooltipStyle.top = rect.bottom + 12;
        tooltipStyle.left = Math.max(8, rect.left + rect.width / 2 - 150);
        break;
      case "top":
        tooltipStyle.bottom = window.innerHeight - rect.top + 12;
        tooltipStyle.left = Math.max(8, rect.left + rect.width / 2 - 150);
        break;
      case "left":
        tooltipStyle.right = window.innerWidth - rect.left + 12;
        tooltipStyle.top = Math.max(8, rect.top + rect.height / 2 - 60);
        break;
      case "right":
        tooltipStyle.left = rect.right + 12;
        tooltipStyle.top = Math.max(8, rect.top + rect.height / 2 - 60);
        break;
    }
  }

  return (
    <>
      <div style={{
        position: "fixed", inset: 0, zIndex: 10000,
        background: "rgba(0,0,0,0.4)",
      }} onClick={onSkip} />
      {rect && (
        <div style={{
          position: "fixed", zIndex: 10000, pointerEvents: "none",
          border: "2px dashed #f0c040", borderRadius: 8,
          top: rect.top - 4, left: rect.left - 4,
          width: rect.width + 8, height: rect.height + 8,
          transition: "all 0.25s ease",
        }} />
      )}
      <div style={tooltipStyle}>
        <div style={{ fontSize: 13, fontWeight: 600, marginBottom: 6 }}>
          {t(current.title)}
        </div>
        <div style={{ fontSize: 13, color: "var(--text-secondary, #8b949e)", marginBottom: 12 }}>
          {t(current.description)}
        </div>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
          <span style={{ fontSize: 12, color: "var(--text-secondary, #8b949e)" }}>
            {step + 1} / {STEPS.length}
          </span>
          <div style={{ display: "flex", gap: 8 }}>
            <button
              onClick={onSkip}
              style={{
                padding: "6px 14px", borderRadius: 6, border: "1px solid var(--border, #30363d)",
                background: "transparent", color: "inherit", cursor: "pointer", fontSize: 13,
              }}
            >
              {t('guide.skip')}
            </button>
            <button
              onClick={handleNext}
              style={{
                padding: "6px 16px", borderRadius: 6, border: "none",
                background: "var(--accent, #58a6ff)", color: "#fff", cursor: "pointer", fontSize: 13, fontWeight: 600,
              }}
            >
              {step < STEPS.length - 1 ? t('onboarding.next') : t('guide.done')}
            </button>
          </div>
        </div>
      </div>
    </>
  );
}
