use rusqlite::{Connection, Result};
use rusqlite::OptionalExtension;

pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS characters (
            id INTEGER PRIMARY KEY,
            game_character_id INTEGER NOT NULL UNIQUE,
            name TEXT NOT NULL,
            last_seen_at TEXT NOT NULL
        )",
        (),
    )?;

    for stmt in [
        "ALTER TABLE characters ADD COLUMN main_job_id INTEGER",
        "ALTER TABLE characters ADD COLUMN sub_job_id INTEGER",
    ] {
        match conn.execute(stmt, ()) {
            Ok(_) => {}
            Err(e) if e.to_string().contains("duplicate column name") => {}
            Err(e) => return Err(e),
        }
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS character_jobs (
            character_id INTEGER NOT NULL REFERENCES characters(id),
            job_id INTEGER NOT NULL,
            level INTEGER NOT NULL,
            master_level INTEGER NOT NULL,
            mastered INTEGER NOT NULL,
            PRIMARY KEY (character_id, job_id)
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS character_items (
            character_id INTEGER NOT NULL REFERENCES characters(id),
            item_id INTEGER NOT NULL,
            container INTEGER NOT NULL,
            PRIMARY KEY (character_id, item_id, container)
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS gear_set_definitions (
            job_id INTEGER NOT NULL,
            set_type TEXT NOT NULL,
            slot TEXT NOT NULL,
            tier INTEGER NOT NULL,
            item_name TEXT NOT NULL,
            item_id INTEGER,
            PRIMARY KEY (job_id, set_type, slot, tier)
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS key_item_catalog (
            key_item_id INTEGER PRIMARY KEY,
            name TEXT NOT NULL
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS key_item_definitions (
            id INTEGER PRIMARY KEY,
            key_item_id INTEGER NOT NULL,
            name TEXT NOT NULL,
            granting_npc TEXT,
            cooldown_duration_seconds INTEGER NOT NULL
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS character_key_items_held (
            character_id INTEGER NOT NULL REFERENCES characters(id),
            key_item_id INTEGER NOT NULL,
            PRIMARY KEY (character_id, key_item_id)
        )",
        (),
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS character_key_item_cooldowns (
            character_id INTEGER NOT NULL REFERENCES characters(id),
            key_item_definition_id INTEGER NOT NULL REFERENCES key_item_definitions(id),
            last_acquired_at TEXT,
            PRIMARY KEY (character_id, key_item_definition_id)
        )",
        (),
    )?;

    // A character can legitimately hold zero key items on a real (non-first)
    // report — e.g. every tracked item was used/lost since the last one. Row
    // count in character_key_items_held can't tell "never reported" apart
    // from "reported, held nothing," so a genuine one-bit marker is needed to
    // know whether a previous snapshot exists to compare against at all (see
    // report_key_items_held).
    conn.execute(
        "CREATE TABLE IF NOT EXISTS character_key_items_baseline (
            character_id INTEGER PRIMARY KEY REFERENCES characters(id)
        )",
        (),
    )?;
    Ok(())
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct Character {
    pub game_character_id: i64,
    pub name: String,
    pub last_seen_at: String,
}

pub fn upsert_character(conn: &Connection, character: &Character) -> Result<()> {
    conn.execute(
        "INSERT INTO characters (game_character_id, name, last_seen_at)
         VALUES (?1, ?2, ?3)
         ON CONFLICT(game_character_id) DO UPDATE SET
            name = excluded.name,
            last_seen_at = excluded.last_seen_at",
        (&character.game_character_id, &character.name, &character.last_seen_at),
    )?;
    Ok(())
}

pub fn touch_character(conn: &Connection, game_character_id: i64, last_seen_at: &str) -> Result<()> {
    conn.execute(
        "UPDATE characters SET last_seen_at = ?1 WHERE game_character_id = ?2",
        (last_seen_at, game_character_id),
    )?;
    Ok(())
}

pub fn list_characters(conn: &Connection) -> Result<Vec<Character>> {
    let mut stmt = conn.prepare(
        "SELECT game_character_id, name, last_seen_at FROM characters ORDER BY name",
    )?;
    let rows = stmt.query_map((), |row| {
        Ok(Character {
            game_character_id: row.get(0)?,
            name: row.get(1)?,
            last_seen_at: row.get(2)?,
        })
    })?;
    rows.collect()
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct JobLevel {
    pub job_id: i64,
    pub level: i64,
    pub master_level: i64,
    pub mastered: bool,
}

pub fn replace_character_jobs(conn: &Connection, character_id: i64, jobs: &[JobLevel]) -> Result<()> {
    conn.execute("BEGIN", ())?;
    let result = (|| -> Result<()> {
        conn.execute("DELETE FROM character_jobs WHERE character_id = ?1", (character_id,))?;
        for job in jobs {
            conn.execute(
                "INSERT INTO character_jobs (character_id, job_id, level, master_level, mastered)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                (character_id, job.job_id, job.level, job.master_level, job.mastered),
            )?;
        }
        Ok(())
    })();
    match result {
        Ok(()) => {
            conn.execute("COMMIT", ())?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ());
            Err(e)
        }
    }
}

pub fn get_character_jobs(conn: &Connection, character_id: i64) -> Result<Vec<JobLevel>> {
    let mut stmt = conn.prepare(
        "SELECT job_id, level, master_level, mastered FROM character_jobs
         WHERE character_id = ?1 ORDER BY job_id",
    )?;
    let rows = stmt.query_map((character_id,), |row| {
        Ok(JobLevel {
            job_id: row.get(0)?,
            level: row.get(1)?,
            master_level: row.get(2)?,
            mastered: row.get(3)?,
        })
    })?;
    rows.collect()
}

/// Windower bag ids are 0-16 (per the addon's own `require('resources').bags`
/// enumeration, iterated in `collect_held_items()` rather than a hardcoded
/// list — see addon/VanaHaven/VanaHaven.lua), so -1 is a safe, distinct
/// sentinel for "this item is currently equipped," not one of the storage
/// bags.
///
/// `pub` but currently unreferenced outside this file's own tests: it's part
/// of the general "what does this character currently hold" primitive the
/// main design spec (§2) explicitly wants built generally enough for a
/// future consumer (Key Item Cooldowns, §8.3) to reuse — not dead scaffolding.
#[allow(dead_code)]
pub const EQUIPPED_CONTAINER: i64 = -1;

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct ItemHeld {
    pub item_id: i64,
    pub container: i64,
}

pub fn replace_character_items(conn: &Connection, character_id: i64, items: &[ItemHeld]) -> Result<()> {
    conn.execute("BEGIN", ())?;
    let result = (|| -> Result<()> {
        conn.execute("DELETE FROM character_items WHERE character_id = ?1", (character_id,))?;
        for item in items {
            conn.execute(
                "INSERT OR IGNORE INTO character_items (character_id, item_id, container)
                 VALUES (?1, ?2, ?3)",
                (character_id, item.item_id, item.container),
            )?;
        }
        Ok(())
    })();
    match result {
        Ok(()) => {
            conn.execute("COMMIT", ())?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ());
            Err(e)
        }
    }
}

/// `pub` but currently unreferenced outside this file's own tests, same
/// reasoning as `EQUIPPED_CONTAINER` above — the tier-computation query
/// (`compute_current_tiers`) reads `character_items` directly via its own
/// SQL rather than through this function, but a future feature (Key Item
/// Cooldowns, §8.3 of the main design spec) needs exactly this "what does
/// this character currently hold" read.
#[allow(dead_code)]
pub fn get_character_items(conn: &Connection, character_id: i64) -> Result<Vec<ItemHeld>> {
    let mut stmt = conn.prepare(
        "SELECT item_id, container FROM character_items WHERE character_id = ?1 ORDER BY item_id",
    )?;
    let rows = stmt.query_map((character_id,), |row| {
        Ok(ItemHeld { item_id: row.get(0)?, container: row.get(1)? })
    })?;
    rows.collect()
}

/// Lives here rather than in gear_reference.rs (which is the only current
/// constructor) because it's also the row type this module's future
/// gear_set_definitions insert/query functions will use.
#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct GearSetDefinitionRow {
    pub job_id: i64,
    pub set_type: String,
    pub slot: String,
    pub tier: i64,
    pub item_name: String,
    pub item_id: Option<i64>,
}

pub fn seed_gear_set_definitions_if_empty(conn: &Connection, rows: &[GearSetDefinitionRow]) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM gear_set_definitions", (), |row| row.get(0))?;
    if count > 0 {
        return Ok(());
    }
    conn.execute("BEGIN", ())?;
    let result = (|| -> Result<()> {
        for row in rows {
            conn.execute(
                "INSERT INTO gear_set_definitions (job_id, set_type, slot, tier, item_name, item_id)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                (row.job_id, &row.set_type, &row.slot, row.tier, &row.item_name, row.item_id),
            )?;
        }
        Ok(())
    })();
    match result {
        Ok(()) => {
            conn.execute("COMMIT", ())?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ());
            Err(e)
        }
    }
}

