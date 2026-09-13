import type { CategoryId } from "../config.ts";
import type { Extractor } from "../types.ts";
import { extractAf3 } from "./af3.ts";

export const EXTRACTORS: Record<CategoryId, Extractor> = {
  af3: extractAf3,
  // empyrean and relic added in Tasks 5-6.
} as Record<CategoryId, Extractor>;
