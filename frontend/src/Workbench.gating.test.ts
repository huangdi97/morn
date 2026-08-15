import { describe, expect, it } from "vitest";
import { biolabEnabled } from "./pages/Workbench";

describe("domain-gated UI extension point", () => {
  it("zero-domain: no BioLab UI is rendered", () => {
    expect(biolabEnabled([])).toBe(false);
    expect(biolabEnabled(["generic"])).toBe(false);
  });

  it("enabled only when the backend advertises the biolab-reference pack", () => {
    expect(biolabEnabled(["biolab-reference"])).toBe(true);
  });
});
