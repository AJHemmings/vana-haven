import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { CATEGORIES, OUTPUT_DIR, type CategoryId } from "./config.ts";
import { fetchWikitextBatch, type ScraperDeps } from "./fetch.ts";
import { findTemplateBlocks } from "./wikitext.ts";
import { EXTRACTORS } from "./extractors/index.ts";
import type { DiscoveryManifest, ExtractedEntry, FinalEntry } from "./types.ts";

function resolveItemId(wikitext: string | undefined, itemName: string): number | null {
  if (!wikitext) {
    console.warn(`[scraper] no item page found for "${itemName}" — itemId set to null`);
    return null;
  }

  const itemBlocks = findTemplateBlocks(wikitext, "item");
  const match = itemBlocks[0]?.match(/\|ffxidb=(\d+)/);
  if (!match) {
    console.warn(`[scraper] "${itemName}" has no ffxidb field — itemId set to null`);
    return null;
  }

  return Number(match[1]);
}

export async function extractCategory(categoryId: CategoryId, deps: ScraperDeps = {}): Promise<FinalEntry[]> {
  // Default throttleState HERE, not just in main() — this function owns both
  // fetchWikitextBatch calls below, so it must guarantee they share one throttle
  // timeline regardless of whether the caller remembered to build one. Same fix
  // applied to discoverCategory after Task 7's code review found the same gap.
  deps.throttleState ??= { hasMadeRequest: false };

  const manifestPath = join(OUTPUT_DIR, `discovered-${categoryId}.json`);
  if (!existsSync(manifestPath)) {
    throw new Error(`No manifest found at ${manifestPath} — run discover.ts for "${categoryId}" first.`);
  }
  const manifest: DiscoveryManifest = JSON.parse(readFileSync(manifestPath, "utf8"));

  const extractor = EXTRACTORS[categoryId];
  const setPageTitles = manifest.matched.map((m) => m.setPageTitle);
  const setWikitext = await fetchWikitextBatch(setPageTitles, deps);

  const rawEntries: ExtractedEntry[] = [];
  for (const { setPageTitle } of manifest.matched) {
    const wikitext = setWikitext.get(setPageTitle) ?? "";
    rawEntries.push(...extractor(wikitext));
  }

  const itemNames = [...new Set(rawEntries.map((e) => e.itemName))];
  const itemWikitext = await fetchWikitextBatch(itemNames, deps);

  return rawEntries.map((entry) => ({
    ...entry,
    itemId: resolveItemId(itemWikitext.get(entry.itemName), entry.itemName),
  }));
}

async function main() {
  const [categoryArg, ...rest] = process.argv.slice(2);
  const force = rest.includes("--force");

  if (!categoryArg || !(categoryArg in CATEGORIES)) {
    console.error("Usage: node tools/scraper/extract.ts <af3|empyrean|relic> [--force]");
    process.exit(1);
  }

  // One shared throttleState for the whole run — extractCategory makes two separate
  // fetchWikitextBatch calls (set-overview pages, then item pages); without an
  // explicitly shared throttleState each would default to its own independent one,
  // so no delay would be enforced between them. Same fix as discover.ts's main() —
  // see that task's comment for why (confirmed as a real gap in Task 3's code review).
  const entries = await extractCategory(categoryArg as CategoryId, {
    force,
    throttleState: { hasMadeRequest: false },
  });

  mkdirSync(OUTPUT_DIR, { recursive: true });
  const outPath = join(OUTPUT_DIR, `gearsets-${categoryArg}.json`);
  writeFileSync(outPath, JSON.stringify(entries, null, 2));

  const nullCount = entries.filter((e) => e.itemId === null).length;
  console.log(`[scraper] wrote ${entries.length} entries to ${outPath}${nullCount ? ` (${nullCount} with itemId: null — see warnings above)` : ""}`);
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  main();
}
