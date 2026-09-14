import { test } from "node:test";
import assert from "node:assert/strict";
import { discoverCategory } from "./discover.ts";

const SET_OVERVIEW_WIKITEXT = `{{Armor Set Table
|Armor Set 1=
{{R Artifact Set 2
|plus=
|jobs=Red Mage
|head=Atrophy Chapeau
}}
}}`;

const ITEM_PAGE_WIKITEXT = `{{item
|ffxidb=23580
}}
{{armor
|slot=Legs
|jobs=[[Red Mage]]
}}`;

test("buckets a page containing {{Armor Set Table}} into matched, with job read via the category's extractor", async () => {
  const fetchCategoryMembersFn = async () => ["Atrophy Armor Set"];
  const fetchWikitextBatchFn = async () => new Map([["Atrophy Armor Set", SET_OVERVIEW_WIKITEXT]]);

  const manifest = await discoverCategory("af3", { fetchCategoryMembersFn, fetchWikitextBatchFn });

  assert.deepEqual(manifest.matched, [{ job: "Red Mage", setType: "af3", setPageTitle: "Atrophy Armor Set" }]);
});

test("does not log an individual item page as unmatched", async () => {
  const fetchCategoryMembersFn = async () => ["Atrophy Tights +3"];
  const fetchWikitextBatchFn = async () => new Map([["Atrophy Tights +3", ITEM_PAGE_WIKITEXT]]);

  const manifest = await discoverCategory("af3", { fetchCategoryMembersFn, fetchWikitextBatchFn });

  assert.deepEqual(manifest.matched, []);
  assert.deepEqual(manifest.unmatched, []);
});

test("logs a page matching neither template as unmatched, with a reason", async () => {
  const fetchCategoryMembersFn = async () => ["Template:R Artifact Set 2"];
  const fetchWikitextBatchFn = async () => new Map([["Template:R Artifact Set 2", "some template definition wikitext"]]);

  const manifest = await discoverCategory("af3", { fetchCategoryMembersFn, fetchWikitextBatchFn });

  assert.deepEqual(manifest.unmatched, [
    { pageTitle: "Template:R Artifact Set 2", reason: "no {{Armor Set Table}} or {{item}} template found" },
  ]);
});

test("computes missingJobs by diffing matched jobs against the 22 canonical jobs", async () => {
  const fetchCategoryMembersFn = async () => ["Atrophy Armor Set"];
  const fetchWikitextBatchFn = async () => new Map([["Atrophy Armor Set", SET_OVERVIEW_WIKITEXT]]);

  const manifest = await discoverCategory("af3", { fetchCategoryMembersFn, fetchWikitextBatchFn });

  assert.equal(manifest.missingJobs.length, 21);
  assert.ok(!manifest.missingJobs.includes("Red Mage"));
  assert.ok(manifest.missingJobs.includes("Warrior"));
});

test("a page missing from the wikitext map entirely is treated as unmatched, not skipped", async () => {
  const fetchCategoryMembersFn = async () => ["Some Oddly Templated Page"];
  const fetchWikitextBatchFn = async () => new Map<string, string>();

  const manifest = await discoverCategory("af3", { fetchCategoryMembersFn, fetchWikitextBatchFn });

  assert.equal(manifest.unmatched.length, 1);
  assert.equal(manifest.unmatched[0].pageTitle, "Some Oddly Templated Page");
  assert.match(manifest.unmatched[0].reason, /no wikitext returned/);
});
