import { test } from "node:test";
import assert from "node:assert/strict";
import { resolveItemId } from "./extract.ts";

const ITEM_PAGE_WIKITEXT = `{{item
|storagemethod=Storage Slip 25
|ffxidb=23580
}}
{{armor
|slot=Legs
|jobs=[[Red Mage]]
}}`;

test("resolveItemId returns the numeric ffxidb value from the item page", () => {
  assert.equal(resolveItemId(ITEM_PAGE_WIKITEXT, "Atrophy Tights +3"), 23580);
});

test("resolveItemId returns null when no wikitext was found for the item", () => {
  assert.equal(resolveItemId(undefined, "Some Missing Item"), null);
});

test("resolveItemId returns null when the item page has no ffxidb field", () => {
  const wikitextWithoutFfxidb = `{{item\n|storagemethod=Storage Slip 25\n}}`;
  assert.equal(resolveItemId(wikitextWithoutFfxidb, "Some Item"), null);
});

test("resolveItemId returns null when there is no {{item}} template at all", () => {
  assert.equal(resolveItemId("some unrelated page content", "Some Item"), null);
});
