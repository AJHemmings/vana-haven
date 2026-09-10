export const JOBS: { id: number; abbreviation: string }[] = [
  { id: 1, abbreviation: "WAR" },
  { id: 2, abbreviation: "MNK" },
  { id: 3, abbreviation: "WHM" },
  { id: 4, abbreviation: "BLM" },
  { id: 5, abbreviation: "RDM" },
  { id: 6, abbreviation: "THF" },
  { id: 7, abbreviation: "PLD" },
  { id: 8, abbreviation: "DRK" },
  { id: 9, abbreviation: "BST" },
  { id: 10, abbreviation: "BRD" },
  { id: 11, abbreviation: "RNG" },
  { id: 12, abbreviation: "SAM" },
  { id: 13, abbreviation: "NIN" },
  { id: 14, abbreviation: "DRG" },
  { id: 15, abbreviation: "SMN" },
  { id: 16, abbreviation: "BLU" },
  { id: 17, abbreviation: "COR" },
  { id: 18, abbreviation: "PUP" },
  { id: 19, abbreviation: "DNC" },
  { id: 20, abbreviation: "SCH" },
  { id: 21, abbreviation: "GEO" },
  { id: 22, abbreviation: "RUN" },
];

export function jobAbbreviation(jobId: number): string {
  return JOBS.find((job) => job.id === jobId)?.abbreviation ?? `Job ${jobId}`;
}