pub fn get_gear_set_definitions(conn: &Connection, job_id: i64) -> Result<Vec<GearSetDefinitionRow>> {
    let mut stmt = conn.prepare(
        "SELECT job_id, set_type, slot, tier, item_name, item_id FROM gear_set_definitions
         WHERE job_id = ?1 ORDER BY set_type, slot, tier",
    )?;
    let rows = stmt.query_map((job_id,), |row| {
        Ok(GearSetDefinitionRow {
            job_id: row.get(0)?,
            set_type: row.get(1)?,
            slot: row.get(2)?,
            tier: row.get(3)?,
            item_name: row.get(4)?,
            item_id: row.get(5)?,
        })
    })?;
    rows.collect()
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct SlotTier {
    pub set_type: String,
    pub slot: String,
    pub current_tier: Option<i64>,
}

/// "Current tier" is the highest tier of each (set_type, slot) whose item_id
/// the character currently holds. Two things make the SQL below less obvious
/// than it looks:
/// - `gsd.item_id` is NULL for the 87 unresolved Relic +2 rows (BG-Wiki
///   never had their item ids). `ci.item_id = gsd.item_id` is never true
///   when either side is NULL, so those rows always LEFT JOIN to nothing
///   and never contribute to MAX — don't "simplify" this into an INNER JOIN
///   or an equality check that assumes NULL behaves like a normal value.
/// - A character can (per the schema, not expected in practice) hold the
///   same item_id in two containers at once, which fans the LEFT JOIN out
///   to two matching rows per gsd row. This is harmless: both carry the
///   same gsd.tier, and MAX is idempotent over a duplicate value.
pub fn compute_current_tiers(conn: &Connection, character_id: i64, job_id: i64) -> Result<Vec<SlotTier>> {
    let mut stmt = conn.prepare(
        "SELECT gsd.set_type, gsd.slot,
                MAX(CASE WHEN ci.item_id IS NOT NULL THEN gsd.tier END) AS current_tier
         FROM gear_set_definitions gsd
         LEFT JOIN character_items ci
           ON ci.character_id = ?1 AND ci.item_id = gsd.item_id
         WHERE gsd.job_id = ?2
         GROUP BY gsd.set_type, gsd.slot
         ORDER BY gsd.set_type, gsd.slot",
    )?;
    let rows = stmt.query_map((character_id, job_id), |row| {
        Ok(SlotTier {
            set_type: row.get(0)?,
            slot: row.get(1)?,
            current_tier: row.get(2)?,
        })
    })?;
    rows.collect()
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct KeyItemCatalogEntry {
    pub key_item_id: i64,
    pub name: String,
}

/// Global (not per-character), wholesale-replaced every addon connection —
/// see the Key Item Cooldowns spec §3 for why this table is re-sent every
/// session rather than seeded once like gear_set_definitions.
pub fn replace_key_item_catalog(conn: &Connection, entries: &[KeyItemCatalogEntry]) -> Result<()> {
    conn.execute("BEGIN", ())?;
    let result = (|| -> Result<()> {
        conn.execute("DELETE FROM key_item_catalog", ())?;
        for entry in entries {
            conn.execute(
                "INSERT INTO key_item_catalog (key_item_id, name) VALUES (?1, ?2)",
                (entry.key_item_id, &entry.name),
            )?;
        }
        Ok(())
    })();
    match result {
        Ok(()) => {
            conn.execute("COMMIT", ())?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ());
            Err(e)
        }
    }
}

pub fn get_key_item_catalog(conn: &Connection) -> Result<Vec<KeyItemCatalogEntry>> {
    let mut stmt = conn.prepare("SELECT key_item_id, name FROM key_item_catalog ORDER BY name")?;
    let rows = stmt.query_map((), |row| {
        Ok(KeyItemCatalogEntry { key_item_id: row.get(0)?, name: row.get(1)? })
    })?;
    rows.collect()
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct KeyItemDefinition {
    pub id: i64,
    pub key_item_id: i64,
    pub name: String,
    pub granting_npc: Option<String>,
    pub cooldown_duration_seconds: i64,
}

/// `name` is denormalized from key_item_catalog at creation time (not looked
/// up here) so a definition still displays correctly even if the catalog is
/// later replaced by a report missing that entry — see spec §4/§9.
pub fn create_key_item_definition(
    conn: &Connection,
    key_item_id: i64,
    name: &str,
    granting_npc: Option<&str>,
    cooldown_duration_seconds: i64,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO key_item_definitions (key_item_id, name, granting_npc, cooldown_duration_seconds)
         VALUES (?1, ?2, ?3, ?4)",
        (key_item_id, name, granting_npc, cooldown_duration_seconds),
    )?;
    Ok(conn.last_insert_rowid())
}

/// Cascades character_key_item_cooldowns rows for this definition across
/// every character — a deleted definition shouldn't leave orphaned cooldown
/// rows behind that a re-added definition with a new id could never surface
/// (per spec §10's "decide in the plan": cascade, not orphan).
pub fn delete_key_item_definition(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("BEGIN", ())?;
    let result = (|| -> Result<()> {
        conn.execute("DELETE FROM character_key_item_cooldowns WHERE key_item_definition_id = ?1", (id,))?;
        conn.execute("DELETE FROM key_item_definitions WHERE id = ?1", (id,))?;
        Ok(())
    })();
    match result {
        Ok(()) => {
            conn.execute("COMMIT", ())?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ());
            Err(e)
        }
    }
}

