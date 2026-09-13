export type CategoryId = "af3" | "empyrean" | "relic";

export type CategoryConfig = {
  id: CategoryId;
  seedCategory: string;
};

// Mirrors src/jobs.ts's JOBS order (Windower's canonical job ID order) — kept as a
// separate list because BG-Wiki wikitext uses full job names ("Red Mage"), not
// abbreviations ("RDM"). Whoever writes the later gearsets-JSON -> GearSetDefinition
// loader (spec §2) needs this exact name -> Windower job code mapping.
export const CANONICAL_JOBS: string[] = [
  "Warrior",
  "Monk",
  "White Mage",
  "Black Mage",
  "Red Mage",
  "Thief",
  "Paladin",
  "Dark Knight",
  "Beastmaster",
  "Bard",
  "Ranger",
  "Samurai",
  "Ninja",
  "Dragoon",
  "Summoner",
  "Blue Mage",
  "Corsair",
  "Puppetmaster",
  "Dancer",
  "Scholar",
  "Geomancer",
  "Rune Fencer",
];

export const CATEGORIES: Record<CategoryId, CategoryConfig> = {
  af3: { id: "af3", seedCategory: "Reforged Artifact Armor +3" },
  empyrean: { id: "empyrean", seedCategory: "Reforged Empyrean Armor +3" },
  // Confirmed live: only 19/22 jobs' set-overview pages carry this category (not
  // 22/22 like af3/empyrean). Geomancer and Rune Fencer never had Relic armor —
  // added after the Relic era — so their absence is correct, not missing coverage.
  // Red Mage's set page ("Duelist's Attire Set") exists with the right template
  // shape but isn't tagged into this category on BG-Wiki itself — a wiki
  // miscategorization to work around manually if/when Red Mage's relic set is
  // needed, not a bug in this tool. Expect discover.ts's missingJobs for "relic"
  // to list exactly these 3 on a real run.
  relic: { id: "relic", seedCategory: "Relic Armor" },
};

export const API_BASE = "https://www.bg-wiki.com/api.php";
export const OUTPUT_DIR = "tools/scraper/out";
export const CACHE_DIR = "tools/scraper/.cache";
export const REQUEST_DELAY_MS = 500;
export const BATCH_SIZE = 50;
// A real identifying contact URL, per spec §6 — BG-Wiki's admins can see who's
// hitting their API and why if this ever needs following up on.
export const USER_AGENT = "VanaHaven-Scraper/0.1 (contact: https://github.com/AJHemmings)";
