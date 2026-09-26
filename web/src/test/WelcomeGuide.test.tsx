import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import WelcomeGuide from "../welcome/WelcomeGuide";
import { LocaleProvider } from "../i18n";

// WelcomeGuide resolves text via useTranslation(); without a LocaleProvider it
// falls back to the default context which returns raw i18n keys.
function renderGuide() {
  return render(
    <LocaleProvider>
      <WelcomeGuide onDismiss={() => {}} />
    </LocaleProvider>
  );
}

describe("WelcomeGuide", () => {
  it("renders welcome text content", () => {
    renderGuide();
    expect(screen.getByText("欢迎使用 Morn")).toBeInTheDocument();
  });

  it("renders description text", () => {
    renderGuide();
    expect(
      screen.getByText("你需要配置一个 AI 模型才能开始")
    ).toBeInTheDocument();
  });
});
