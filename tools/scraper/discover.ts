import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { pathToFileURL } from "node:url";
import { CANONICAL_JOBS, CATEGORIES, OUTPUT_DIR, type CategoryId } from "./config.ts";
import { fetchCategoryMembers, fetchWikitextBatch, type ScraperDeps } from "./fetch.ts";
import { findTemplateBlocks } from "./wikitext.ts";
import { EXTRACTORS } from "./extractors/index.ts";
import type { DiscoveryManifest } from "./types.ts";

export type DiscoverDeps = {
  fetchCategoryMembersFn?: typeof fetchCategoryMembers;
  fetchWikitextBatchFn?: typeof fetchWikitextBatch;
} & ScraperDeps;

export async function discoverCategory(categoryId: CategoryId, deps: DiscoverDeps = {}): Promise<DiscoveryManifest> {
  const { fetchCategoryMembersFn = fetchCategoryMembers, fetchWikitextBatchFn = fetchWikitextBatch, ...fetchDeps } = deps;
  // Default throttleState HERE, not just in main() — this is the function that
  // actually owns both fetch calls, so it must guarantee they share one throttle
  // timeline regardless of whether the caller remembered to build one.
  fetchDeps.throttleState ??= { hasMadeRequest: false };
  const config = CATEGORIES[categoryId];
  const extractor = EXTRACTORS[categoryId];

  const memberTitles = await fetchCategoryMembersFn(config.seedCategory, fetchDeps);
  const wikitextByTitle = await fetchWikitextBatchFn(memberTitles, fetchDeps);

  const manifest: DiscoveryManifest = { matched: [], unmatched: [], missingJobs: [] };

  for (const title of memberTitles) {
    const wikitext = wikitextByTitle.get(title);

    if (wikitext === undefined) {
      manifest.unmatched.push({
        pageTitle: title,
        reason: "no wikitext returned for this title — possible title mismatch, deleted page, or fetch gap",
      });
    } else if (findTemplateBlocks(wikitext, "Armor Set Table").length > 0) {
      const job = extractor(wikitext)[0]?.job ?? "Unknown";
      manifest.matched.push({ job, setType: categoryId, setPageTitle: title });
    } else if (findTemplateBlocks(wikitext, "item").length > 0) {
      // An individual item page — expected category membership, not logged.
    } else {
      manifest.unmatched.push({
        pageTitle: title,
        reason: "no {{Armor Set Table}} or {{item}} template found",
      });
    }
  }

  const matchedJobs = new Set(manifest.matched.map((m) => m.job));
  manifest.missingJobs = CANONICAL_JOBS.filter((job) => !matchedJobs.has(job));

  return manifest;
}

async function main() {
  const [categoryArg, ...rest] = process.argv.slice(2);
  const force = rest.includes("--force");

  if (!categoryArg || !(categoryArg in CATEGORIES)) {
    console.error("Usage: node tools/scraper/discover.ts <af3|empyrean|relic> [--force]");
    process.exit(1);
  }

  // One shared throttleState for the whole run — without this, the categorymembers
  // call and the wikitext batch call inside discoverCategory would each default to
  // their own independent throttle state (fetch.ts's ScraperDeps default is a fresh
  // object per call unless the caller supplies one), meaning no delay would actually
  // be enforced between them. Confirmed as a real gap during Task 3's code review.
  const manifest = await discoverCategory(categoryArg as CategoryId, {
    force,
    throttleState: { hasMadeRequest: false },
  });

  mkdirSync(OUTPUT_DIR, { recursive: true });
  const outPath = join(OUTPUT_DIR, `discovered-${categoryArg}.json`);
  writeFileSync(outPath, JSON.stringify(manifest, null, 2));

  const unresolvedJobCount = manifest.matched.filter((m) => m.job === "Unknown").length;
  console.log(
    `[scraper] wrote ${outPath} — matched ${manifest.matched.length}${unresolvedJobCount ? ` (${unresolvedJobCount} with unresolved job — review before running extract.ts)` : ""}, unmatched ${manifest.unmatched.length}, missingJobs: ${manifest.missingJobs.join(", ") || "none"}`
  );
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  main();
}
