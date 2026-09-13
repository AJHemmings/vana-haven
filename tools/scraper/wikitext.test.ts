import { test } from "node:test";
import assert from "node:assert/strict";
import { findTemplateBlocks, parseTemplateFields, stripWikiLink } from "./wikitext.ts";

test("findTemplateBlocks finds a single simple block", () => {
  const wikitext = "before\n{{Foo\n|a=1\n|b=2\n}}\nafter";
  const blocks = findTemplateBlocks(wikitext, "Foo");
  assert.equal(blocks.length, 1);
  assert.equal(blocks[0], "{{Foo\n|a=1\n|b=2\n}}");
});

test("findTemplateBlocks returns an empty array when the template isn't present", () => {
  assert.deepEqual(findTemplateBlocks("no templates here", "Foo"), []);
});

test("findTemplateBlocks finds multiple occurrences", () => {
  const wikitext = "{{Foo|a=1}}\nsome text\n{{Foo|a=2}}";
  const blocks = findTemplateBlocks(wikitext, "Foo");
  assert.equal(blocks.length, 2);
  assert.match(blocks[0], /a=1/);
  assert.match(blocks[1], /a=2/);
});

// Real distinguishing case from BG-Wiki: "{{Relic Set" and "{{Relic + Set" are two
// different templates whose names share a prefix. A naive substring search for
// "{{Relic Set" must not match inside "{{Relic + Set" (confirmed these are genuinely
// different templates in Duelist's Attire Set's real wikitext).
test("does not match a template whose name is a different template with a shared prefix", () => {
  const wikitext = "{{Relic + Set\n|plus=1\n|relic job=Red Mage\n}}";
  assert.deepEqual(findTemplateBlocks(wikitext, "Relic Set"), []);
  assert.equal(findTemplateBlocks(wikitext, "Relic + Set").length, 1);
  assert.deepEqual(findTemplateBlocks("{{Relic Setter\n|x=1\n}}", "Relic Set"), []);
});

// Real case from Duelist's Attire Set's "+2" tier block: a relic stats field embeds
// a nested {{augment}} template. The block-extractor must balance nested {{ }} pairs
// rather than stopping at the first "}}", or it would truncate mid-block.
test("balances nested templates inside a field value instead of stopping at the first closing brace", () => {
  const wikitext =
    "{{Relic + Set\n" +
    "|plus=2\n" +
    "|relic stats 1=DEF:31 {{augment}}: \"Enfeebling Magic duration\"\n" +
    "|relic feet=Duelist's Boots +2\n" +
    "}}\n" +
    "trailing text outside the block";
  const blocks = findTemplateBlocks(wikitext, "Relic + Set");
  assert.equal(blocks.length, 1);
  assert.match(blocks[0], /relic feet=Duelist's Boots \+2/);
  assert.doesNotMatch(blocks[0], /trailing text/);
});

test("parseTemplateFields extracts key=value pairs from a block's lines", () => {
  const block = "{{Foo\n|plus=1\n|jobs=Red Mage\n|head=Atrophy Chapeau +1\n}}";
  const fields = parseTemplateFields(block);
  assert.equal(fields["plus"], "1");
  assert.equal(fields["jobs"], "Red Mage");
  assert.equal(fields["head"], "Atrophy Chapeau +1");
});

test("parseTemplateFields treats a blank field value as an empty string, not absent", () => {
  const block = "{{Foo\n|plus=\n|jobs=Red Mage\n}}";
  const fields = parseTemplateFields(block);
  assert.equal(fields["plus"], "");
  assert.equal("plus" in fields, true);
});

// Real distinguishing case: "|head=" must not be confused with "|head base stats=" —
// both start with "|head" but only one is the literal key "head".
test("parseTemplateFields does not confuse a field with a longer field sharing its prefix", () => {
  const block = "{{Foo\n|head=Atrophy Chapeau\n|head base stats=DEF:69 HP+17\n}}";
  const fields = parseTemplateFields(block);
  assert.equal(fields["head"], "Atrophy Chapeau");
  assert.equal(fields["head base stats"], "DEF:69 HP+17");
});

test("parseTemplateFields ignores lines that aren't |key=value pairs", () => {
  const block = "{{Foo\n|a=1\nsome free text\n}}";
  const fields = parseTemplateFields(block);
  assert.deepEqual(Object.keys(fields), ["a"]);
});

test("does not return a bogus block when a template is never closed", () => {
  const wikitext = "before\n{{Foo\n|a=1\n|b=2\nafter with no closing braces at all";
  assert.deepEqual(findTemplateBlocks(wikitext, "Foo"), []);
});

test("stripWikiLink strips a plain wikilink", () => {
  assert.equal(stripWikiLink("[[Scholar]]"), "Scholar");
});

test("stripWikiLink uses the display text of a piped wikilink", () => {
  assert.equal(stripWikiLink("[[Scholar|SCH]]"), "SCH");
});

test("stripWikiLink returns a plain (non-wikilink) value unchanged", () => {
  assert.equal(stripWikiLink("Scholar"), "Scholar");
});
