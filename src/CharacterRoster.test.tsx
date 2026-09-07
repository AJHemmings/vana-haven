import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import CharacterRoster from "./CharacterRoster";

describe("CharacterRoster", () => {
  it("shows an empty-state message when there are no characters", () => {
    render(<CharacterRoster characters={[]} />);
    expect(screen.getByText(/no characters yet/i)).toBeInTheDocument();
  });

  it("lists each character by name", () => {
    render(
      <CharacterRoster
        characters={[
          { game_character_id: 1, name: "Gozoto", last_seen_at: "2026-09-07T12:00:00Z" },
          { game_character_id: 2, name: "Unii", last_seen_at: "2026-09-07T12:05:00Z" },
        ]}
      />
    );
    expect(screen.getByText("Gozoto")).toBeInTheDocument();
    expect(screen.getByText("Unii")).toBeInTheDocument();
  });
});
