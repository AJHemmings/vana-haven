import { findTemplateBlocks, parseTemplateFields, stripPipeTrick, stripWikiLink } from "../wikitext.ts";
import type { ExtractedEntry } from "../types.ts";

const SLOTS = ["head", "body", "hands", "legs", "feet"] as const;

export function extractAf3(wikitext: string): ExtractedEntry[] {
  const results: ExtractedEntry[] = [];

  for (const block of findTemplateBlocks(wikitext, "R Artifact Set 2")) {
    const fields = parseTemplateFields(block);
    const rawJob = fields["jobs"];
    if (!rawJob) continue;
    const job = stripPipeTrick(stripWikiLink(rawJob));
    const tier = fields["plus"]?.trim() ? fields["plus"].trim() : "0";

    for (const slot of SLOTS) {
      const rawItemName = fields[slot];
      if (!rawItemName) continue;
      const itemName = stripPipeTrick(stripWikiLink(rawItemName));
      results.push({ job, setType: "af3", slot, tier, itemName });
    }
  }

  return results;
}
