mod db;
mod sync;

use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use tauri::Manager;

struct AppState {
    db: Arc<Mutex<Connection>>,
}

#[tauri::command]
fn get_characters(state: tauri::State<AppState>) -> Result<Vec<db::Character>, String> {
    let conn = state.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    db::list_characters(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_character(
    game_character_id: i64,
    state: tauri::State<AppState>,
) -> Result<Option<db::CharacterDetail>, String> {
    let conn = state.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    db::get_character(&conn, game_character_id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_character_jobs(
    game_character_id: i64,
    state: tauri::State<AppState>,
) -> Result<Vec<db::JobLevel>, String> {
    let conn = state.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let character_id = db::resolve_character_id(&conn, game_character_id).map_err(|e| e.to_string())?;
    match character_id {
        Some(id) => db::get_character_jobs(&conn, id).map_err(|e| e.to_string()),
        None => Ok(vec![]),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;

            let db_path = app_data_dir.join("vana-haven.db");
            let conn = Connection::open(db_path)?;
            db::init_db(&conn)?;
            let db = Arc::new(Mutex::new(conn));
            app.manage(AppState { db: Arc::clone(&db) });

            let port_file = app_data_dir.join("port.txt");
            let std_listener = sync::port::bind_with_fallback(&port_file)?;
            std_listener.set_nonblocking(true)?;
            // `setup()` runs on the main thread outside of Tauri's managed Tokio
            // runtime, but `TcpListener::from_std` needs a runtime context to
            // register the socket with the reactor (it panics otherwise:
            // "there is no reactor running"). `block_on` makes that runtime
            // current on this thread for the duration of the call.
            let listener = tauri::async_runtime::block_on(async {
                tokio::net::TcpListener::from_std(std_listener)
            })?;
            sync::spawn_listener(listener, db, app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_characters, get_character, get_character_jobs])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
