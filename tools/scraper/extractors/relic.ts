import { findTemplateBlocks, parseTemplateFields, stripWikiLink } from "../wikitext.ts";
import type { ExtractedEntry } from "../types.ts";

const RELIC_SLOT_FIELDS = ["relic head", "relic body", "relic hands", "relic legs", "relic feet"] as const;

function slotFromFieldName(fieldName: (typeof RELIC_SLOT_FIELDS)[number]): string {
  return fieldName.replace("relic ", "");
}

export function extractRelic(wikitext: string): ExtractedEntry[] {
  const results: ExtractedEntry[] = [];

  for (const block of findTemplateBlocks(wikitext, "Relic Set")) {
    const fields = parseTemplateFields(block);
    const rawJob = fields["relic job"];
    // Same job guard as af3.ts/empyrean.ts — see empyrean.ts's Task 5 comment for why.
    if (!rawJob) continue;
    const job = stripWikiLink(rawJob);

    for (const fieldName of RELIC_SLOT_FIELDS) {
      const rawItemName = fields[fieldName];
      if (!rawItemName) continue;
      const itemName = stripWikiLink(rawItemName);
      results.push({ job, setType: "relic", slot: slotFromFieldName(fieldName), tier: "0", itemName });
    }
  }

  for (const block of findTemplateBlocks(wikitext, "Relic + Set")) {
    const fields = parseTemplateFields(block);
    const rawJob = fields["relic job"];
    if (!rawJob) continue;
    const job = stripWikiLink(rawJob);
    const tier = fields["plus"]?.trim() ? fields["plus"].trim() : "0";

    for (const fieldName of RELIC_SLOT_FIELDS) {
      const rawItemName = fields[fieldName];
      if (!rawItemName) continue;
      const itemName = stripWikiLink(rawItemName);
      results.push({ job, setType: "relic", slot: slotFromFieldName(fieldName), tier, itemName });
    }
  }

  return results;
}
