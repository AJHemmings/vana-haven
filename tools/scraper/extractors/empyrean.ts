import { findTemplateBlocks, parseTemplateFields } from "../wikitext.ts";
import type { ExtractedEntry } from "../types.ts";

const SLOTS = ["head", "body", "hands", "legs", "feet"] as const;

export function extractEmpyrean(wikitext: string): ExtractedEntry[] {
  const results: ExtractedEntry[] = [];

  for (const block of findTemplateBlocks(wikitext, "R Empyrean Set 2")) {
    const fields = parseTemplateFields(block);
    const job = fields["jobs"];
    // Guards against a block missing jobs= entirely — parseTemplateFields returns
    // Record<string,string> (no noUncheckedIndexedAccess), so TypeScript won't catch
    // job being undefined at runtime. Without this, an undefined job would silently
    // corrupt discover.ts's missingJobs diff later. Same guard as af3.ts.
    if (!job) continue;
    const tier = fields["plus"]?.trim() ? fields["plus"].trim() : "0";

    for (const slot of SLOTS) {
      const itemName = fields[slot];
      if (!itemName) continue;
      results.push({ job, setType: "empyrean", slot, tier, itemName });
    }
  }

  return results;
}
