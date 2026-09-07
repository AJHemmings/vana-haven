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
            let Ok((stream, _)) = listener.accept().await else { continue };
            let db = Arc::clone(&db);
            let app = app.clone();
            tauri::async_runtime::spawn(handle_connection(stream, db, app));
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
                let ok = {
                    let conn = db.lock().unwrap();
                    db::upsert_character(&conn, &character).is_ok()
                };
                if ok {
                    let _ = app.emit("character-updated", character.game_character_id);
                }
            }
            Ok(AddonMessage::Heartbeat { game_character_id }) => {
                let conn = db.lock().unwrap();
                let _ = db::touch_character(&conn, game_character_id, &now);
            }
            Err(_) => {
                // Malformed line from the addon: skip it, keep the connection open
                // rather than dropping the whole session over one bad message.
            }
        }
    }
}
