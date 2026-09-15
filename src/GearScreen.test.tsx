import { render, screen, fireEvent } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import GearScreen from "./GearScreen";
import * as bridge from "./bridge";

function renderAt(gameCharacterId: string, jobId: string) {
  return render(
    <MemoryRouter initialEntries={[`/character/${gameCharacterId}/jobs/${jobId}/gear`]}>
      <Routes>
        <Route path="/character/:gameCharacterId/jobs/:jobId/gear" element={<GearScreen />} />
      </Routes>
    </MemoryRouter>
  );
}

const AF3_HEAD = { job_id: 20, set_type: "af3" as const, slot: "head" as const, tier: 0, item_name: "Academic's Mortarboard", item_id: 27683 };
const AF3_HEAD_T1 = { ...AF3_HEAD, tier: 1, item_name: "Academic's Mortarboard +1" };
const RELIC_HEAD_UNRESOLVED = { job_id: 20, set_type: "relic" as const, slot: "head" as const, tier: 2, item_name: "Argute Mortarboard +2", item_id: null };

describe("GearScreen", () => {
  beforeEach(() => {
    vi.spyOn(bridge, "onCharacterUpdated").mockResolvedValue(() => {});
  });

  it("shows not-obtained for every slot when nothing is held", async () => {
    vi.spyOn(bridge, "fetchGearProgression").mockResolvedValue({
      definitions: [AF3_HEAD],
      current_tiers: [{ set_type: "af3", slot: "head", current_tier: null }],
    });

    renderAt("1", "20");

    expect(await screen.findByText(AF3_HEAD.item_name)).toBeInTheDocument();
    expect(screen.getByText(AF3_HEAD.item_name).closest('[data-state]')).toHaveAttribute("data-state", "not-obtained");
  });

  it("highlights the character's current tier", async () => {
    vi.spyOn(bridge, "fetchGearProgression").mockResolvedValue({
      definitions: [AF3_HEAD, AF3_HEAD_T1],
      current_tiers: [{ set_type: "af3", slot: "head", current_tier: 1 }],
    });

    renderAt("1", "20");

    await screen.findByText(AF3_HEAD_T1.item_name);
    expect(screen.getByText(AF3_HEAD_T1.item_name).closest('[data-state]')).toHaveAttribute("data-state", "current");
    expect(screen.getByText(AF3_HEAD.item_name).closest('[data-state]')).toHaveAttribute("data-state", "reached");
  });

  it("shows a distinct can't-verify state for null-item_id rows regardless of tier", async () => {
    vi.spyOn(bridge, "fetchGearProgression").mockResolvedValue({
      definitions: [RELIC_HEAD_UNRESOLVED],
      current_tiers: [{ set_type: "relic", slot: "head", current_tier: null }],
    });

    renderAt("1", "20");

    await screen.findByText(RELIC_HEAD_UNRESOLVED.item_name);
    expect(screen.getByText(RELIC_HEAD_UNRESOLVED.item_name).closest('[data-state]')).toHaveAttribute("data-state", "unverifiable");
  });

  it("shows an empty state for a set type with no definitions for this job", async () => {
    vi.spyOn(bridge, "fetchGearProgression").mockResolvedValue({
      definitions: [AF3_HEAD], // no relic rows at all — e.g. Geomancer
      current_tiers: [{ set_type: "af3", slot: "head", current_tier: null }],
    });

    renderAt("1", "21"); // Geomancer

    await screen.findByText(AF3_HEAD.item_name);
    fireEvent.click(screen.getByRole("tab", { name: /relic/i }));

    expect(screen.getByText(/no relic armor exists for this job/i)).toBeInTheDocument();
  });

  it("switches tabs between af3, empyrean, and relic", async () => {
    vi.spyOn(bridge, "fetchGearProgression").mockResolvedValue({
      definitions: [AF3_HEAD],
      current_tiers: [{ set_type: "af3", slot: "head", current_tier: null }],
    });

    renderAt("1", "20");

    await screen.findByText(AF3_HEAD.item_name);
    expect(screen.getByText(AF3_HEAD.item_name)).toBeVisible();

    fireEvent.click(screen.getByRole("tab", { name: /empyrean/i }));
    expect(screen.queryByText(AF3_HEAD.item_name)).not.toBeInTheDocument();
  });

  it("shows a loading message before data arrives", () => {
    vi.spyOn(bridge, "fetchGearProgression").mockReturnValue(new Promise(() => {})); // never resolves

    renderAt("1", "20");

    expect(screen.getByText(/waiting for gear data/i)).toBeInTheDocument();
  });

  it("shows a distinct error message when the fetch fails, not the empty-state message", async () => {
    vi.spyOn(bridge, "fetchGearProgression").mockRejectedValue(new Error("network error"));
    vi.spyOn(console, "error").mockImplementation(() => {});

    renderAt("1", "20");

    expect(await screen.findByText(/failed to load gear data/i)).toBeInTheDocument();
    expect(screen.queryByText(/no .* armor exists for this job/i)).not.toBeInTheDocument();
  });

  it("shows unverifiable even for a tier below the character's current tier", async () => {
    const AF3_HEAD_T0_UNVERIFIABLE = { job_id: 20, set_type: "af3" as const, slot: "head" as const, tier: 0, item_name: "Mystery Item", item_id: null };
    const AF3_HEAD_T1_KNOWN = { job_id: 20, set_type: "af3" as const, slot: "head" as const, tier: 1, item_name: "Known Item T1", item_id: 99 };

    vi.spyOn(bridge, "fetchGearProgression").mockResolvedValue({
      definitions: [AF3_HEAD_T0_UNVERIFIABLE, AF3_HEAD_T1_KNOWN],
      current_tiers: [{ set_type: "af3", slot: "head", current_tier: 1 }],
    });

    renderAt("1", "20");

    await screen.findByText("Mystery Item");
    // Tier 0 has a null item_id, so it must show as unverifiable — NOT "reached",
    // even though tier 0 < the character's current tier of 1.
    expect(screen.getByText("Mystery Item").closest('[data-state]')).toHaveAttribute("data-state", "unverifiable");
  });
});
