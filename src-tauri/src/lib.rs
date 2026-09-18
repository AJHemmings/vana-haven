mod db;
mod gear_reference;
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

#[derive(serde::Serialize)]
struct GearProgression {
    definitions: Vec<db::GearSetDefinitionRow>,
    current_tiers: Vec<db::SlotTier>,
}

#[tauri::command]
fn get_gear_progression(
    game_character_id: i64,
    job_id: i64,
    state: tauri::State<AppState>,
) -> Result<GearProgression, String> {
    let conn = state.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let definitions = db::get_gear_set_definitions(&conn, job_id).map_err(|e| e.to_string())?;
    let character_id = db::resolve_character_id(&conn, game_character_id).map_err(|e| e.to_string())?;
    let current_tiers = match character_id {
        Some(id) => db::compute_current_tiers(&conn, id, job_id).map_err(|e| e.to_string())?,
        None => vec![],
    };
    Ok(GearProgression { definitions, current_tiers })
}

#[tauri::command]
fn get_key_item_catalog(state: tauri::State<AppState>) -> Result<Vec<db::KeyItemCatalogEntry>, String> {
    let conn = state.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    db::get_key_item_catalog(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_key_item_definition(
    key_item_id: i64,
    name: String,
    granting_npc: Option<String>,
    cooldown_duration_seconds: i64,
    state: tauri::State<AppState>,
) -> Result<i64, String> {
    let conn = state.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    db::create_key_item_definition(&conn, key_item_id, &name, granting_npc.as_deref(), cooldown_duration_seconds)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_key_item_definition(id: i64, state: tauri::State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    db::delete_key_item_definition(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_key_item_tracking(
    game_character_id: i64,
    state: tauri::State<AppState>,
) -> Result<Vec<db::KeyItemTracking>, String> {
    let conn = state.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let character_id = db::resolve_character_id(&conn, game_character_id).map_err(|e| e.to_string())?;
    match character_id {
        Some(id) => db::get_key_item_tracking(&conn, id).map_err(|e| e.to_string()),
        None => Ok(vec![]),
    }
}

#[tauri::command]
fn get_daily_todo_items(
    game_character_id: i64,
    state: tauri::State<AppState>,
) -> Result<Vec<db::DailyTodoItem>, String> {
    let conn = state.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let character_id = db::resolve_character_id(&conn, game_character_id).map_err(|e| e.to_string())?;
    match character_id {
        Some(id) => db::get_daily_todo_items(&conn, id).map_err(|e| e.to_string()),
        None => Ok(vec![]),
    }
}

#[tauri::command]
fn create_daily_todo_item(
    game_character_id: i64,
    text: String,
    cadence: String,
    state: tauri::State<AppState>,
) -> Result<i64, String> {
    let conn = state.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let character_id = db::resolve_character_id(&conn, game_character_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "unknown character".to_string())?;
    let now = chrono::Utc::now().to_rfc3339();
    db::create_daily_todo_item(&conn, character_id, &text, &cadence, &now).map_err(|e| e.to_string())
}

#[tauri::command]
fn delete_daily_todo_item(id: i64, state: tauri::State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    db::delete_daily_todo_item(&conn, id).map_err(|e| e.to_string())
}

#[tauri::command]
fn set_daily_todo_completed(id: i64, completed: bool, state: tauri::State<AppState>) -> Result<(), String> {
    let conn = state.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let completed_at = if completed { Some(chrono::Utc::now().to_rfc3339()) } else { None };
    db::set_daily_todo_completion(&conn, id, completed_at.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_monthly_cycle_started_at(state: tauri::State<AppState>) -> Result<Option<String>, String> {
    let conn = state.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    db::get_monthly_cycle_started_at(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn advance_monthly_cycle(state: tauri::State<AppState>) -> Result<String, String> {
    let conn = state.db.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let now = chrono::Utc::now().to_rfc3339();
    db::advance_monthly_cycle(&conn, &now).map_err(|e| e.to_string())?;
    Ok(now)
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

            let af3_json = include_str!("../../tools/scraper/out/gearsets-af3.json");
            let empyrean_json = include_str!("../../tools/scraper/out/gearsets-empyrean.json");
            let relic_json = include_str!("../../tools/scraper/out/gearsets-relic.json");
            let mut gear_rows = gear_reference::parse_gear_set_definitions(af3_json)?;
            gear_rows.extend(gear_reference::parse_gear_set_definitions(empyrean_json)?);
            gear_rows.extend(gear_reference::parse_gear_set_definitions(relic_json)?);
            db::seed_gear_set_definitions_if_empty(&conn, &gear_rows)?;

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
        .invoke_handler(tauri::generate_handler![
            get_characters,
            get_character,
            get_character_jobs,
            get_gear_progression,
            get_key_item_catalog,
            create_key_item_definition,
            delete_key_item_definition,
            get_key_item_tracking,
            get_daily_todo_items,
            create_daily_todo_item,
            delete_daily_todo_item,
            set_daily_todo_completed,
            get_monthly_cycle_started_at,
            advance_monthly_cycle
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
