import { useEffect, useMemo, useState, type FormEvent } from "react";
import { Link, useParams } from "react-router-dom";
import {
  createKeyItemDefinition,
  deleteKeyItemDefinition,
  fetchKeyItemCatalog,
  fetchKeyItemTracking,
  onCharacterUpdated,
  onKeyItemCatalogUpdated,
  type KeyItemCatalogEntry,
  type KeyItemTrackingRow,
} from "./bridge";

const DURATION_UNIT_SECONDS = { hours: 3600, days: 86400 } as const;
type DurationUnit = keyof typeof DURATION_UNIT_SECONDS;

// Consistent with the main spec's §9: the key item's name links to BG-Wiki
// via the same generated-URL pattern, not a hand-maintained mapping.
function bgWikiUrl(name: string): string {
  return `https://www.bg-wiki.com/ffxi/${name.replace(/ /g, "_")}`;
}

function formatCountdown(remainingMs: number): string {
  const totalSeconds = Math.max(0, Math.floor(remainingMs / 1000));
  const days = Math.floor(totalSeconds / 86400);
  const hours = Math.floor((totalSeconds % 86400) / 3600);
  const minutes = Math.floor((totalSeconds % 3600) / 60);
  const seconds = totalSeconds % 60;
  const pad = (n: number) => String(n).padStart(2, "0");
  const clock = `${pad(hours)}:${pad(minutes)}:${pad(seconds)}`;
  return days > 0 ? `${days}d ${clock}` : clock;
}

type Status = { kind: "ready" } | { kind: "cooldown"; availableAt: number } | { kind: "unknown" };

// Three honest states, not a guess — see the Key Item Cooldowns spec §7.
function computeStatus(row: KeyItemTrackingRow, now: number): Status {
  if (row.last_acquired_at !== null) {
    const availableAt = new Date(row.last_acquired_at).getTime() + row.cooldown_duration_seconds * 1000;
    return now >= availableAt ? { kind: "ready" } : { kind: "cooldown", availableAt };
  }
  return row.currently_held ? { kind: "unknown" } : { kind: "ready" };
}

