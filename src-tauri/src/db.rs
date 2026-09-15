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
}
