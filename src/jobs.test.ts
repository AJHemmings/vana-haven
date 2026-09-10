import { describe, expect, it } from "vitest";
import { JOBS, jobAbbreviation } from "./jobs";

describe("jobs", () => {
  it("has all 22 jobs", () => {
    expect(JOBS).toHaveLength(22);
  });

  it("returns the abbreviation for a known job id", () => {
    expect(jobAbbreviation(4)).toBe("BLM");
  });

  it("falls back to a generic label for an unknown job id", () => {
    expect(jobAbbreviation(999)).toBe("Job 999");
  });
});
