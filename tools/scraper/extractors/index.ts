import type { CategoryId } from "../config.ts";
import type { Extractor } from "../types.ts";
import { extractAf3 } from "./af3.ts";
import { extractEmpyrean } from "./empyrean.ts";

export const EXTRACTORS: Record<CategoryId, Extractor> = {
  af3: extractAf3,
  empyrean: extractEmpyrean,
  // relic added in Task 6.
} as Record<CategoryId, Extractor>;
