import { test } from "node:test";
import assert from "node:assert/strict";
import { extractEmpyrean } from "./empyrean.ts";

const EMPYREAN_FIXTURE = `{{disambiguation|Reverence Armor Set|Caballarius Armor Set|Creed Armor Set}}
{{Armor Set Table
|Image=Creed Armor Set-fix.jpg
|Image Size=201
|Image Link=
|Armor Set 1=
{{R Empyrean Set 2
|width=100%
|plus=
|jobs=Paladin
|armor level=109
|set bonus=Occasionally absorbs [[damage taken]]
|head=Chevalier's Armet
|head base stats=[[Defense|DEF]]:85 [[HP]]+81
|body=Chevalier's Cuirass
|body base stats=[[Defense|DEF]]:109 [[HP]]+96
|hands=Chevalier's Gauntlets
|hands base stats=[[Defense|DEF]]:77 [[HP]]+16
|legs=Chevalier's Cuisses
|legs base stats=[[Defense|DEF]]:97 [[HP]]+77
|feet=Chevalier's Sabatons
|feet base stats=[[Defense|DEF]]:67 [[HP]]+10
}}
|Armor Set 2=
{{R Empyrean Set 2
|width=100%
|plus=1
|jobs=Paladin
|armor level=119
|set bonus=Occasionally absorbs [[damage taken]]
|head=Chevalier's Armet +1
|head base stats=[[Defense|DEF]]:118 [[HP]]+125
|body=Chevalier's Cuirass +1
|body base stats=[[Defense|DEF]]:150 [[HP]]+131
|hands=Chevalier's Gauntlets +1
|hands base stats=[[Defense|DEF]]:106 [[HP]]+34
|legs=Chevalier's Cuisses +1
|legs base stats=[[Defense|DEF]]:130 [[HP]]+107
|feet=Chevalier's Sabatons +1
|feet base stats=[[Defense|DEF]]:91 [[HP]]+22
}}
}}
== Notes ==
{{Reforged Empyrean Set Navigation}}`;

test("extracts all 5 slots for the base (NQ) tier, tier normalized to \"0\"", () => {
  const entries = extractEmpyrean(EMPYREAN_FIXTURE).filter((e) => e.tier === "0");
  assert.equal(entries.length, 5);
  const head = entries.find((e) => e.slot === "head")!;
  assert.equal(head.job, "Paladin");
  assert.equal(head.setType, "empyrean");
  assert.equal(head.itemName, "Chevalier's Armet");
});

test("extracts the +1 tier with tier read from the block's own plus= field", () => {
  const entries = extractEmpyrean(EMPYREAN_FIXTURE).filter((e) => e.tier === "1");
  assert.equal(entries.length, 5);
  const feet = entries.find((e) => e.slot === "feet")!;
  assert.equal(feet.itemName, "Chevalier's Sabatons +1");
});

test("returns an empty array for wikitext with no R Empyrean Set 2 template", () => {
  assert.deepEqual(extractEmpyrean("no armor set data here"), []);
});
