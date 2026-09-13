import type { CategoryId } from "./config.ts";

export type ExtractedEntry = {
  job: string;
  setType: CategoryId;
  slot: string;
  tier: string;
  itemName: string;
};

export type FinalEntry = ExtractedEntry & { itemId: number | null };

export type ManifestMatch = { job: string; setType: CategoryId; setPageTitle: string };
export type ManifestUnmatched = { pageTitle: string; reason: string };
export type DiscoveryManifest = {
  matched: ManifestMatch[];
  unmatched: ManifestUnmatched[];
  missingJobs: string[];
};

export type Extractor = (wikitext: string) => ExtractedEntry[];
