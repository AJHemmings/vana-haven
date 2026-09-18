import { render, screen, fireEvent, act } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import DailiesScreen from "./DailiesScreen";
import * as bridge from "./bridge";
import type { DailyTodoItem } from "./bridge";

function renderAt(gameCharacterId: string) {
  return render(
    <MemoryRouter initialEntries={[`/character/${gameCharacterId}/dailies`]}>
      <Routes>
        <Route path="/character/:gameCharacterId/dailies" element={<DailiesScreen />} />
      </Routes>
    </MemoryRouter>
  );
}

const DAILY_DONE: DailyTodoItem = {
  id: 1,
  text: "Sortie run",
  cadence: "daily",
  last_completed_at: "2026-09-17T16:00:00.000Z", // after the 15:00 UTC boundary
};

const DAILY_NOT_DONE: DailyTodoItem = {
  id: 2,
  text: "Escha zone",
  cadence: "daily",
  last_completed_at: "2026-09-16T20:00:00.000Z", // before the boundary
};

const MONTHLY_ITEM: DailyTodoItem = {
  id: 3,
  text: "Ambuscade prep",
  cadence: "monthly",
  last_completed_at: null,
};

describe("DailiesScreen", () => {
  beforeEach(() => {
    vi.spyOn(bridge, "fetchMonthlyCycleStartedAt").mockResolvedValue(null);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("shows a loading message before data arrives", () => {
    vi.spyOn(bridge, "fetchDailyTodoItems").mockReturnValue(new Promise(() => {}));

    renderAt("1");

    expect(screen.getByText(/waiting for daily todo data/i)).toBeInTheDocument();
  });

  it("shows a distinct error message when the fetch fails", async () => {
    vi.spyOn(bridge, "fetchDailyTodoItems").mockRejectedValue(new Error("network error"));
    vi.spyOn(console, "error").mockImplementation(() => {});

    renderAt("1");

    expect(await screen.findByText(/failed to load daily todo data/i)).toBeInTheDocument();
  });

  it("shows an empty state when nothing is tracked yet", async () => {
    vi.spyOn(bridge, "fetchDailyTodoItems").mockResolvedValue([]);

    renderAt("1");

    expect(await screen.findByText(/no todo items yet/i)).toBeInTheDocument();
  });

  it("checks off an item completed within the current daily period", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-09-17T20:00:00.000Z"));
    vi.spyOn(bridge, "fetchDailyTodoItems").mockResolvedValue([DAILY_DONE]);

    renderAt("1");
    await act(async () => {
      await Promise.resolve();
    });

    expect(screen.getByRole("checkbox")).toBeChecked();
  });

  it("leaves an item unchecked when its last completion is before the current boundary", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-09-17T20:00:00.000Z"));
    vi.spyOn(bridge, "fetchDailyTodoItems").mockResolvedValue([DAILY_NOT_DONE]);

    renderAt("1");
    await act(async () => {
      await Promise.resolve();
    });

    expect(screen.getByRole("checkbox")).not.toBeChecked();
  });

  it("adds a new todo item with the selected cadence", async () => {
    vi.spyOn(bridge, "fetchDailyTodoItems").mockResolvedValue([]);
    const createSpy = vi.spyOn(bridge, "createDailyTodoItem").mockResolvedValue(1);

    renderAt("1");
    await screen.findByText(/no todo items yet/i);

    fireEvent.change(screen.getByPlaceholderText(/add a todo item/i), { target: { value: "Ambuscade" } });
    fireEvent.change(screen.getByLabelText(/reset cadence/i), { target: { value: "weekly" } });
    fireEvent.click(screen.getByRole("button", { name: /add/i }));

    expect(createSpy).toHaveBeenCalledWith(1, "Ambuscade", "weekly");
  });

  it("toggles completion when the checkbox is clicked", async () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-09-17T20:00:00.000Z"));
    vi.spyOn(bridge, "fetchDailyTodoItems").mockResolvedValue([DAILY_NOT_DONE]);
    const toggleSpy = vi.spyOn(bridge, "setDailyTodoCompleted").mockResolvedValue(undefined);

    renderAt("1");
    await act(async () => {
      await Promise.resolve();
    });

    fireEvent.click(screen.getByRole("checkbox"));

    expect(toggleSpy).toHaveBeenCalledWith(2, true);
  });

  it("removes an item on delete", async () => {
    vi.spyOn(bridge, "fetchDailyTodoItems").mockResolvedValue([DAILY_NOT_DONE]);
    const deleteSpy = vi.spyOn(bridge, "deleteDailyTodoItem").mockResolvedValue(undefined);

    renderAt("1");
    await screen.findByText("Escha zone");

    fireEvent.click(screen.getByRole("button", { name: /remove escha zone/i }));

    expect(deleteSpy).toHaveBeenCalledWith(2);
  });

  it("hides the version-update action when there are no monthly items", async () => {
    vi.spyOn(bridge, "fetchDailyTodoItems").mockResolvedValue([DAILY_NOT_DONE]);

    renderAt("1");
    await screen.findByText("Escha zone");

    expect(screen.queryByRole("button", { name: /version update happened/i })).not.toBeInTheDocument();
  });

  it("advances the monthly cycle marker when the version-update action is used", async () => {
    vi.spyOn(bridge, "fetchDailyTodoItems").mockResolvedValue([MONTHLY_ITEM]);
    const advanceSpy = vi.spyOn(bridge, "advanceMonthlyCycle").mockResolvedValue("2026-09-17T20:00:00.000Z");

    renderAt("1");
    await screen.findByText("Ambuscade prep");

    fireEvent.click(screen.getByRole("button", { name: /version update happened/i }));

    expect(advanceSpy).toHaveBeenCalled();
  });
});
