import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type { Character } from "./CharacterRoster";

export type CharacterDetail = {
  game_character_id: number;
  name: string;
  main_job_id: number | null;
  sub_job_id: number | null;
};

export type JobLevel = {
  job_id: number;
  level: number;
  master_level: number;
  mastered: boolean;
};

export async function fetchCharacters(): Promise<Character[]> {
  return invoke<Character[]>("get_characters");
}

export async function fetchCharacter(gameCharacterId: number): Promise<CharacterDetail | null> {
  return invoke<CharacterDetail | null>("get_character", { gameCharacterId });
}

export async function fetchCharacterJobs(gameCharacterId: number): Promise<JobLevel[]> {
  return invoke<JobLevel[]>("get_character_jobs", { gameCharacterId });
}

export function onCharacterUpdated(callback: () => void): Promise<() => void> {
  return listen("character-updated", callback).then((unlisten) => unlisten);
}