/// `pub` but currently unreferenced outside this file's own tests, same
/// reasoning as `EQUIPPED_CONTAINER`/`get_character_items` above: no Tauri
/// command needs a definitions-without-character-context view (the UI always
/// reads the per-character join, `get_key_item_tracking`), but it's the
/// natural list-all counterpart to create/delete and worth keeping tested.
#[allow(dead_code)]
pub fn get_key_item_definitions(conn: &Connection) -> Result<Vec<KeyItemDefinition>> {
    let mut stmt = conn.prepare(
        "SELECT id, key_item_id, name, granting_npc, cooldown_duration_seconds
         FROM key_item_definitions ORDER BY name",
    )?;
    let rows = stmt.query_map((), |row| {
        Ok(KeyItemDefinition {
            id: row.get(0)?,
            key_item_id: row.get(1)?,
            name: row.get(2)?,
            granting_npc: row.get(3)?,
            cooldown_duration_seconds: row.get(4)?,
        })
    })?;
    rows.collect()
}

pub fn get_character_key_items_held(conn: &Connection, character_id: i64) -> Result<Vec<i64>> {
    let mut stmt = conn.prepare(
        "SELECT key_item_id FROM character_key_items_held WHERE character_id = ?1 ORDER BY key_item_id",
    )?;
    let rows = stmt.query_map((character_id,), |row| row.get(0))?;
    rows.collect()
}

/// The one piece of real logic in this module, analogous to
/// `compute_current_tiers` for Gear Set Progression — see the Key Item
/// Cooldowns spec §5 for the full reasoning behind each step.
///
/// A character's first-ever report is a baseline, never a transition: it
/// wholesale-replaces the snapshot and returns without touching
/// character_key_item_cooldowns at all. Read literally, "present in the new
/// report but not in an empty previous snapshot" would make every currently-
/// held tracked key item look like a fresh acquisition on that first report,
/// fabricating `last_acquired_at = now` for items the character has clearly
/// held for an unknown amount of time already — exactly what spec §7's third
/// honest UI state exists to avoid guessing about. "First-ever" is tracked
/// via character_key_items_baseline explicitly, not by checking whether
/// character_key_items_held has any rows: a character can legitimately hold
/// zero key items on a real, later report too (every tracked item used or
/// lost since the last one), and that must still compare against the true
/// (empty) previous snapshot rather than being mistaken for "never reported."
pub fn report_key_items_held(
    conn: &Connection,
    character_id: i64,
    key_item_ids: &[i64],
    now: &str,
) -> Result<()> {
    conn.execute("BEGIN", ())?;
    let result = (|| -> Result<()> {
        let has_baseline: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM character_key_items_baseline WHERE character_id = ?1)",
            (character_id,),
            |row| row.get(0),
        )?;
        if has_baseline {
            let previous = get_character_key_items_held(conn, character_id)?;
            let mut def_stmt = conn.prepare("SELECT id, key_item_id FROM key_item_definitions")?;
            let definitions: Vec<(i64, i64)> = def_stmt
                .query_map((), |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<Result<_>>()?;
            for (definition_id, key_item_id) in definitions {
                let transitioned = key_item_ids.contains(&key_item_id) && !previous.contains(&key_item_id);
                if transitioned {
                    conn.execute(
                        "INSERT INTO character_key_item_cooldowns (character_id, key_item_definition_id, last_acquired_at)
                         VALUES (?1, ?2, ?3)
                         ON CONFLICT(character_id, key_item_definition_id) DO UPDATE SET
                            last_acquired_at = excluded.last_acquired_at",
                        (character_id, definition_id, now),
                    )?;
                }
            }
        } else {
            conn.execute(
                "INSERT INTO character_key_items_baseline (character_id) VALUES (?1)",
                (character_id,),
            )?;
        }
        conn.execute("DELETE FROM character_key_items_held WHERE character_id = ?1", (character_id,))?;
        for key_item_id in key_item_ids {
            conn.execute(
                "INSERT OR IGNORE INTO character_key_items_held (character_id, key_item_id) VALUES (?1, ?2)",
                (character_id, key_item_id),
            )?;
        }
        Ok(())
    })();
    match result {
        Ok(()) => {
            conn.execute("COMMIT", ())?;
            Ok(())
        }
        Err(e) => {
            let _ = conn.execute("ROLLBACK", ());
            Err(e)
        }
    }
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct KeyItemTracking {
    pub id: i64,
    pub key_item_id: i64,
    pub name: String,
    pub granting_npc: Option<String>,
    pub cooldown_duration_seconds: i64,
    pub last_acquired_at: Option<String>,
    pub currently_held: bool,
}

/// Joins key_item_definitions against both character_key_items_held and
/// character_key_item_cooldowns so the UI can tell the three honest states
/// apart (spec §4, §7) from one call: Ready needs to know cooldown elapsed
/// OR never held; the "held but unknown acquisition" state needs both
/// `currently_held` and a null `last_acquired_at` together, which neither
/// table alone can answer.
pub fn get_key_item_tracking(conn: &Connection, character_id: i64) -> Result<Vec<KeyItemTracking>> {
    let mut stmt = conn.prepare(
        "SELECT kid.id, kid.key_item_id, kid.name, kid.granting_npc, kid.cooldown_duration_seconds,
                ckic.last_acquired_at,
                CASE WHEN ckih.key_item_id IS NOT NULL THEN 1 ELSE 0 END AS currently_held
         FROM key_item_definitions kid
         LEFT JOIN character_key_item_cooldowns ckic
           ON ckic.character_id = ?1 AND ckic.key_item_definition_id = kid.id
         LEFT JOIN character_key_items_held ckih
           ON ckih.character_id = ?1 AND ckih.key_item_id = kid.key_item_id
         ORDER BY kid.name",
    )?;
    let rows = stmt.query_map((character_id,), |row| {
        Ok(KeyItemTracking {
            id: row.get(0)?,
            key_item_id: row.get(1)?,
            name: row.get(2)?,
            granting_npc: row.get(3)?,
            cooldown_duration_seconds: row.get(4)?,
            last_acquired_at: row.get(5)?,
            currently_held: row.get(6)?,
        })
    })?;
    rows.collect()
}

#[derive(Debug, PartialEq, serde::Serialize)]
pub struct CharacterDetail {
    pub game_character_id: i64,
    pub name: String,
    pub main_job_id: Option<i64>,
    pub sub_job_id: Option<i64>,
}

