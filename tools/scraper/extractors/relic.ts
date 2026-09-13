import { findTemplateBlocks, parseTemplateFields } from "../wikitext.ts";
import type { ExtractedEntry } from "../types.ts";

const RELIC_SLOT_FIELDS = ["relic head", "relic body", "relic hands", "relic legs", "relic feet"] as const;

function slotFromFieldName(fieldName: (typeof RELIC_SLOT_FIELDS)[number]): string {
  return fieldName.replace("relic ", "");
}

export function extractRelic(wikitext: string): ExtractedEntry[] {
  const results: ExtractedEntry[] = [];

  for (const block of findTemplateBlocks(wikitext, "Relic Set")) {
    const fields = parseTemplateFields(block);
    const job = fields["relic job"];
    // Same job guard as af3.ts/empyrean.ts — see empyrean.ts's Task 5 comment for why.
    if (!job) continue;

    for (const fieldName of RELIC_SLOT_FIELDS) {
      const itemName = fields[fieldName];
      if (!itemName) continue;
      results.push({ job, setType: "relic", slot: slotFromFieldName(fieldName), tier: "0", itemName });
    }
  }

  for (const block of findTemplateBlocks(wikitext, "Relic + Set")) {
    const fields = parseTemplateFields(block);
    const job = fields["relic job"];
    if (!job) continue;
    const tier = fields["plus"]?.trim() ? fields["plus"].trim() : "0";

    for (const fieldName of RELIC_SLOT_FIELDS) {
      const itemName = fields[fieldName];
      if (!itemName) continue;
      results.push({ job, setType: "relic", slot: slotFromFieldName(fieldName), tier, itemName });
    }
  }

  return results;
}
