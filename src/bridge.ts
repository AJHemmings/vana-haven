import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Character } from "./CharacterRoster";

export async function fetchCharacters(): Promise<Character[]> {
  return invoke<Character[]>("get_characters");
}

export function onCharacterUpdated(callback: () => void): Promise<() => void> {
  return listen("character-updated", callback).then((unlisten) => unlisten);
}
