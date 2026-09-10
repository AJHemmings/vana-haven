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
}
