import { render, screen, fireEvent, act } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import KeyItemsScreen from "./KeyItemsScreen";
import * as bridge from "./bridge";
import type { KeyItemTrackingRow } from "./bridge";

function renderAt(gameCharacterId: string) {
  return render(
    <MemoryRouter initialEntries={[`/character/${gameCharacterId}/key-items`]}>
      <Routes>
        <Route path="/character/:gameCharacterId/key-items" element={<KeyItemsScreen />} />
      </Routes>
    </MemoryRouter>
  );
}

const READY_UNTRACKED: KeyItemTrackingRow = {
  id: 1,
  key_item_id: 42,
  name: "Mystical Canteen",
  granting_npc: "Incantrix",
  cooldown_duration_seconds: 72000,
  last_acquired_at: null,
  currently_held: false,
};

const ON_COOLDOWN: KeyItemTrackingRow = {
  id: 2,
  key_item_id: 43,
  name: "Some Currency",
  granting_npc: null,
  cooldown_duration_seconds: 3600,
  last_acquired_at: "2026-09-15T00:00:00.000Z",
  currently_held: true,
};

const HELD_BEFORE_TRACKING: KeyItemTrackingRow = {
  id: 3,
  key_item_id: 44,
  name: "Old Key Item",
  granting_npc: null,
  cooldown_duration_seconds: 3600,
  last_acquired_at: null,
  currently_held: true,
};

