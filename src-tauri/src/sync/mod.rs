pub mod port;
pub mod protocol;

use crate::db;
use protocol::AddonMessage;
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

pub fn spawn_listener(listener: TcpListener, db: Arc<Mutex<Connection>>, app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    let db = Arc::clone(&db);
                    let app = app.clone();
                    tauri::async_runtime::spawn(handle_connection(stream, db, app));
                }
                Err(e) => {
                    // Un-throttled retry here would busy-spin a CPU core right
                    // while the user is playing FFXI — the one time this app
                    // must not cost frame time. Log and back off instead.
                    eprintln!("[vana-haven] accept() failed: {e}");
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
            }
        }
    });
}

async fn handle_connection(stream: TcpStream, db: Arc<Mutex<Connection>>, app: AppHandle) {
    let reader = BufReader::new(stream);
    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if line.trim().is_empty() {
            continue;
        }
        let now = chrono::Utc::now().to_rfc3339();
        match protocol::parse_message(&line) {
            Ok(AddonMessage::Handshake { game_character_id, name }) => {
                let character = db::Character { game_character_id, name, last_seen_at: now };
                let result = {
                    let conn = db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                    db::upsert_character(&conn, &character)
                };
                match result {
                    Ok(()) => {
                        let _ = app.emit("character-updated", character.game_character_id);
                    }
                    Err(e) => eprintln!(
                        "[vana-haven] failed to save character {}: {e}",
                        character.game_character_id
                    ),
                }
            }
            Ok(AddonMessage::Heartbeat { game_character_id }) => {
                let conn = db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                if let Err(e) = db::touch_character(&conn, game_character_id, &now) {
                    eprintln!("[vana-haven] failed to update heartbeat for {game_character_id}: {e}");
                }
            }
            Ok(AddonMessage::JobLevels { game_character_id, main_job_id, sub_job_id, jobs }) => {
                let conn = db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
                match db::resolve_character_id(&conn, game_character_id) {
                    Ok(Some(character_id)) => {
                        let job_levels: Vec<db::JobLevel> = jobs
                            .into_iter()
                            .map(|j| db::JobLevel {
                                job_id: j.job_id,
                                level: j.level,
                                master_level: j.master_level,
                                mastered: j.mastered,
                            })
                            .collect();
                        match db::replace_character_jobs(&conn, character_id, &job_levels) {
                            Ok(()) => match db::update_character_jobs_summary(&conn, character_id, main_job_id, sub_job_id) {
                                Ok(()) => { let _ = app.emit("character-updated", game_character_id); }
                                Err(e) => eprintln!("[vana-haven] failed to save job summary for {game_character_id}: {e}"),
                            },
                            Err(e) => eprintln!("[vana-haven] failed to save job levels for {game_character_id}: {e}"),
                        }
                    }
                    Ok(None) => {
                        // A job_levels message arrived before this character's first
                        // Handshake — shouldn't normally happen (the addon always
                        // handshakes first on connect) but isn't structurally
                        // prevented. Same "log and skip" treatment as a malformed
                        // message, not a crash.
                        eprintln!("[vana-haven] ignoring job_levels for unknown character {game_character_id}");
                    }
                    Err(e) => eprintln!("[vana-haven] failed to resolve character {game_character_id}: {e}"),
                }
            }
            Err(e) => {
                // Skip the bad line and keep the connection open rather than
                // dropping the whole session over one malformed message — but
                // log it, since a real-addon encoding mismatch (Task 12) with
                // silent failure here is nearly undebuggable otherwise.
                eprintln!("[vana-haven] ignoring malformed message: {e} (raw: {line})");
            }
        }
    }
}
