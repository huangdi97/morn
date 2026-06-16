import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import WelcomeGuide from "../welcome/WelcomeGuide";

describe("WelcomeGuide", () => {
  it("renders welcome text content", () => {
    render(<WelcomeGuide onDismiss={() => {}} />);
    expect(screen.getByText("欢迎使用 Morn")).toBeInTheDocument();
  });

  it("renders description text", () => {
    render(<WelcomeGuide onDismiss={() => {}} />);
    expect(
      screen.getByText("你需要配置一个 AI 模型才能开始")
    ).toBeInTheDocument();
  });
});