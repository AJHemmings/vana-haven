import { findTemplateBlocks, parseTemplateFields } from "../wikitext.ts";
import type { ExtractedEntry } from "../types.ts";

const SLOTS = ["head", "body", "hands", "legs", "feet"] as const;

export function extractAf3(wikitext: string): ExtractedEntry[] {
  const results: ExtractedEntry[] = [];

  for (const block of findTemplateBlocks(wikitext, "R Artifact Set 2")) {
    const fields = parseTemplateFields(block);
    const job = fields["jobs"];
    if (!job) continue;
    const tier = fields["plus"]?.trim() ? fields["plus"].trim() : "0";

    for (const slot of SLOTS) {
      const itemName = fields[slot];
      if (!itemName) continue;
      results.push({ job, setType: "af3", slot, tier, itemName });
    }
  }

  return results;
}
