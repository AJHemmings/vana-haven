import { useEffect, useState, type FormEvent } from "react";
import { Link, useParams } from "react-router-dom";
import {
  advanceMonthlyCycle,
  createDailyTodoItem,
  deleteDailyTodoItem,
  fetchDailyTodoItems,
  fetchMonthlyCycleStartedAt,
  setDailyTodoCompleted,
  type DailyTodoItem,
} from "./bridge";
import { isDoneForCurrentPeriod } from "./dailies";

const CADENCE_LABEL: Record<DailyTodoItem["cadence"], string> = {
  daily: "Daily",
  weekly: "Weekly",
  monthly: "Monthly",
};

export default function DailiesScreen() {
  const { gameCharacterId } = useParams<{ gameCharacterId: string }>();
  const id = Number(gameCharacterId);

  const [items, setItems] = useState<DailyTodoItem[] | null>(null);
  const [fetchError, setFetchError] = useState(false);
  const [monthlyCycleStartedAt, setMonthlyCycleStartedAt] = useState<string | null>(null);
  const [now, setNow] = useState(() => Date.now());

  const [text, setText] = useState("");
  const [cadence, setCadence] = useState<DailyTodoItem["cadence"]>("daily");
  const [submitting, setSubmitting] = useState(false);

  const loadItems = () => {
    setFetchError(false);
    fetchDailyTodoItems(id)
      .then(setItems)
      .catch((err) => {
        console.error("[vana-haven] failed to fetch daily todo items", err);
        setFetchError(true);
      });
  };

  useEffect(() => {
    loadItems();
    fetchMonthlyCycleStartedAt()
      .then(setMonthlyCycleStartedAt)
      .catch((err) => console.error("[vana-haven] failed to fetch monthly cycle marker", err));
  }, [id]);

  // Boundaries only move forward at fixed instants (15:00 UTC etc.), not
  // continuously like the Key Items countdown — a coarse tick is enough to
  // notice a crossing while the screen stays open.
  useEffect(() => {
    const interval = setInterval(() => setNow(Date.now()), 30_000);
    return () => clearInterval(interval);
  }, []);

  const handleAdd = (e: FormEvent) => {
    e.preventDefault();
    const trimmed = text.trim();
    if (!trimmed) return;

    setSubmitting(true);
    createDailyTodoItem(id, trimmed, cadence)
      .then(() => {
        setText("");
        loadItems();
      })
      .catch((err) => console.error("[vana-haven] failed to create daily todo item", err))
      .finally(() => setSubmitting(false));
  };

  const handleToggle = (item: DailyTodoItem, done: boolean) => {
    setDailyTodoCompleted(item.id, !done)
      .then(loadItems)
      .catch((err) => console.error("[vana-haven] failed to update daily todo item", err));
  };

  const handleDelete = (itemId: number) => {
    deleteDailyTodoItem(itemId)
      .then(loadItems)
      .catch((err) => console.error("[vana-haven] failed to delete daily todo item", err));
  };

  const handleVersionUpdate = () => {
    advanceMonthlyCycle()
      .then(setMonthlyCycleStartedAt)
      .catch((err) => console.error("[vana-haven] failed to advance monthly cycle", err));
  };

  const hasMonthlyItems = (items ?? []).some((item) => item.cadence === "monthly");

  return (
    <main className="min-h-screen bg-neutral-950 text-neutral-100 p-6">
      <Link to={`/character/${id}`} className="text-neutral-400 hover:underline">
        ← back
      </Link>
      <h1 className="text-xl font-semibold mt-2">Dailies</h1>

      <form onSubmit={handleAdd} className="mt-4 mb-6 flex flex-col gap-2 max-w-md">
        <input
          type="text"
          placeholder="add a todo item..."
          value={text}
          onChange={(e) => setText(e.target.value)}
          className="w-full px-2 py-1 rounded bg-neutral-900 border border-neutral-700"
        />
        <div className="flex gap-2">
          <select
            value={cadence}
            onChange={(e) => setCadence(e.target.value as DailyTodoItem["cadence"])}
            aria-label="reset cadence"
            className="px-2 py-1 rounded bg-neutral-900 border border-neutral-700"
          >
            <option value="daily">Daily</option>
            <option value="weekly">Weekly</option>
            <option value="monthly">Monthly</option>
          </select>
          <button
            type="submit"
            disabled={!text.trim() || submitting}
            className="px-3 py-1.5 rounded bg-neutral-800 hover:bg-neutral-700 disabled:opacity-40"
          >
            Add
          </button>
        </div>
      </form>

      {hasMonthlyItems && (
        <button
          onClick={handleVersionUpdate}
          className="mb-4 px-3 py-1.5 rounded bg-neutral-800 hover:bg-neutral-700 text-sm"
        >
          Version update happened — reset monthly items
        </button>
      )}

      {fetchError ? (
        <p className="text-neutral-400">failed to load daily todo data</p>
      ) : items === null ? (
        <p className="text-neutral-400">waiting for daily todo data...</p>
      ) : items.length === 0 ? (
        <p className="text-neutral-400">No todo items yet.</p>
      ) : (
        <ul className="divide-y divide-neutral-800">
          {items.map((item) => {
            const done = isDoneForCurrentPeriod(item.cadence, item.last_completed_at, new Date(now), monthlyCycleStartedAt);
            return (
              <li key={item.id} className="py-3 flex justify-between items-center">
                <label className="flex items-center gap-2">
                  <input type="checkbox" checked={done} onChange={() => handleToggle(item, done)} />
                  <span className={done ? "line-through text-neutral-500" : undefined}>{item.text}</span>
                  <span className="text-neutral-400 text-sm">({CADENCE_LABEL[item.cadence]})</span>
                </label>
                <button
                  onClick={() => handleDelete(item.id)}
                  aria-label={`remove ${item.text}`}
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
