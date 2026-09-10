import { render, screen } from "@testing-library/react";
import { MemoryRouter, Route, Routes } from "react-router-dom";
import { beforeEach, describe, expect, it, vi } from "vitest";
import JobsScreen from "./JobsScreen";
import * as bridge from "./bridge";

function renderAt(gameCharacterId: string) {
  return render(
    <MemoryRouter initialEntries={[`/character/${gameCharacterId}/jobs`]}>
      <Routes>
        <Route path="/character/:gameCharacterId/jobs" element={<JobsScreen />} />
      </Routes>
    </MemoryRouter>
  );
}

describe("JobsScreen", () => {
  beforeEach(() => {
    vi.spyOn(bridge, "onCharacterUpdated").mockResolvedValue(() => {});
  });

  it("shows a waiting message when no job data exists yet", async () => {
    vi.spyOn(bridge, "fetchCharacter").mockResolvedValue({
      game_character_id: 1,
      name: "Gozoto",
      main_job_id: null,
      sub_job_id: null,
    });
    vi.spyOn(bridge, "fetchCharacterJobs").mockResolvedValue([]);

    renderAt("1");

    expect(await screen.findByText(/waiting for job data/i)).toBeInTheDocument();
  });

  it("restates main/sub job with level at the top", async () => {
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

  it("shows all 22 jobs in canonical order, even ones with no data", async () => {
    vi.spyOn(bridge, "fetchCharacter").mockResolvedValue({
      game_character_id: 1,
      name: "Gozoto",
      main_job_id: 4,
      sub_job_id: 20,
    });
    vi.spyOn(bridge, "fetchCharacterJobs").mockResolvedValue([
      { job_id: 4, level: 99, master_level: 12, mastered: true },
    ]);

    renderAt("1");

    expect(await screen.findByText("WAR")).toBeInTheDocument(); // job 1, no data
    expect(screen.getByText("BLM")).toBeInTheDocument(); // job 4, has data
    expect(screen.getByText("RUN")).toBeInTheDocument(); // job 22, no data
  });

  it("shows level and master level for a reported job", async () => {
    vi.spyOn(bridge, "fetchCharacter").mockResolvedValue({
      game_character_id: 1,
      name: "Gozoto",
      main_job_id: 4,
      sub_job_id: 20,
    });
    vi.spyOn(bridge, "fetchCharacterJobs").mockResolvedValue([
      { job_id: 4, level: 99, master_level: 12, mastered: true },
    ]);

    renderAt("1");

    await screen.findByText("BLM");
    const blmRow = screen.getByText("BLM").closest("li");
    expect(blmRow).toHaveTextContent("Lv. 99");
    expect(blmRow).toHaveTextContent("ML 12");
  });

  it("shows level 0 / ML 0 for a job with no reported row", async () => {
    vi.spyOn(bridge, "fetchCharacter").mockResolvedValue({
      game_character_id: 1,
      name: "Gozoto",
      main_job_id: 4,
      sub_job_id: 20,
    });
    vi.spyOn(bridge, "fetchCharacterJobs").mockResolvedValue([
      { job_id: 4, level: 99, master_level: 12, mastered: true },
    ]);

    renderAt("1");

    await screen.findByText("WAR");
    const warRow = screen.getByText("WAR").closest("li");
    expect(warRow).toHaveTextContent("Lv. 0");
    expect(warRow).toHaveTextContent("ML 0");
  });

  it("links back to the character hub", async () => {
    vi.spyOn(bridge, "fetchCharacter").mockResolvedValue({
      game_character_id: 1,
      name: "Gozoto",
      main_job_id: null,
      sub_job_id: null,
    });
    vi.spyOn(bridge, "fetchCharacterJobs").mockResolvedValue([]);

    renderAt("1");

    const backLink = await screen.findByText("← back");
    expect(backLink.closest("a")).toHaveAttribute("href", "/character/1");
  });
});
