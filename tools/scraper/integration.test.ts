import { test } from "node:test";
import assert from "node:assert/strict";
import { fetchWikitextBatch } from "./fetch.ts";
import { extractAf3 } from "./extractors/af3.ts";

// Opt-in only — real network call to BG-Wiki. Run explicitly with:
//   RUN_SCRAPER_INTEGRATION=1 node --test tools/scraper/integration.test.ts   (bash)
//   $env:RUN_SCRAPER_INTEGRATION=1; node --test tools/scraper/integration.test.ts   (PowerShell)
// Skipped by default so `npm run test:scraper` never depends on network access.
if (process.env.RUN_SCRAPER_INTEGRATION) {
  test("BG-Wiki API still returns wikitext the real AF3 extractor can parse", async () => {
    const result = await fetchWikitextBatch(["Atrophy Armor Set"], { force: true, cacheDir: "tools/scraper/.cache" });
    const wikitext = result.get("Atrophy Armor Set");

    assert.ok(wikitext, "expected wikitext content for Atrophy Armor Set");

    // Routing through the real extractor (not just checking template-name
    // substrings) catches a wiki editor renaming a field inside the template
    // (e.g. head= -> helm=), which a substring check on the template name
    // alone would miss entirely.
    const entries = extractAf3(wikitext!);
    const baseHead = entries.find((e) => e.slot === "head" && e.tier === "0");
    assert.ok(baseHead, "expected extractAf3 to parse a base-tier head entry from live wikitext");
    assert.equal(baseHead!.job, "Red Mage");
  });
}
