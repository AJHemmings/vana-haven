import { test } from "node:test";
import assert from "node:assert/strict";
import { fetchWikitextBatch } from "./fetch.ts";

// Opt-in only — real network call to BG-Wiki. Run explicitly with:
//   RUN_SCRAPER_INTEGRATION=1 node --test tools/scraper/integration.test.ts   (bash)
//   $env:RUN_SCRAPER_INTEGRATION=1; node --test tools/scraper/integration.test.ts   (PowerShell)
// Skipped by default so `npm run test:scraper` never depends on network access.
if (process.env.RUN_SCRAPER_INTEGRATION) {
  test("BG-Wiki API still returns wikitext in the expected shape for a known page", async () => {
    const result = await fetchWikitextBatch(["Atrophy Armor Set"], { force: true, cacheDir: "tools/scraper/.cache" });
    const wikitext = result.get("Atrophy Armor Set");

    assert.ok(wikitext, "expected wikitext content for Atrophy Armor Set");
    assert.match(wikitext!, /\{\{Armor Set Table/);
    assert.match(wikitext!, /\{\{R Artifact Set 2/);
  });
}
