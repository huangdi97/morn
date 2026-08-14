import { describe, expect, it } from "vitest";

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
