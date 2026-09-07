use rusqlite::{Connection, Result};

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
}
