import { describe, expect, it } from "vitest";
import { apiBaseForOrigin } from "./api";

describe("morn frontend", () => {
  it("has four product surfaces", () => {
    const surfaces = ["Workbench", "Studio", "Console", "Hub"];
    expect(surfaces).toHaveLength(4);
    expect(surfaces).toContain("Workbench");
  });

  it("treats status strings as pills", () => {
    const cls = "InReview".toLowerCase().replace(/[^a-z0-9]/g, "-");
    expect(cls).toBe("inreview");
  });
});

describe("API origin routing", () => {
  it("keeps browser/dev API requests relative to the Vite proxy", () => {
    expect(apiBaseForOrigin("http://127.0.0.1:5173/workbench")).toBe("/api");
    expect(apiBaseForOrigin("https://morn.example/workbench")).toBe("/api");
    expect(apiBaseForOrigin("")).toBe("/api");
  });

  it("uses the local backend for both Tauri desktop origins", () => {
    expect(apiBaseForOrigin("tauri://localhost/workbench")).toBe("http://127.0.0.1:8090/api");
    expect(apiBaseForOrigin("http://tauri.localhost/console")).toBe("http://127.0.0.1:8090/api");
  });
});
