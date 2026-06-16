import { describe, it, expect } from "vitest";
import { render, screen } from "@testing-library/react";
import StatusBar from "../StatusBar";

describe("StatusBar", () => {
  it("renders status indicator", () => {
    render(<StatusBar />);
    expect(screen.getByText(/🟢/)).toBeInTheDocument();
  });

  it("shows cost area", () => {
    render(<StatusBar />);
    expect(screen.getByText(/💰/)).toBeInTheDocument();
  });

  it("shows provider info", () => {
    render(<StatusBar />);
    expect(screen.getByText(/Provider/)).toBeInTheDocument();
  });
});