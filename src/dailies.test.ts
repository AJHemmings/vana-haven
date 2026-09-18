import { describe, expect, it } from "vitest";
import { dailyBoundary, isDoneForCurrentPeriod, weeklyBoundary } from "./dailies";

describe("dailyBoundary", () => {
  it("is exactly now when now sits precisely on a 15:00 UTC boundary", () => {
    const now = new Date("2026-09-17T15:00:00.000Z");
    expect(new Date(dailyBoundary(now)).toISOString()).toBe("2026-09-17T15:00:00.000Z");
  });

  it("falls back to the previous day's boundary just before 15:00 UTC", () => {
    const now = new Date("2026-09-17T14:59:59.000Z");
    expect(new Date(dailyBoundary(now)).toISOString()).toBe("2026-09-16T15:00:00.000Z");
  });

  it("stays on today's boundary later in the day", () => {
    const now = new Date("2026-09-17T20:00:00.000Z");
    expect(new Date(dailyBoundary(now)).toISOString()).toBe("2026-09-17T15:00:00.000Z");
  });
});

describe("weeklyBoundary", () => {
  // 2026-09-19 is a Saturday; Sat 15:00 UTC = Sun 00:00 JST.
  it("falls back to last week's boundary just before Saturday 15:00 UTC", () => {
    const now = new Date("2026-09-19T14:00:00.000Z");
    expect(new Date(weeklyBoundary(now)).toISOString()).toBe("2026-09-12T15:00:00.000Z");
  });

  it("crosses onto this week's boundary at Saturday 15:00 UTC", () => {
    const now = new Date("2026-09-19T16:00:00.000Z");
    expect(new Date(weeklyBoundary(now)).toISOString()).toBe("2026-09-19T15:00:00.000Z");
  });

  it("stays on the same weekly boundary through the rest of the week", () => {
    const now = new Date("2026-09-20T10:00:00.000Z");
    expect(new Date(weeklyBoundary(now)).toISOString()).toBe("2026-09-19T15:00:00.000Z");
  });
});

describe("isDoneForCurrentPeriod", () => {
  const now = new Date("2026-09-17T20:00:00.000Z"); // daily boundary: 2026-09-17T15:00:00Z

  it("is never done when never completed", () => {
    expect(isDoneForCurrentPeriod("daily", null, now, null)).toBe(false);
  });

  it("daily: done when completed after today's boundary", () => {
    expect(isDoneForCurrentPeriod("daily", "2026-09-17T16:00:00.000Z", now, null)).toBe(true);
  });

  it("daily: not done when last completed before today's boundary", () => {
    expect(isDoneForCurrentPeriod("daily", "2026-09-16T20:00:00.000Z", now, null)).toBe(false);
  });

  it("weekly: done when completed within the current weekly period", () => {
    const weeklyNow = new Date("2026-09-20T10:00:00.000Z"); // boundary 2026-09-19T15:00:00Z
    expect(isDoneForCurrentPeriod("weekly", "2026-09-19T18:00:00.000Z", weeklyNow, null)).toBe(true);
  });

  it("weekly: not done when last completed before the current weekly boundary", () => {
    const weeklyNow = new Date("2026-09-20T10:00:00.000Z");
    expect(isDoneForCurrentPeriod("weekly", "2026-09-15T00:00:00.000Z", weeklyNow, null)).toBe(false);
  });

  it("monthly: done if ever completed and no cycle has been started yet", () => {
    expect(isDoneForCurrentPeriod("monthly", "2026-01-01T00:00:00.000Z", now, null)).toBe(true);
  });

  it("monthly: done when completed after the current cycle started", () => {
    expect(isDoneForCurrentPeriod("monthly", "2026-09-05T00:00:00.000Z", now, "2026-09-01T15:00:00.000Z")).toBe(true);
  });

  it("monthly: not done when last completed before the current cycle started", () => {
    expect(isDoneForCurrentPeriod("monthly", "2026-08-05T00:00:00.000Z", now, "2026-09-01T15:00:00.000Z")).toBe(
      false
    );
  });
});
