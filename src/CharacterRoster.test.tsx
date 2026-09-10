import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router-dom";
import { describe, expect, it } from "vitest";
import CharacterRoster from "./CharacterRoster";

describe("CharacterRoster", () => {
  it("shows an empty-state message when there are no characters", () => {
    render(
      <MemoryRouter>
        <CharacterRoster characters={[]} />
      </MemoryRouter>
    );
    expect(screen.getByText(/no characters yet/i)).toBeInTheDocument();
  });

  it("lists each character by name, linking to their character page", () => {
    render(
      <MemoryRouter>
        <CharacterRoster
          characters={[
            { game_character_id: 1, name: "Gozoto", last_seen_at: "2026-09-07T12:00:00Z" },
            { game_character_id: 2, name: "Unii", last_seen_at: "2026-09-07T12:05:00Z" },
          ]}
        />
      </MemoryRouter>
    );
    const gozotoLink = screen.getByText("Gozoto");
    expect(gozotoLink).toBeInTheDocument();
    expect(gozotoLink.closest("a")).toHaveAttribute("href", "/character/1");
    expect(screen.getByText("Unii").closest("a")).toHaveAttribute("href", "/character/2");
  });
});
