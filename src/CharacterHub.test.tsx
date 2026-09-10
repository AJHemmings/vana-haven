import { render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import CharacterHub from "./CharacterHub";
import * as bridge from "./bridge";

function renderAt(gameCharacterId: string) {
  return render(
    <MemoryRouter initialEntries={[`/character/${gameCharacterId}`]}>
      <Routes>
        <Route path="/character/:gameCharacterId" element={<CharacterHub />} />
      </Routes>
    </MemoryRouter>
  );
}

describe("CharacterHub", () => {
  beforeEach(() => {
    vi.spyOn(bridge, "onCharacterUpdated").mockResolvedValue(() => {});
  });

  it("shows a waiting message before job data has arrived", async () => {
    vi.spyOn(bridge, "fetchCharacter").mockResolvedValue({
      game_character_id: 1,
      name: "Gozoto",
      main_job_id: null,
      sub_job_id: null,
    });
    vi.spyOn(bridge, "fetchCharacterJobs").mockResolvedValue([]);

    renderAt("1");

    expect(await screen.findByText("Gozoto")).toBeInTheDocument();
    expect(screen.getByText(/waiting for job data/i)).toBeInTheDocument();
  });

  it("shows the main/sub job summary with level once data has arrived", async () => {
    vi.spyOn(bridge, "fetchCharacter").mockResolvedValue({
      game_character_id: 1,
      name: "Gozoto",
      main_job_id: 4,
      sub_job_id: 20,
    });
    vi.spyOn(bridge, "fetchCharacterJobs").mockResolvedValue([
      { job_id: 4, level: 99, master_level: 12, mastered: true },
      { job_id: 20, level: 49, master_level: 0, mastered: false },
    ]);

    renderAt("1");

    expect(await screen.findByText("BLM Lv.99 / SCH Lv.49")).toBeInTheDocument();
  });

  it("shows the main job alone when no sub job is equipped", async () => {
    vi.spyOn(bridge, "fetchCharacter").mockResolvedValue({
      game_character_id: 1,
      name: "Gozoto",
      main_job_id: 4,
      sub_job_id: 0,
    });
    vi.spyOn(bridge, "fetchCharacterJobs").mockResolvedValue([
      { job_id: 4, level: 99, master_level: 12, mastered: true },
    ]);

    renderAt("1");

    expect(await screen.findByText("BLM Lv.99")).toBeInTheDocument();
  });

  it("links to the Jobs screen", async () => {
    vi.spyOn(bridge, "fetchCharacter").mockResolvedValue({
      game_character_id: 1,
      name: "Gozoto",
      main_job_id: 4,
      sub_job_id: 20,
    });
    vi.spyOn(bridge, "fetchCharacterJobs").mockResolvedValue([
      { job_id: 4, level: 99, master_level: 12, mastered: true },
      { job_id: 20, level: 49, master_level: 0, mastered: false },
    ]);

    renderAt("1");

    const jobsLink = await screen.findByText("Jobs");
    expect(jobsLink.closest("a")).toHaveAttribute("href", "/character/1/jobs");
  });
});
