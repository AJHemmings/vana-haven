import { useEffect, useState } from "react";
import { Link, useParams } from "react-router-dom";
import { fetchGearProgression, type GearProgression, type GearSetDefinitionRow } from "./bridge";
import { jobAbbreviation } from "./jobs";
import ItemPlaceholderIcon from "./ItemPlaceholderIcon";

const SET_TYPES = ["af3", "empyrean", "relic"] as const;
type SetType = (typeof SET_TYPES)[number];
const SET_TYPE_LABELS: Record<SetType, string> = { af3: "AF3", empyrean: "Empyrean", relic: "Relic" };
const SLOTS = ["head", "body", "hands", "legs", "feet"] as const;

type SlotState = "reached" | "current" | "not-obtained" | "unverifiable";

function tierState(row: GearSetDefinitionRow, currentTier: number | null): SlotState {
  if (row.item_id === null) return "unverifiable";
  if (currentTier === null) return "not-obtained";
  if (row.tier === currentTier) return "current";
  return row.tier < currentTier ? "reached" : "not-obtained";
}

export default function GearScreen() {
  const { gameCharacterId, jobId } = useParams<{ gameCharacterId: string; jobId: string }>();
  const id = Number(gameCharacterId);
  const job = Number(jobId);
  const [progression, setProgression] = useState<GearProgression | null>(null);
  const [manualSet, setManualSet] = useState<SetType | null>(null);

  useEffect(() => {
    fetchGearProgression(id, job)
      .then(setProgression)
      .catch((err) => console.error("[vana-haven] failed to fetch gear progression", err));
  }, [id, job]);

  const currentTierFor = (setType: SetType, slot: string): number | null =>
    progression?.current_tiers.find((t) => t.set_type === setType && t.slot === slot)?.current_tier ?? null;

  const rowsFor = (setType: SetType, slot: string) =>
    (progression?.definitions ?? [])
      .filter((d) => d.set_type === setType && d.slot === slot)
      .sort((a, b) => a.tier - b.tier);

  const hasAnyDefinitions = (setType: SetType) =>
    (progression?.definitions ?? []).some((d) => d.set_type === setType);

  // Default to AF3, but if this job has no AF3 rows at all (only relevant while
  // definitions are still loading, or for a hypothetical job missing AF3 entirely),
  // land on the first tab that actually has data instead of showing an empty state
  // the user never asked to see. A manual tab click always wins after that.
  const activeSet = manualSet ?? SET_TYPES.find(hasAnyDefinitions) ?? "af3";

  return (
    <main className="min-h-screen bg-neutral-950 text-neutral-100 p-6">
      <Link to={`/character/${id}/jobs`} className="text-neutral-400 hover:underline">
        ← back
      </Link>
      <h1 className="text-xl font-semibold mt-2">{jobAbbreviation(job)} Gear Sets</h1>

      <div role="tablist" className="flex gap-2 mt-4 mb-6">
        {SET_TYPES.map((setType) => (
          <button
            key={setType}
            role="tab"
            aria-selected={activeSet === setType}
            onClick={() => setManualSet(setType)}
            className={`px-3 py-1 rounded ${activeSet === setType ? "bg-neutral-700" : "bg-neutral-900"}`}
          >
            {SET_TYPE_LABELS[setType]}
          </button>
        ))}
      </div>

      {!hasAnyDefinitions(activeSet) ? (
        <p className="text-neutral-400">No {SET_TYPE_LABELS[activeSet]} armor exists for this job.</p>
      ) : (
        <ul className="divide-y divide-neutral-800">
          {SLOTS.map((slot) => {
            const rows = rowsFor(activeSet, slot);
            if (rows.length === 0) return null;
            const currentTier = currentTierFor(activeSet, slot);
            return (
              <li key={slot} className="py-3">
                <div className="text-neutral-400 text-sm mb-1 capitalize">{slot}</div>
                <div className="flex gap-3">
                  {rows.map((row) => {
                    const state = tierState(row, currentTier);
                    return (
                      <div
                        key={row.tier}
                        data-state={state}
                        title={state === "unverifiable" ? "BG-Wiki doesn't have this item's ID yet" : row.item_name}
                        className={
                          state === "current"
                            ? "ring-2 ring-emerald-400 rounded p-1"
                            : state === "reached"
                              ? "opacity-80 p-1"
                              : state === "unverifiable"
                                ? "opacity-40 grayscale p-1"
                                : "opacity-40 p-1"
                        }
                      >
                        <ItemPlaceholderIcon name={row.item_name} />
                        <div className="text-xs text-neutral-400 mt-1 max-w-[6rem] truncate">{row.item_name}</div>
                      </div>
                    );
                  })}
                </div>
              </li>
            );
          })}
        </ul>
      )}
    </main>
  );
}