pub fn resolve_character_id(conn: &Connection, game_character_id: i64) -> Result<Option<i64>> {
    conn.query_row(
        "SELECT id FROM characters WHERE game_character_id = ?1",
        (game_character_id,),
        |row| row.get(0),
    )
    .optional()
}

pub fn get_character(conn: &Connection, game_character_id: i64) -> Result<Option<CharacterDetail>> {
    conn.query_row(
        "SELECT game_character_id, name, main_job_id, sub_job_id FROM characters
         WHERE game_character_id = ?1",
        (game_character_id,),
        |row| {
            Ok(CharacterDetail {
                game_character_id: row.get(0)?,
                name: row.get(1)?,
                main_job_id: row.get(2)?,
                sub_job_id: row.get(3)?,
            })
        },
    )
    .optional()
}

pub fn update_character_jobs_summary(
    conn: &Connection,
    character_id: i64,
    main_job_id: i64,
    sub_job_id: i64,
) -> Result<()> {
    conn.execute(
        "UPDATE characters SET main_job_id = ?1, sub_job_id = ?2 WHERE id = ?3",
        (main_job_id, sub_job_id, character_id),
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        conn
    }

    #[test]
    fn list_characters_is_empty_on_fresh_db() {
        let conn = setup();
        assert_eq!(list_characters(&conn).unwrap(), vec![]);
    }

    #[test]
    fn upsert_character_inserts_new_character() {
        let conn = setup();
        let character = Character {
            game_character_id: 12345,
            name: "Gozoto".to_string(),
            last_seen_at: "2026-09-07T12:00:00Z".to_string(),
        };
        upsert_character(&conn, &character).unwrap();
        assert_eq!(list_characters(&conn).unwrap(), vec![character]);
    }

    #[test]
    fn upsert_character_updates_existing_character_by_game_id() {
        let conn = setup();
        upsert_character(&conn, &Character {
            game_character_id: 12345,
            name: "Gozoto".to_string(),
            last_seen_at: "2026-09-07T12:00:00Z".to_string(),
        }).unwrap();
        upsert_character(&conn, &Character {
            game_character_id: 12345,
            name: "Gozoto".to_string(),
            last_seen_at: "2026-09-07T13:00:00Z".to_string(),
        }).unwrap();

        let characters = list_characters(&conn).unwrap();
        assert_eq!(characters.len(), 1);
        assert_eq!(characters[0].last_seen_at, "2026-09-07T13:00:00Z");
    }

    #[test]
    fn touch_character_updates_last_seen_for_known_character() {
        let conn = setup();
        upsert_character(&conn, &Character {
            game_character_id: 12345,
            name: "Gozoto".to_string(),
            last_seen_at: "2026-09-07T12:00:00Z".to_string(),
        }).unwrap();

        touch_character(&conn, 12345, "2026-09-07T14:30:00Z").unwrap();

        assert_eq!(list_characters(&conn).unwrap()[0].last_seen_at, "2026-09-07T14:30:00Z");
    }

    #[test]
    fn get_character_jobs_is_empty_for_unknown_character() {
        let conn = setup();
        assert_eq!(get_character_jobs(&conn, 1).unwrap(), vec![]);
    }

    #[test]
    fn replace_character_jobs_inserts_all_rows() {
        let conn = setup();
        upsert_character(&conn, &Character {
            game_character_id: 12345,
            name: "Gozoto".to_string(),
            last_seen_at: "2026-09-07T12:00:00Z".to_string(),
        }).unwrap();
        let jobs = vec![
            JobLevel { job_id: 1, level: 75, master_level: 0, mastered: false },
            JobLevel { job_id: 22, level: 99, master_level: 12, mastered: true },
        ];
        replace_character_jobs(&conn, 1, &jobs).unwrap();
        assert_eq!(get_character_jobs(&conn, 1).unwrap(), jobs);
    }

    #[test]
    fn replace_character_jobs_replaces_wholesale_not_incrementally() {
        let conn = setup();
        upsert_character(&conn, &Character {
            game_character_id: 12345,
            name: "Gozoto".to_string(),
            last_seen_at: "2026-09-07T12:00:00Z".to_string(),
        }).unwrap();
        replace_character_jobs(&conn, 1, &[
            JobLevel { job_id: 1, level: 50, master_level: 0, mastered: false },
            JobLevel { job_id: 2, level: 10, master_level: 0, mastered: false },
        ]).unwrap();

        // Second call omits job_id 2 entirely — it must be gone afterward, not stale.
        replace_character_jobs(&conn, 1, &[
            JobLevel { job_id: 1, level: 51, master_level: 0, mastered: false },
        ]).unwrap();

        let jobs = get_character_jobs(&conn, 1).unwrap();
        assert_eq!(jobs, vec![JobLevel { job_id: 1, level: 51, master_level: 0, mastered: false }]);
    }

    #[test]
    fn get_character_jobs_only_returns_rows_for_the_given_character() {
        let conn = setup();
        upsert_character(&conn, &Character {
            game_character_id: 12345,
            name: "Gozoto".to_string(),
            last_seen_at: "2026-09-07T12:00:00Z".to_string(),
        }).unwrap();
        upsert_character(&conn, &Character {
            game_character_id: 67890,
            name: "Zootog".to_string(),
            last_seen_at: "2026-09-07T13:00:00Z".to_string(),
        }).unwrap();
        replace_character_jobs(&conn, 1, &[JobLevel { job_id: 1, level: 50, master_level: 0, mastered: false }]).unwrap();
        replace_character_jobs(&conn, 2, &[JobLevel { job_id: 1, level: 99, master_level: 5, mastered: true }]).unwrap();

        assert_eq!(get_character_jobs(&conn, 1).unwrap()[0].level, 50);
        assert_eq!(get_character_jobs(&conn, 2).unwrap()[0].level, 99);
    }

    #[test]
    fn init_db_is_idempotent_across_repeated_calls() {
        let conn = Connection::open_in_memory().unwrap();
        init_db(&conn).unwrap();
        init_db(&conn).unwrap(); // must not error on the second call's ALTER TABLEs
    }

    #[test]
    fn resolve_character_id_returns_none_for_unknown_game_id() {
        let conn = setup();
        assert_eq!(resolve_character_id(&conn, 99999).unwrap(), None);
    }

    #[test]
    fn resolve_character_id_returns_internal_id_for_known_character() {
        let conn = setup();
        upsert_character(&conn, &Character {
            game_character_id: 12345,
            name: "Gozoto".to_string(),
            last_seen_at: "2026-09-07T12:00:00Z".to_string(),
        }).unwrap();

        let id = resolve_character_id(&conn, 12345).unwrap();
        assert!(id.is_some());
    }

    #[test]
    fn get_character_returns_none_for_unknown_game_id() {
        let conn = setup();
        assert_eq!(get_character(&conn, 99999).unwrap(), None);
    }

    #[test]
    fn get_character_returns_null_main_sub_job_before_first_report() {
        let conn = setup();
        upsert_character(&conn, &Character {
            game_character_id: 12345,
            name: "Gozoto".to_string(),
            last_seen_at: "2026-09-07T12:00:00Z".to_string(),
        }).unwrap();

        let detail = get_character(&conn, 12345).unwrap().unwrap();
        assert_eq!(detail.name, "Gozoto");
        assert_eq!(detail.main_job_id, None);
        assert_eq!(detail.sub_job_id, None);
    }

    #[test]
    fn update_character_jobs_summary_sets_main_and_sub_job() {
        let conn = setup();
        upsert_character(&conn, &Character {
            game_character_id: 12345,
            name: "Gozoto".to_string(),
            last_seen_at: "2026-09-07T12:00:00Z".to_string(),
        }).unwrap();
        let character_id = resolve_character_id(&conn, 12345).unwrap().unwrap();

        update_character_jobs_summary(&conn, character_id, 4, 20).unwrap();

        let detail = get_character(&conn, 12345).unwrap().unwrap();
        assert_eq!(detail.main_job_id, Some(4));
        assert_eq!(detail.sub_job_id, Some(20));
    }

    #[test]
    fn update_character_jobs_summary_allows_zero_sub_job_id() {
        let conn = setup();
        upsert_character(&conn, &Character {
            game_character_id: 12345,
            name: "Gozoto".to_string(),
            last_seen_at: "2026-09-07T12:00:00Z".to_string(),
        }).unwrap();
        let character_id = resolve_character_id(&conn, 12345).unwrap().unwrap();

        // sub_job_id 0 is a real, valid value meaning "no sub job equipped" — not
        // an error and not treated as absence.
        update_character_jobs_summary(&conn, character_id, 4, 0).unwrap();

        let detail = get_character(&conn, 12345).unwrap().unwrap();
        assert_eq!(detail.sub_job_id, Some(0));
    }

    #[test]
    fn get_character_items_is_empty_for_unknown_character() {
        let conn = setup();
        assert_eq!(get_character_items(&conn, 1).unwrap(), vec![]);
    }

    #[test]
    fn replace_character_items_inserts_all_rows() {
        let conn = setup();
        upsert_character(&conn, &Character {
            game_character_id: 12345,
            name: "Gozoto".to_string(),
            last_seen_at: "2026-09-07T12:00:00Z".to_string(),
        }).unwrap();
        let items = vec![
            ItemHeld { item_id: 15079, container: 0 },
            ItemHeld { item_id: 27683, container: EQUIPPED_CONTAINER },
        ];
        replace_character_items(&conn, 1, &items).unwrap();
        let mut result = get_character_items(&conn, 1).unwrap();
        result.sort_by_key(|i| i.item_id);
        assert_eq!(result, items);
    }

    #[test]
    fn replace_character_items_replaces_wholesale_not_incrementally() {
        let conn = setup();
        upsert_character(&conn, &Character {
            game_character_id: 12345,
            name: "Gozoto".to_string(),
            last_seen_at: "2026-09-07T12:00:00Z".to_string(),
        }).unwrap();
        replace_character_items(&conn, 1, &[
            ItemHeld { item_id: 100, container: 0 },
            ItemHeld { item_id: 200, container: 0 },
        ]).unwrap();

        // Second report omits item 200 entirely — it must be gone afterward,
        // not stale (mirrors replace_character_jobs's wholesale-replace test).
        replace_character_items(&conn, 1, &[
            ItemHeld { item_id: 100, container: 0 },
        ]).unwrap();

        assert_eq!(get_character_items(&conn, 1).unwrap(), vec![ItemHeld { item_id: 100, container: 0 }]);
    }

    #[test]
    fn get_character_items_only_returns_rows_for_the_given_character() {
        let conn = setup();
        upsert_character(&conn, &Character { game_character_id: 12345, name: "Gozoto".to_string(), last_seen_at: "2026-09-07T12:00:00Z".to_string() }).unwrap();
        upsert_character(&conn, &Character { game_character_id: 67890, name: "Zootog".to_string(), last_seen_at: "2026-09-07T13:00:00Z".to_string() }).unwrap();
        replace_character_items(&conn, 1, &[ItemHeld { item_id: 100, container: 0 }]).unwrap();
        replace_character_items(&conn, 2, &[ItemHeld { item_id: 200, container: 0 }]).unwrap();

        assert_eq!(get_character_items(&conn, 1), Ok(vec![ItemHeld { item_id: 100, container: 0 }]));
        assert_eq!(get_character_items(&conn, 2), Ok(vec![ItemHeld { item_id: 200, container: 0 }]));
    }

    #[test]
    fn replace_character_items_allows_the_same_item_id_in_two_containers() {
        // Not expected in practice (unique gear), but the schema shouldn't assume
        // it can't happen — e.g. two copies of a stackable-adjacent item.
        let conn = setup();
        upsert_character(&conn, &Character { game_character_id: 12345, name: "Gozoto".to_string(), last_seen_at: "2026-09-07T12:00:00Z".to_string() }).unwrap();
        replace_character_items(&conn, 1, &[
            ItemHeld { item_id: 100, container: 0 },
            ItemHeld { item_id: 100, container: 2 },
        ]).unwrap();
        assert_eq!(get_character_items(&conn, 1).unwrap().len(), 2);
    }

    #[test]
    fn replace_character_items_tolerates_duplicate_item_and_container_pairs() {
        // A real, ordinary FFXI inventory shape: two identical unstackable items
        // (e.g. two of the same ring) in the same bag produce two entries with
        // the exact same (item_id, container) — the schema's PRIMARY KEY would
        // otherwise reject the second one and roll back the ENTIRE snapshot,
        // silently and permanently stalling gear sync for that character.
        let conn = setup();
        upsert_character(&conn, &Character {
            game_character_id: 12345,
            name: "Gozoto".to_string(),
            last_seen_at: "2026-09-07T12:00:00Z".to_string(),
        }).unwrap();
        let result = replace_character_items(&conn, 1, &[
            ItemHeld { item_id: 100, container: 0 },
            ItemHeld { item_id: 100, container: 0 }, // exact duplicate
            ItemHeld { item_id: 200, container: 0 }, // a distinct item in the same write
        ]);
        assert!(result.is_ok());
        let items = get_character_items(&conn, 1).unwrap();
        // Exactly one row for the duplicated pair (not two, not zero), plus the distinct item.
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|i| i.item_id == 100 && i.container == 0));
        assert!(items.iter().any(|i| i.item_id == 200 && i.container == 0));
    }

    fn sample_definition_row(job_id: i64, tier: i64, item_id: Option<i64>) -> GearSetDefinitionRow {
        GearSetDefinitionRow {
            job_id,
            set_type: "af3".to_string(),
            slot: "head".to_string(),
            tier,
            item_name: format!("Test Item T{tier}"),
            item_id,
        }
    }

    #[test]
    fn get_gear_set_definitions_is_empty_before_seeding() {
        let conn = setup();
        assert_eq!(get_gear_set_definitions(&conn, 1).unwrap(), vec![]);
    }

    #[test]
    fn seed_gear_set_definitions_if_empty_inserts_rows() {
        let conn = setup();
        let rows = vec![sample_definition_row(20, 0, Some(1)), sample_definition_row(20, 1, Some(2))];
        seed_gear_set_definitions_if_empty(&conn, &rows).unwrap();
        assert_eq!(get_gear_set_definitions(&conn, 20).unwrap().len(), 2);
    }

    #[test]
    fn seed_gear_set_definitions_if_empty_skips_seeding_when_table_already_populated() {
        // This proves the COUNT(*)-guard short-circuits the second call —
        // it does NOT prove re-seeding with new/changed rows against an
        // already-populated table is safe (it currently isn't: the insert
        // loop would hit a PRIMARY KEY collision if it ever ran a second
        // time, since seeding is a one-time-ever operation by design).
        let conn = setup();
        let rows = vec![sample_definition_row(20, 0, Some(1))];
        seed_gear_set_definitions_if_empty(&conn, &rows).unwrap();
        seed_gear_set_definitions_if_empty(&conn, &rows).unwrap();
        assert_eq!(get_gear_set_definitions(&conn, 20).unwrap().len(), 1);
    }

    #[test]
    fn get_gear_set_definitions_only_returns_rows_for_the_given_job() {
        let conn = setup();
        let rows = vec![sample_definition_row(20, 0, Some(1)), sample_definition_row(21, 0, Some(2))];
        seed_gear_set_definitions_if_empty(&conn, &rows).unwrap();
        assert_eq!(get_gear_set_definitions(&conn, 20).unwrap().len(), 1);
        assert_eq!(get_gear_set_definitions(&conn, 21).unwrap().len(), 1);
    }

    #[test]
    fn get_gear_set_definitions_keeps_null_item_id_rows() {
        let conn = setup();
        let rows = vec![sample_definition_row(20, 2, None)];
        seed_gear_set_definitions_if_empty(&conn, &rows).unwrap();
        assert_eq!(get_gear_set_definitions(&conn, 20).unwrap()[0].item_id, None);
    }

    #[test]
    fn compute_current_tiers_reports_not_obtained_when_nothing_held() {
        let conn = setup();
        upsert_character(&conn, &Character { game_character_id: 12345, name: "Gozoto".to_string(), last_seen_at: "2026-09-07T12:00:00Z".to_string() }).unwrap();
        seed_gear_set_definitions_if_empty(&conn, &[
            sample_definition_row(20, 0, Some(1)),
            sample_definition_row(20, 1, Some(2)),
        ]).unwrap();

        let tiers = compute_current_tiers(&conn, 1, 20).unwrap();
        assert_eq!(tiers.len(), 1); // one (set_type, slot) group
        assert_eq!(tiers[0].current_tier, None);
    }

    #[test]
    fn compute_current_tiers_picks_highest_tier_when_multiple_are_held() {
        let conn = setup();
        upsert_character(&conn, &Character { game_character_id: 12345, name: "Gozoto".to_string(), last_seen_at: "2026-09-07T12:00:00Z".to_string() }).unwrap();
        seed_gear_set_definitions_if_empty(&conn, &[
            sample_definition_row(20, 0, Some(1)),
            sample_definition_row(20, 1, Some(2)),
            sample_definition_row(20, 2, Some(3)),
        ]).unwrap();
        // Shouldn't normally happen (upgrading is destructive) but the query
        // must not assume it can't: both tier 0 and tier 1 items held.
        replace_character_items(&conn, 1, &[
            ItemHeld { item_id: 1, container: 0 },
            ItemHeld { item_id: 2, container: 0 },
        ]).unwrap();

        let tiers = compute_current_tiers(&conn, 1, 20).unwrap();
        assert_eq!(tiers[0].current_tier, Some(1));
    }

    #[test]
    fn compute_current_tiers_ignores_null_item_id_rows() {
        let conn = setup();
        upsert_character(&conn, &Character { game_character_id: 12345, name: "Gozoto".to_string(), last_seen_at: "2026-09-07T12:00:00Z".to_string() }).unwrap();
        seed_gear_set_definitions_if_empty(&conn, &[
            sample_definition_row(20, 0, Some(1)),
            sample_definition_row(20, 1, None), // unresolved item_id — can never be "held"
        ]).unwrap();
        replace_character_items(&conn, 1, &[ItemHeld { item_id: 1, container: 0 }]).unwrap();

        let tiers = compute_current_tiers(&conn, 1, 20).unwrap();
        assert_eq!(tiers[0].current_tier, Some(0)); // the null row never contributes, doesn't crash
    }

    #[test]
    fn compute_current_tiers_covers_every_set_type_and_slot_for_the_job() {
        let conn = setup();
        upsert_character(&conn, &Character { game_character_id: 12345, name: "Gozoto".to_string(), last_seen_at: "2026-09-07T12:00:00Z".to_string() }).unwrap();
        seed_gear_set_definitions_if_empty(&conn, &[
            GearSetDefinitionRow { job_id: 20, set_type: "af3".to_string(), slot: "head".to_string(), tier: 0, item_name: "A".to_string(), item_id: Some(1) },
            GearSetDefinitionRow { job_id: 20, set_type: "af3".to_string(), slot: "body".to_string(), tier: 0, item_name: "B".to_string(), item_id: Some(2) },
            GearSetDefinitionRow { job_id: 20, set_type: "relic".to_string(), slot: "head".to_string(), tier: 0, item_name: "C".to_string(), item_id: Some(3) },
        ]).unwrap();

        let tiers = compute_current_tiers(&conn, 1, 20).unwrap();
        assert_eq!(tiers.len(), 3); // (af3,head), (af3,body), (relic,head)
    }

    #[test]
    fn compute_current_tiers_is_not_inflated_by_the_same_item_in_two_containers() {
        // Schema-permitted (see replace_character_items_allows_the_same_item_id_in_two_containers)
        // but not expected in practice — the LEFT JOIN fans out to two rows
        // for the one gsd row, and MAX must not double-count or misbehave.
        let conn = setup();
        upsert_character(&conn, &Character { game_character_id: 12345, name: "Gozoto".to_string(), last_seen_at: "2026-09-07T12:00:00Z".to_string() }).unwrap();
        seed_gear_set_definitions_if_empty(&conn, &[
            sample_definition_row(20, 0, Some(1)),
            sample_definition_row(20, 1, Some(2)),
        ]).unwrap();
        replace_character_items(&conn, 1, &[
            ItemHeld { item_id: 1, container: 0 },
            ItemHeld { item_id: 1, container: EQUIPPED_CONTAINER },
        ]).unwrap();

        let tiers = compute_current_tiers(&conn, 1, 20).unwrap();
        assert_eq!(tiers[0].current_tier, Some(0));
    }

    fn make_character(conn: &Connection, game_character_id: i64, name: &str) -> i64 {
        upsert_character(conn, &Character {
            game_character_id,
            name: name.to_string(),
            last_seen_at: "2026-09-15T12:00:00Z".to_string(),
        }).unwrap();
        resolve_character_id(conn, game_character_id).unwrap().unwrap()
    }

    #[test]
    fn get_key_item_catalog_is_empty_on_fresh_db() {
        let conn = setup();
        assert_eq!(get_key_item_catalog(&conn).unwrap(), vec![]);
    }

    #[test]
    fn replace_key_item_catalog_inserts_all_entries() {
        let conn = setup();
        let entries = vec![
            KeyItemCatalogEntry { key_item_id: 1, name: "Rubber Cockatrice".to_string() },
            KeyItemCatalogEntry { key_item_id: 2, name: "Mystical Canteen".to_string() },
        ];
        replace_key_item_catalog(&conn, &entries).unwrap();
        let mut result = get_key_item_catalog(&conn).unwrap();
        result.sort_by_key(|e| e.key_item_id);
        assert_eq!(result, entries);
    }

    #[test]
    fn replace_key_item_catalog_replaces_wholesale_not_incrementally() {
        let conn = setup();
        replace_key_item_catalog(&conn, &[
            KeyItemCatalogEntry { key_item_id: 1, name: "A".to_string() },
            KeyItemCatalogEntry { key_item_id: 2, name: "B".to_string() },
        ]).unwrap();

        // Second report omits key_item_id 2 entirely — a game patch or a
        // Windower version correcting something should show up automatically,
        // not leave stale rows behind (spec §3).
        replace_key_item_catalog(&conn, &[
            KeyItemCatalogEntry { key_item_id: 1, name: "A".to_string() },
        ]).unwrap();

        assert_eq!(get_key_item_catalog(&conn).unwrap(), vec![
            KeyItemCatalogEntry { key_item_id: 1, name: "A".to_string() },
        ]);
    }

    #[test]
    fn get_key_item_definitions_is_empty_before_any_are_created() {
        let conn = setup();
        assert_eq!(get_key_item_definitions(&conn).unwrap(), vec![]);
    }

    #[test]
    fn create_key_item_definition_stores_a_denormalized_name_and_optional_npc() {
        let conn = setup();
        create_key_item_definition(&conn, 42, "Mystical Canteen", Some("Incantrix"), 72000).unwrap();

        let defs = get_key_item_definitions(&conn).unwrap();
        assert_eq!(defs.len(), 1);
        assert_eq!(defs[0].key_item_id, 42);
        assert_eq!(defs[0].name, "Mystical Canteen");
        assert_eq!(defs[0].granting_npc, Some("Incantrix".to_string()));
        assert_eq!(defs[0].cooldown_duration_seconds, 72000);
    }

    #[test]
    fn create_key_item_definition_allows_no_granting_npc() {
        let conn = setup();
        create_key_item_definition(&conn, 42, "Mystical Canteen", None, 72000).unwrap();
        assert_eq!(get_key_item_definitions(&conn).unwrap()[0].granting_npc, None);
    }

    #[test]
    fn delete_key_item_definition_removes_it() {
        let conn = setup();
        let id = create_key_item_definition(&conn, 42, "Mystical Canteen", None, 72000).unwrap();
        delete_key_item_definition(&conn, id).unwrap();
        assert_eq!(get_key_item_definitions(&conn).unwrap(), vec![]);
    }

    #[test]
    fn delete_key_item_definition_cascades_cooldown_rows_across_characters() {
        let conn = setup();
        let char_a = make_character(&conn, 1, "Gozoto");
        let char_b = make_character(&conn, 2, "Zootog");
        let id = create_key_item_definition(&conn, 42, "Mystical Canteen", None, 72000).unwrap();
        report_key_items_held(&conn, char_a, &[], "2026-09-14T00:00:00Z").unwrap();
        report_key_items_held(&conn, char_b, &[], "2026-09-14T00:00:00Z").unwrap();
        report_key_items_held(&conn, char_a, &[42], "2026-09-15T00:00:00Z").unwrap();
        report_key_items_held(&conn, char_b, &[42], "2026-09-15T00:00:00Z").unwrap();
        assert_eq!(get_key_item_tracking(&conn, char_a).unwrap()[0].last_acquired_at, Some("2026-09-15T00:00:00Z".to_string()));

        delete_key_item_definition(&conn, id).unwrap();

        // No rows left behind for either character, and a re-created definition
        // (new id) doesn't inherit a stale cooldown from the deleted one.
        assert_eq!(get_key_item_tracking(&conn, char_a).unwrap(), vec![]);
        assert_eq!(get_key_item_tracking(&conn, char_b).unwrap(), vec![]);
    }

    #[test]
    fn get_character_key_items_held_is_empty_for_unknown_character() {
        let conn = setup();
        assert_eq!(get_character_key_items_held(&conn, 1).unwrap(), Vec::<i64>::new());
    }

    #[test]
    fn report_key_items_held_first_ever_report_is_a_baseline_not_a_transition() {
        let conn = setup();
        let character_id = make_character(&conn, 1, "Gozoto");
        create_key_item_definition(&conn, 42, "Mystical Canteen", None, 72000).unwrap();

        // Character already possesses the tracked key item on the very first
        // report this app ever receives for them — must NOT be treated as a
        // fresh acquisition (spec §5 step 2).
        report_key_items_held(&conn, character_id, &[42], "2026-09-15T00:00:00Z").unwrap();

        assert_eq!(get_character_key_items_held(&conn, character_id).unwrap(), vec![42]);
        let tracking = get_key_item_tracking(&conn, character_id).unwrap();
        assert_eq!(tracking[0].last_acquired_at, None);
        assert!(tracking[0].currently_held);
    }

    #[test]
    fn report_key_items_held_detects_a_genuine_0_to_1_transition() {
        let conn = setup();
        let character_id = make_character(&conn, 1, "Gozoto");
        create_key_item_definition(&conn, 42, "Mystical Canteen", None, 72000).unwrap();

        report_key_items_held(&conn, character_id, &[], "2026-09-15T00:00:00Z").unwrap(); // baseline: doesn't have it yet
        report_key_items_held(&conn, character_id, &[42], "2026-09-15T06:00:00Z").unwrap(); // just received it

        let tracking = get_key_item_tracking(&conn, character_id).unwrap();
        assert_eq!(tracking[0].last_acquired_at, Some("2026-09-15T06:00:00Z".to_string()));
    }

    #[test]
    fn report_key_items_held_present_in_both_snapshots_is_not_a_transition() {
        let conn = setup();
        let character_id = make_character(&conn, 1, "Gozoto");
        create_key_item_definition(&conn, 42, "Mystical Canteen", None, 72000).unwrap();

        report_key_items_held(&conn, character_id, &[42], "2026-09-15T00:00:00Z").unwrap(); // baseline, already held
        report_key_items_held(&conn, character_id, &[42], "2026-09-15T06:00:00Z").unwrap(); // still held, unchanged

        // Baseline already skipped the first report; unchanged possession on
        // the second must not retroactively fabricate an acquisition time either.
        assert_eq!(get_key_item_tracking(&conn, character_id).unwrap()[0].last_acquired_at, None);
    }

    #[test]
    fn report_key_items_held_loss_does_not_start_or_clear_a_cooldown() {
        let conn = setup();
        let character_id = make_character(&conn, 1, "Gozoto");
        create_key_item_definition(&conn, 42, "Mystical Canteen", None, 72000).unwrap();

        report_key_items_held(&conn, character_id, &[], "2026-09-15T00:00:00Z").unwrap(); // baseline
        report_key_items_held(&conn, character_id, &[42], "2026-09-15T06:00:00Z").unwrap(); // received
        report_key_items_held(&conn, character_id, &[], "2026-09-15T07:00:00Z").unwrap(); // used/lost

        let tracking = get_key_item_tracking(&conn, character_id).unwrap();
        // last_acquired_at is confirmed with the user to start on receipt, not
        // reset by later loss (spec §5) — the cooldown clock keeps running.
        assert_eq!(tracking[0].last_acquired_at, Some("2026-09-15T06:00:00Z".to_string()));
        assert!(!tracking[0].currently_held);
    }

    #[test]
    fn report_key_items_held_only_evaluates_transitions_for_tracked_definitions() {
        let conn = setup();
        let character_id = make_character(&conn, 1, "Gozoto");
        create_key_item_definition(&conn, 42, "Mystical Canteen", None, 72000).unwrap();

        report_key_items_held(&conn, character_id, &[], "2026-09-15T00:00:00Z").unwrap(); // baseline
        // Newly-held item 999 has no tracked definition — must not error or
        // create a cooldown row for anything.
        report_key_items_held(&conn, character_id, &[999], "2026-09-15T06:00:00Z").unwrap();

        assert_eq!(get_key_item_tracking(&conn, character_id).unwrap()[0].last_acquired_at, None);
    }

    #[test]
    fn report_key_items_held_replaces_the_held_snapshot_wholesale() {
        let conn = setup();
        let character_id = make_character(&conn, 1, "Gozoto");
        report_key_items_held(&conn, character_id, &[1, 2], "2026-09-15T00:00:00Z").unwrap();
        report_key_items_held(&conn, character_id, &[1], "2026-09-15T06:00:00Z").unwrap();
        assert_eq!(get_character_key_items_held(&conn, character_id).unwrap(), vec![1]);
    }

    #[test]
    fn report_key_items_held_is_isolated_per_character() {
        let conn = setup();
        let char_a = make_character(&conn, 1, "Gozoto");
        let char_b = make_character(&conn, 2, "Zootog");
        create_key_item_definition(&conn, 42, "Mystical Canteen", None, 72000).unwrap();

        report_key_items_held(&conn, char_a, &[], "2026-09-15T00:00:00Z").unwrap();
        report_key_items_held(&conn, char_b, &[], "2026-09-15T00:00:00Z").unwrap();
        report_key_items_held(&conn, char_a, &[42], "2026-09-15T06:00:00Z").unwrap(); // only A receives it

        assert_eq!(get_key_item_tracking(&conn, char_a).unwrap()[0].last_acquired_at, Some("2026-09-15T06:00:00Z".to_string()));
        assert_eq!(get_key_item_tracking(&conn, char_b).unwrap()[0].last_acquired_at, None);
        assert!(!get_key_item_tracking(&conn, char_b).unwrap()[0].currently_held);
    }

    #[test]
    fn get_key_item_tracking_is_empty_when_nothing_is_tracked() {
        let conn = setup();
        let character_id = make_character(&conn, 1, "Gozoto");
        assert_eq!(get_key_item_tracking(&conn, character_id).unwrap(), vec![]);
    }

    #[test]
    fn get_key_item_tracking_ready_state_when_never_held_and_no_acquisition() {
        let conn = setup();
        let character_id = make_character(&conn, 1, "Gozoto");
        create_key_item_definition(&conn, 42, "Mystical Canteen", Some("Incantrix"), 72000).unwrap();
        report_key_items_held(&conn, character_id, &[], "2026-09-15T00:00:00Z").unwrap();

        let row = &get_key_item_tracking(&conn, character_id).unwrap()[0];
        assert_eq!(row.last_acquired_at, None);
        assert!(!row.currently_held);
        assert_eq!(row.granting_npc, Some("Incantrix".to_string()));
    }

    #[test]
    fn get_key_item_tracking_on_cooldown_state_carries_last_acquired_at_and_held() {
        let conn = setup();
        let character_id = make_character(&conn, 1, "Gozoto");
        create_key_item_definition(&conn, 42, "Mystical Canteen", None, 72000).unwrap();
        report_key_items_held(&conn, character_id, &[], "2026-09-15T00:00:00Z").unwrap();
        report_key_items_held(&conn, character_id, &[42], "2026-09-15T06:00:00Z").unwrap();

        let row = &get_key_item_tracking(&conn, character_id).unwrap()[0];
        assert_eq!(row.last_acquired_at, Some("2026-09-15T06:00:00Z".to_string()));
        assert!(row.currently_held);
        assert_eq!(row.cooldown_duration_seconds, 72000);
    }

    #[test]
    fn get_key_item_tracking_held_before_tracking_started_state() {
        let conn = setup();
        let character_id = make_character(&conn, 1, "Gozoto");
        // Character already holds key item 42 before this definition ever existed.
        report_key_items_held(&conn, character_id, &[42], "2026-09-15T00:00:00Z").unwrap();
        create_key_item_definition(&conn, 42, "Mystical Canteen", None, 72000).unwrap();
        // Next report still shows it held — nothing changed, so no transition.
        report_key_items_held(&conn, character_id, &[42], "2026-09-15T06:00:00Z").unwrap();

        let row = &get_key_item_tracking(&conn, character_id).unwrap()[0];
        assert_eq!(row.last_acquired_at, None); // genuinely unknown, not guessed
        assert!(row.currently_held);
    }

    #[test]
    fn get_key_item_tracking_only_returns_rows_for_the_given_character() {
        let conn = setup();
        let char_a = make_character(&conn, 1, "Gozoto");
        let char_b = make_character(&conn, 2, "Zootog");
        create_key_item_definition(&conn, 42, "Mystical Canteen", None, 72000).unwrap();
        report_key_items_held(&conn, char_a, &[42], "2026-09-15T00:00:00Z").unwrap();

        assert_eq!(get_key_item_tracking(&conn, char_a).unwrap().len(), 1);
        assert_eq!(get_key_item_tracking(&conn, char_b).unwrap().len(), 1); // tracked globally, both show up
        assert!(!get_key_item_tracking(&conn, char_b).unwrap()[0].currently_held);
    }
}
