import { test } from "node:test";
import assert from "node:assert/strict";
import { extractRelic } from "./relic.ts";

const RELIC_FIXTURE = `{{Armor Set Table
|Image=Duelist's_Armor_Set.jpg
|Image Size=200
|Image Link=
|Armor Set 1=
{{Relic Set
|relic job=Red Mage
|relic head=Duelist's Chapeau
|relic level 1=75
|relic stats 1=[[DEF]]:24 [[MP]]+14<br />Adds [[Refresh]] effect<br />
|relic body=Duelist's Tabard
|relic level 2=74
|relic stats 2=[[DEF]]:45 [[MP]]+24<br>Enhances [[Fast Cast]] effect
|relic hands=Duelist's Gloves
|relic level 3=72
|relic stats 3=[[DEF]]:17 [[MP]]+18
|relic legs=Duelist's Tights
|relic level 4=73
|relic stats 4=[[DEF]]:33 [[MP]]+16
|relic feet=Duelist's Boots
|relic level 5=71
|relic stats 5=[[DEF]]:15 [[MP]]+15
}}
|Armor Set 2=
|Armor Set 3=
{{Relic + Set
|plus=1
|relic job=Red Mage
|relic level=75
|relic head=Duelist's Chapeau +1
|relic stats 1=[[DEF]]:25 [[HP]]+14
|relic body=Duelist's Tabard +1
|relic stats 2=[[DEF]]:46 [[MP]]+30
|relic hands=Duelist's Gloves +1
|relic stats 3=[[DEF]]:18 [[MP]]+23
|relic legs=Duelist's Tights +1
|relic stats 4=[[DEF]]:34 [[MP]]+16
|relic feet=Duelist's Boots +1
|relic stats 5=[[DEF]]:16 [[MP]]+15
}}
|Armor Set 4=
{{Relic + Set
|plus=2
|relic job=Red Mage
|relic level=90
|relic head=Duelist's Chapeau +2
|relic stats 1=[[Defense|DEF]]:31 {{augment}}<span style=color:#D696AA>:</span> "Enfeebling Magic duration"
|relic body=Duelist's Tabard +2
|relic stats 2=[[Defense|DEF]]:59 {{augment}}<span style=color:#D696AA>:</span> "Enhances 'Chainspell' effect"
|relic hands=Duelist's Gloves +2
|relic stats 3=[[Defense|DEF]]:22 {{augment}}<span style=color:#D696AA>:</span> "Enhancing Magic duration"
|relic legs=Duelist's Tights +2
|relic stats 4=[[Defense|DEF]]:43 {{augment}}<span style=color:#D696AA>:</span> "Enspell Damage" and "Accuracy"
|relic feet=Duelist's Boots +2
|relic stats 5=[[Defense|DEF]]:20 {{augment}}<span style=color:#D696AA>:</span> "Immunobreak Chance"
}}
}}`;

test("extracts all 5 slots for the NQ tier from {{Relic Set}}, tier normalized to \"0\"", () => {
  const entries = extractRelic(RELIC_FIXTURE).filter((e) => e.tier === "0");
  assert.equal(entries.length, 5);
  const head = entries.find((e) => e.slot === "head")!;
  assert.equal(head.job, "Red Mage");
  assert.equal(head.setType, "relic");
  assert.equal(head.itemName, "Duelist's Chapeau");
});

test("extracts the +1 tier from {{Relic + Set}} with tier read from plus=", () => {
  const entries = extractRelic(RELIC_FIXTURE).filter((e) => e.tier === "1");
  assert.equal(entries.length, 5);
  const legs = entries.find((e) => e.slot === "legs")!;
  assert.equal(legs.itemName, "Duelist's Tights +1");
});

test("extracts the +2 tier correctly despite nested {{augment}} templates in its stats fields", () => {
  const entries = extractRelic(RELIC_FIXTURE).filter((e) => e.tier === "2");
  assert.equal(entries.length, 5);
  const feet = entries.find((e) => e.slot === "feet")!;
  assert.equal(feet.itemName, "Duelist's Boots +2");
});

test("returns 15 total entries (5 slots x 3 populated tiers)", () => {
  assert.equal(extractRelic(RELIC_FIXTURE).length, 15);
});

test("returns an empty array for wikitext with neither Relic template", () => {
  assert.deepEqual(extractRelic("no relic data here"), []);
});

test("strips wikilink syntax from a bracketed relic job= value", () => {
  const fixtureWithBracketedJob = `{{Armor Set Table
|Armor Set 1=
{{Relic Set
|relic job=[[Some Job]]
|relic head=Duelist's Chapeau
}}
}}`;
  const entries = extractRelic(fixtureWithBracketedJob);
  assert.equal(entries.length, 1);
  assert.equal(entries[0].job, "Some Job");
});
