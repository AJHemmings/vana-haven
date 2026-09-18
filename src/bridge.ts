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

export type GearSlot = "head" | "body" | "hands" | "legs" | "feet";

export type GearSetDefinitionRow = {
  job_id: number;
  set_type: "af3" | "empyrean" | "relic";
  slot: GearSlot;
  tier: number;
  item_name: string;
  item_id: number | null;
};

export type SlotTier = {
  set_type: "af3" | "empyrean" | "relic";
  slot: GearSlot;
  current_tier: number | null;
};

export type GearProgression = {
  definitions: GearSetDefinitionRow[];
  current_tiers: SlotTier[];
};

export type KeyItemCatalogEntry = {
  key_item_id: number;
  name: string;
};

export type KeyItemTrackingRow = {
  id: number;
  key_item_id: number;
  name: string;
  granting_npc: string | null;
  cooldown_duration_seconds: number;
  last_acquired_at: string | null;
  currently_held: boolean;
};

export type DailyTodoItem = {
  id: number;
  text: string;
  cadence: "daily" | "weekly" | "monthly";
  last_completed_at: string | null;
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

export async function fetchGearProgression(gameCharacterId: number, jobId: number): Promise<GearProgression> {
  return invoke<GearProgression>("get_gear_progression", { gameCharacterId, jobId });
}

export async function fetchKeyItemCatalog(): Promise<KeyItemCatalogEntry[]> {
  return invoke<KeyItemCatalogEntry[]>("get_key_item_catalog");
}

export async function createKeyItemDefinition(
  keyItemId: number,
  name: string,
  grantingNpc: string | null,
  cooldownDurationSeconds: number
): Promise<number> {
  return invoke<number>("create_key_item_definition", {
    keyItemId,
    name,
    grantingNpc,
    cooldownDurationSeconds,
  });
}

export async function deleteKeyItemDefinition(id: number): Promise<void> {
  return invoke<void>("delete_key_item_definition", { id });
}

export async function fetchKeyItemTracking(gameCharacterId: number): Promise<KeyItemTrackingRow[]> {
  return invoke<KeyItemTrackingRow[]>("get_key_item_tracking", { gameCharacterId });
}

export async function fetchDailyTodoItems(gameCharacterId: number): Promise<DailyTodoItem[]> {
  return invoke<DailyTodoItem[]>("get_daily_todo_items", { gameCharacterId });
}

export async function createDailyTodoItem(
  gameCharacterId: number,
  text: string,
  cadence: DailyTodoItem["cadence"]
): Promise<number> {
  return invoke<number>("create_daily_todo_item", { gameCharacterId, text, cadence });
}

export async function deleteDailyTodoItem(id: number): Promise<void> {
  return invoke<void>("delete_daily_todo_item", { id });
}

export async function setDailyTodoCompleted(id: number, completed: boolean): Promise<void> {
  return invoke<void>("set_daily_todo_completed", { id, completed });
}

export async function fetchMonthlyCycleStartedAt(): Promise<string | null> {
  return invoke<string | null>("get_monthly_cycle_started_at");
}

export async function advanceMonthlyCycle(): Promise<string> {
  return invoke<string>("advance_monthly_cycle");
}

export function onCharacterUpdated(callback: () => void): Promise<() => void> {
  return listen("character-updated", callback).then((unlisten) => unlisten);
}

export function onKeyItemCatalogUpdated(callback: () => void): Promise<() => void> {
  return listen("key-item-catalog-updated", callback).then((unlisten) => unlisten);
}