export default function KeyItemsScreen() {
  const { gameCharacterId } = useParams<{ gameCharacterId: string }>();
  const id = Number(gameCharacterId);

  const [tracking, setTracking] = useState<KeyItemTrackingRow[] | null>(null);
  const [catalog, setCatalog] = useState<KeyItemCatalogEntry[]>([]);
  const [fetchError, setFetchError] = useState(false);
  const [now, setNow] = useState(() => Date.now());

  const [search, setSearch] = useState("");
  const [selected, setSelected] = useState<KeyItemCatalogEntry | null>(null);
  const [durationValue, setDurationValue] = useState("20");
  const [durationUnit, setDurationUnit] = useState<DurationUnit>("hours");
  const [grantingNpc, setGrantingNpc] = useState("");
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    let unlistenCharacter: (() => void) | undefined;
    let unlistenCatalog: (() => void) | undefined;

    const loadTracking = () => {
      setFetchError(false);
      fetchKeyItemTracking(id)
        .then(setTracking)
        .catch((err) => {
          console.error("[vana-haven] failed to fetch key item tracking", err);
          setFetchError(true);
        });
    };
    const loadCatalog = () => {
      fetchKeyItemCatalog()
        .then(setCatalog)
        .catch((err) => console.error("[vana-haven] failed to fetch key item catalog", err));
    };

    loadTracking();
    loadCatalog();
    onCharacterUpdated(loadTracking).then((fn) => {
      unlistenCharacter = fn;
    });
    onKeyItemCatalogUpdated(loadCatalog).then((fn) => {
      unlistenCatalog = fn;
    });

    return () => {
      unlistenCharacter?.();
      unlistenCatalog?.();
    };
  }, [id]);

  // Live countdown — computed client-side from each row's fixed
  // last_acquired_at + cooldown_duration_seconds on a local tick, not
  // re-fetched from the backend every second (spec §7).
  useEffect(() => {
    const interval = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(interval);
  }, []);

  const matches = useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return [];
    return catalog.filter((entry) => entry.name.toLowerCase().includes(q)).slice(0, 20);
  }, [search, catalog]);

  const resetForm = () => {
    setSelected(null);
    setSearch("");
    setGrantingNpc("");
    setDurationValue("20");
    setDurationUnit("hours");
  };

  const handleAdd = (e: FormEvent) => {
    e.preventDefault();
    if (!selected) return;
    const seconds = Number(durationValue) * DURATION_UNIT_SECONDS[durationUnit];
    if (!(seconds > 0)) return;

    setSubmitting(true);
    createKeyItemDefinition(selected.key_item_id, selected.name, grantingNpc.trim() || null, seconds)
      .then(() => {
        resetForm();
        return fetchKeyItemTracking(id).then(setTracking);
      })
      .catch((err) => console.error("[vana-haven] failed to create key item definition", err))
      .finally(() => setSubmitting(false));
  };

  const handleDelete = (definitionId: number) => {
    deleteKeyItemDefinition(definitionId)
      .then(() => fetchKeyItemTracking(id).then(setTracking))
      .catch((err) => console.error("[vana-haven] failed to delete key item definition", err));
  };

  return (
    <main className="min-h-screen bg-neutral-950 text-neutral-100 p-6">
      <Link to={`/character/${id}`} className="text-neutral-400 hover:underline">
        ← back
      </Link>
      <h1 className="text-xl font-semibold mt-2">Key Items</h1>

      <form onSubmit={handleAdd} className="mt-4 mb-6 flex flex-col gap-2 max-w-md">
        <div className="relative">
          <input
            type="text"
            placeholder={catalog.length === 0 ? "waiting for the addon to connect..." : "search key items..."}
            disabled={catalog.length === 0}
            value={selected ? selected.name : search}
            onChange={(e) => {
              setSelected(null);
              setSearch(e.target.value);
            }}
            className="w-full px-2 py-1 rounded bg-neutral-900 border border-neutral-700 disabled:opacity-40"
          />
          {!selected && matches.length > 0 && (
            <ul className="absolute z-10 w-full bg-neutral-900 border border-neutral-700 rounded mt-1 max-h-48 overflow-auto">
              {matches.map((entry) => (
                <li key={entry.key_item_id}>
                  <button
                    type="button"
                    onClick={() => {
                      setSelected(entry);
                      setSearch(entry.name);
                    }}
                    className="w-full text-left px-2 py-1 hover:bg-neutral-800"
                  >
                    {entry.name}
                  </button>
                </li>
              ))}
            </ul>
          )}
        </div>
        <div className="flex gap-2">
          <input
            type="number"
            min="1"
            value={durationValue}
            onChange={(e) => setDurationValue(e.target.value)}
            aria-label="cooldown duration"
            className="w-24 px-2 py-1 rounded bg-neutral-900 border border-neutral-700"
          />
          <select
            value={durationUnit}
            onChange={(e) => setDurationUnit(e.target.value as DurationUnit)}
            aria-label="cooldown duration unit"
            className="px-2 py-1 rounded bg-neutral-900 border border-neutral-700"
          >
            <option value="hours">hours</option>
            <option value="days">days</option>
          </select>
        </div>
        <input
          type="text"
          placeholder="granting NPC (optional)"
          value={grantingNpc}
          onChange={(e) => setGrantingNpc(e.target.value)}
          className="px-2 py-1 rounded bg-neutral-900 border border-neutral-700"
        />
        <button
          type="submit"
          disabled={!selected || submitting}
          className="px-3 py-1.5 rounded bg-neutral-800 hover:bg-neutral-700 disabled:opacity-40"
        >
          Track
        </button>
      </form>

      {fetchError ? (
        <p className="text-neutral-400">failed to load key item data</p>
      ) : tracking === null ? (
        <p className="text-neutral-400">waiting for key item data...</p>
      ) : tracking.length === 0 ? (
        <p className="text-neutral-400">No key items tracked yet.</p>
      ) : (
        <ul className="divide-y divide-neutral-800">
          {tracking.map((row) => {
            const status = computeStatus(row, now);
            return (
              <li key={row.id} className="py-3 flex justify-between items-center">
                <div>
                  <a href={bgWikiUrl(row.name)} target="_blank" rel="noreferrer" className="hover:underline">
                    {row.name}
                  </a>
                  {row.granting_npc && <span className="text-neutral-400 text-sm ml-2">({row.granting_npc})</span>}
                  <div data-state={status.kind} className="text-sm mt-0.5">
                    {status.kind === "ready" && <span className="text-emerald-400">Ready</span>}
                    {status.kind === "cooldown" && (
                      <span className="text-neutral-400">On cooldown — {formatCountdown(status.availableAt - now)}</span>
                    )}
                    {status.kind === "unknown" && (
                      <span className="text-neutral-500">Held — acquired before tracking started</span>
                    )}
                  </div>
                </div>
                <button
                  onClick={() => handleDelete(row.id)}
                  aria-label={`stop tracking ${row.name}`}
                  className="text-neutral-500 hover:text-red-400 px-2"
                >
                  ✕
                </button>
              </li>
            );
          })}
        </ul>
      )}
    </main>
  );
}
