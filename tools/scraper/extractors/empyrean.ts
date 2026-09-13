import { findTemplateBlocks, parseTemplateFields, stripWikiLink } from "../wikitext.ts";
import type { ExtractedEntry } from "../types.ts";

const SLOTS = ["head", "body", "hands", "legs", "feet"] as const;

export function extractEmpyrean(wikitext: string): ExtractedEntry[] {
  const results: ExtractedEntry[] = [];

  for (const block of findTemplateBlocks(wikitext, "R Empyrean Set 2")) {
    const fields = parseTemplateFields(block);
    const rawJob = fields["jobs"];
    // Guards against a block missing jobs= entirely — parseTemplateFields returns
    // Record<string,string> (no noUncheckedIndexedAccess), so TypeScript won't catch
    // job being undefined at runtime. Without this, an undefined job would silently
    // corrupt discover.ts's missingJobs diff later. Same guard as af3.ts.
    if (!rawJob) continue;
    const job = stripWikiLink(rawJob);
    const tier = fields["plus"]?.trim() ? fields["plus"].trim() : "0";

    for (const slot of SLOTS) {
      const rawItemName = fields[slot];
      if (!rawItemName) continue;
      const itemName = stripWikiLink(rawItemName);
      results.push({ job, setType: "empyrean", slot, tier, itemName });
    }
  }

  return results;
}
