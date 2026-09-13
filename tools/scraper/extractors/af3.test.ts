import { test } from "node:test";
import assert from "node:assert/strict";
import { extractAf3 } from "./af3.ts";

const AF3_FIXTURE = `{{disambiguation|Warlock's Armor Set|Vitiation Armor Set|Lethargy Armor Set}}
{{Armor Set Table
|Image=Warlock's_Armor_Set.jpg
|Image Size=201
|Image Link=
|Armor Set 1=
{{R Artifact Set 2
|width=100%
|plus=
|jobs=Red Mage
|armor level=109
|set bonus=
|head=Atrophy Chapeau
|head base stats=[[Defense|DEF]]:69 [[HP]]+17
|body=Atrophy Tabard
|body base stats=[[Defense|DEF]]:90 [[HP]]+25
|hands=Atrophy Gloves
|hands base stats=[[Defense|DEF]]:60 [[HP]]+10
|legs=Atrophy Tights
|legs base stats=[[Defense|DEF]]:77 [[HP]]+20
|feet=Atrophy Boots
|feet base stats=[[Defense|DEF]]:47 [[HP]]+41
}}
|Armor Set 2=
{{R Artifact Set 2
|width=100%
|plus=1
|jobs=Red Mage
|armor level=119
|set bonus=
|head=Atrophy Chapeau +1
|head base stats=[[Defense|DEF]]:96 [[HP]]+36
|body=Atrophy Tabard +1
|body base stats=[[Defense|DEF]]:126 [[HP]]+54
|hands=Atrophy Gloves +1
|hands base stats=[[Defense|DEF]]:84 [[HP]]+22
|legs=Atrophy Tights +1
|legs base stats=[[Defense|DEF]]:108 [[HP]]+43
|feet=Atrophy Boots +1
|feet base stats=[[Defense|DEF]]:66 [[HP]]+48
}}
}}
==Notes==
{{Reforged Artifact Set Navigation}}`;

test("extracts all 5 slots for the base (NQ) tier, tier normalized to \"0\"", () => {
  const entries = extractAf3(AF3_FIXTURE).filter((e) => e.tier === "0");
  assert.equal(entries.length, 5);
  const legs = entries.find((e) => e.slot === "legs")!;
  assert.equal(legs.job, "Red Mage");
  assert.equal(legs.setType, "af3");
  assert.equal(legs.itemName, "Atrophy Tights");
});

test("extracts the +1 tier with tier read from the block's own plus= field", () => {
  const entries = extractAf3(AF3_FIXTURE).filter((e) => e.tier === "1");
  assert.equal(entries.length, 5);
  const head = entries.find((e) => e.slot === "head")!;
  assert.equal(head.itemName, "Atrophy Chapeau +1");
});

test("returns 10 total entries (5 slots x 2 tiers present in the fixture)", () => {
  assert.equal(extractAf3(AF3_FIXTURE).length, 10);
});

test("returns an empty array for wikitext with no R Artifact Set 2 template", () => {
  assert.deepEqual(extractAf3("no armor set data here"), []);
});
