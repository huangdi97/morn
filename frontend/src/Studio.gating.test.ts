import { describe, expect, it } from "vitest";

import { reviewedSolutionPolicy } from "./pages/Studio";

describe("reviewed SolutionPackage policy projection", () => {
  it("projects immutable v11.5 profile/site/acceptance constraints for Work instantiation", () => {
    expect(
      reviewedSolutionPolicy({
        schema: "morn.solution-package/v11.5",
        profile_ref: "morn.factory.readonly@1.0.0",
        site_ref: "plant-a",
        acceptance_criteria: ["delivery impact reviewed", "source evidence attached"],
      }),
    ).toEqual({
      profileRef: "morn.factory.readonly@1.0.0",
      siteRef: "plant-a",
      acceptanceCriteria: ["delivery impact reviewed", "source evidence attached"],
    });
  });

  it("preserves site-neutral reviewed packages without inventing a site", () => {
    expect(
      reviewedSolutionPolicy({
        schema: "morn.solution-package/v11.5",
        profile_ref: "morn.lite@1.0.0",
        site_ref: null,
        acceptance_criteria: ["reviewed outcome exists"],
      }),
    ).toEqual({
      profileRef: "morn.lite@1.0.0",
      siteRef: null,
      acceptanceCriteria: ["reviewed outcome exists"],
    });
  });

  it("does not reinterpret legacy packages as v11.5 reviewed policy", () => {
    expect(reviewedSolutionPolicy({ kind: "legacy" })).toBeNull();
    expect(reviewedSolutionPolicy(null)).toBeNull();
  });
});
