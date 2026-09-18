export type Cadence = "daily" | "weekly" | "monthly";

const JST_OFFSET_MS = 9 * 3600 * 1000;

// Daily reset is JST midnight, i.e. 15:00 UTC (spec §8.4) — the most recent
// such instant at or before `now`.
export function dailyBoundary(now: Date): number {
  const shifted = new Date(now.getTime() + JST_OFFSET_MS);
  const midnightShifted = Date.UTC(shifted.getUTCFullYear(), shifted.getUTCMonth(), shifted.getUTCDate());
  return midnightShifted - JST_OFFSET_MS;
}

// Weekly reset is Sunday JST midnight — the most recent such instant at or
// before `now`, found by walking back to Sunday within the JST-shifted
// wall-clock date before converting back to a real UTC instant.
export function weeklyBoundary(now: Date): number {
  const shifted = new Date(now.getTime() + JST_OFFSET_MS);
  const dayOfWeek = shifted.getUTCDay(); // 0 = Sunday
  const midnightTodayShifted = Date.UTC(shifted.getUTCFullYear(), shifted.getUTCMonth(), shifted.getUTCDate());
  const sundayMidnightShifted = midnightTodayShifted - dayOfWeek * 86400000;
  return sundayMidnightShifted - JST_OFFSET_MS;
}

// Monthly has no fixed-formula boundary — a version update's date varies
// month to month, so it's a marker the user advances manually rather than
// something derivable from `now` (spec §8.4, §11). `monthlyCycleStartedAt`
// is that marker's raw timestamp, fetched from the backend.
export function isDoneForCurrentPeriod(
  cadence: Cadence,
  lastCompletedAt: string | null,
  now: Date,
  monthlyCycleStartedAt: string | null
): boolean {
  if (lastCompletedAt === null) return false;
  const completedMs = new Date(lastCompletedAt).getTime();

  if (cadence === "daily") return completedMs >= dailyBoundary(now);
  if (cadence === "weekly") return completedMs >= weeklyBoundary(now);

  // monthly: nothing has invalidated this completion if no cycle has been
  // started yet.
  if (monthlyCycleStartedAt === null) return true;
  return completedMs >= new Date(monthlyCycleStartedAt).getTime();
}
