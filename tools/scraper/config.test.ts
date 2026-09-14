import { test } from "node:test";
import assert from "node:assert/strict";
import { CANONICAL_JOBS, CATEGORIES } from "./config.ts";

test("CANONICAL_JOBS has all 22 FFXI jobs", () => {
  assert.equal(CANONICAL_JOBS.length, 22);
  assert.ok(CANONICAL_JOBS.includes("Red Mage"));
  assert.ok(CANONICAL_JOBS.includes("Rune Fencer"));
});

test("CANONICAL_JOBS has no duplicates", () => {
  assert.equal(new Set(CANONICAL_JOBS).size, CANONICAL_JOBS.length);
});

test("CATEGORIES has exactly af3, empyrean, and relic with non-empty seed categories", () => {
  const ids = Object.keys(CATEGORIES).sort();
  assert.deepEqual(ids, ["af3", "empyrean", "relic"]);
  for (const id of ids) {
    assert.ok(CATEGORIES[id as keyof typeof CATEGORIES].seedCategory.length > 0);
  }
});