describe("KeyItemsScreen", () => {
  beforeEach(() => {
    vi.spyOn(bridge, "onCharacterUpdated").mockResolvedValue(() => {});
    vi.spyOn(bridge, "onKeyItemCatalogUpdated").mockResolvedValue(() => {});
    vi.spyOn(bridge, "fetchKeyItemCatalog").mockResolvedValue([]);
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it("shows a loading message before data arrives", () => {
    vi.spyOn(bridge, "fetchKeyItemTracking").mockReturnValue(new Promise(() => {}));

    renderAt("1");

    expect(screen.getByText(/waiting for key item data/i)).toBeInTheDocument();
  });

  it("shows a distinct error message when the fetch fails", async () => {
    vi.spyOn(bridge, "fetchKeyItemTracking").mockRejectedValue(new Error("network error"));
    vi.spyOn(console, "error").mockImplementation(() => {});

    renderAt("1");

    expect(await screen.findByText(/failed to load key item data/i)).toBeInTheDocument();
  });

  it("shows an empty state when nothing is tracked yet", async () => {
    vi.spyOn(bridge, "fetchKeyItemTracking").mockResolvedValue([]);

    renderAt("1");

    expect(await screen.findByText(/no key items tracked yet/i)).toBeInTheDocument();
  });

  it("shows Ready for an item never held and never acquired", async () => {
    vi.spyOn(bridge, "fetchKeyItemTracking").mockResolvedValue([READY_UNTRACKED]);

    renderAt("1");

    expect(await screen.findByText("Mystical Canteen")).toBeInTheDocument();
    expect(screen.getByText("Ready").closest("[data-state]")).toHaveAttribute("data-state", "ready");
    expect(screen.getByText("(Incantrix)")).toBeInTheDocument();
  });

  it("shows a live countdown while on cooldown", async () => {
    vi.spyOn(bridge, "fetchKeyItemTracking").mockResolvedValue([ON_COOLDOWN]);
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-09-15T00:30:00.000Z")); // 30 min into a 1h cooldown

    renderAt("1");
    // RTL's findBy*/waitFor poll via setTimeout, which fake timers intercept
    // but never auto-advance — flush the mocked fetch's microtask directly
    // instead of relying on that polling.
    await act(async () => {
      await Promise.resolve();
    });

    expect(screen.getByText(/on cooldown/i).closest("[data-state]")).toHaveAttribute("data-state", "cooldown");
    expect(screen.getByText(/00:30:00/)).toBeInTheDocument(); // exactly 30 min remaining of the 1h cooldown

    act(() => {
      vi.advanceTimersByTime(10_000); // 10s later — the countdown ticks down, not re-fetched
    });

    expect(screen.getByText(/00:29:50/)).toBeInTheDocument();
  });

  it("shows Ready once the cooldown has fully elapsed", async () => {
    vi.spyOn(bridge, "fetchKeyItemTracking").mockResolvedValue([ON_COOLDOWN]);
    vi.useFakeTimers();
    vi.setSystemTime(new Date("2026-09-15T02:00:00.000Z")); // well past the 1h cooldown

    renderAt("1");
    await act(async () => {
      await Promise.resolve();
    });

    expect(screen.getByText("Ready").closest("[data-state]")).toHaveAttribute("data-state", "ready");
  });

  it("shows the honest unknown-timing state for an item held before tracking started", async () => {
    vi.spyOn(bridge, "fetchKeyItemTracking").mockResolvedValue([HELD_BEFORE_TRACKING]);

    renderAt("1");

    const text = await screen.findByText(/acquired before tracking started/i);
    expect(text.closest("[data-state]")).toHaveAttribute("data-state", "unknown");
  });

  it("shows a waiting-for-addon placeholder in the add form when the catalog is empty", async () => {
    vi.spyOn(bridge, "fetchKeyItemTracking").mockResolvedValue([]);

    renderAt("1");
    await screen.findByText(/no key items tracked yet/i);

    const input = screen.getByPlaceholderText(/waiting for the addon to connect/i);
    expect(input).toBeDisabled();
  });

  it("filters the catalog as the user types and adds a tracked definition on submit", async () => {
    vi.spyOn(bridge, "fetchKeyItemTracking").mockResolvedValue([]);
    vi.spyOn(bridge, "fetchKeyItemCatalog").mockResolvedValue([
      { key_item_id: 42, name: "Mystical Canteen" },
      { key_item_id: 99, name: "Rubber Cockatrice" },
    ]);
    const createSpy = vi.spyOn(bridge, "createKeyItemDefinition").mockResolvedValue(1);

    renderAt("1");
    await screen.findByText(/no key items tracked yet/i);

    const input = await screen.findByPlaceholderText(/search key items/i);
    fireEvent.change(input, { target: { value: "myst" } });

    const option = await screen.findByText("Mystical Canteen");
    expect(screen.queryByText("Rubber Cockatrice")).not.toBeInTheDocument();
    fireEvent.click(option);

    fireEvent.change(screen.getByLabelText(/cooldown duration$/i), { target: { value: "2" } });
    fireEvent.change(screen.getByLabelText(/cooldown duration unit/i), { target: { value: "days" } });
    fireEvent.change(screen.getByPlaceholderText(/granting npc/i), { target: { value: "Incantrix" } });

    fireEvent.click(screen.getByRole("button", { name: /track/i }));

    expect(createSpy).toHaveBeenCalledWith(42, "Mystical Canteen", "Incantrix", 2 * 86400);
  });

  it("stops tracking a key item on delete", async () => {
    vi.spyOn(bridge, "fetchKeyItemTracking").mockResolvedValue([READY_UNTRACKED]);
    const deleteSpy = vi.spyOn(bridge, "deleteKeyItemDefinition").mockResolvedValue(undefined);

    renderAt("1");
    await screen.findByText("Mystical Canteen");

    fireEvent.click(screen.getByRole("button", { name: /stop tracking mystical canteen/i }));

    expect(deleteSpy).toHaveBeenCalledWith(1);
  });

  it("links the key item name to its BG-Wiki page", async () => {
    vi.spyOn(bridge, "fetchKeyItemTracking").mockResolvedValue([READY_UNTRACKED]);

    renderAt("1");

    const link = await screen.findByText("Mystical Canteen");
    expect(link.closest("a")).toHaveAttribute("href", "https://www.bg-wiki.com/ffxi/Mystical_Canteen");
  });
});
