//! IPC-слой DEVNOTES (Tauri 2).
//!
//! Тонкая прослойка: каждая команда маршалит вызов в [`devnotes_core::service::NotesService`]
//! и приводит доменную ошибку к строке для передачи во фронтенд. Бизнес-логики здесь нет
//! (см. `docs/04-ARCHITECTURE.md`, раздел про границу IPC ↔ UseCases).

use std::sync::Mutex;

use devnotes_core::domain::{NoteContent, NoteSeries, Project, SearchHit};
use devnotes_core::ports::{SystemClock, UuidV7Generator};
use devnotes_core::service::NotesService;
use devnotes_core::sqlite::SqliteStore;
use tauri::{Manager, State};

/// Глобальное состояние приложения: сервис за мьютексом (SQLite-соединение
/// не Sync, поэтому доступ сериализуется). Для локального однопользовательского
/// приложения этого достаточно.
struct AppState(Mutex<NotesService>);

/// Тип результата команд: ошибка отдаётся строкой (её покажет UI через toast).
type CmdResult<T> = Result<T, String>;

/// Хелпер: привести доменную ошибку к строке.
fn to_err<E: std::fmt::Display>(e: E) -> String {
    e.to_string()
}

// --- Команды: проекты ------------------------------------------------------

#[tauri::command]
fn list_projects(state: State<AppState>) -> CmdResult<Vec<Project>> {
    let svc = state.0.lock().map_err(to_err)?;
    svc.list_projects().map_err(to_err)
}

#[tauri::command]
fn create_project(
    state: State<AppState>,
    name: String,
    short_name: Option<String>,
    description: Option<String>,
) -> CmdResult<Project> {
    let svc = state.0.lock().map_err(to_err)?;
    svc.create_project(&name, short_name.as_deref(), description.as_deref())
        .map_err(to_err)
}

#[tauri::command]
fn delete_project(state: State<AppState>, id: String) -> CmdResult<()> {
    let svc = state.0.lock().map_err(to_err)?;
    svc.delete_project(&id).map_err(to_err)
}

// --- Команды: серии --------------------------------------------------------

#[tauri::command]
fn list_series(state: State<AppState>, project_id: Option<String>) -> CmdResult<Vec<NoteSeries>> {
    let svc = state.0.lock().map_err(to_err)?;
    svc.list_series(project_id.as_deref()).map_err(to_err)
}

#[tauri::command]
fn create_series(
    state: State<AppState>,
    project_id: Option<String>,
    title: String,
    description: Option<String>,
) -> CmdResult<NoteSeries> {
    let svc = state.0.lock().map_err(to_err)?;
    svc.create_series(project_id.as_deref(), &title, description.as_deref())
        .map_err(to_err)
}

#[tauri::command]
fn delete_series(state: State<AppState>, id: String) -> CmdResult<()> {
    let svc = state.0.lock().map_err(to_err)?;
    svc.delete_series(&id).map_err(to_err)
}

// --- Команды: блоки контента ----------------------------------------------

#[tauri::command]
fn list_content(state: State<AppState>, series_id: String) -> CmdResult<Vec<NoteContent>> {
    let svc = state.0.lock().map_err(to_err)?;
    svc.list_content(&series_id).map_err(to_err)
}

#[tauri::command]
fn add_content(
    state: State<AppState>,
    series_id: String,
    title: Option<String>,
    text: String,
    content_type: String,
) -> CmdResult<NoteContent> {
    let svc = state.0.lock().map_err(to_err)?;
    svc.add_content(&series_id, title.as_deref(), &text, &content_type)
        .map_err(to_err)
}

#[tauri::command]
fn update_content(
    state: State<AppState>,
    id: String,
    title: Option<String>,
    text: String,
    content_type: String,
) -> CmdResult<()> {
    let svc = state.0.lock().map_err(to_err)?;
    svc.update_content(&id, title.as_deref(), &text, &content_type)
        .map_err(to_err)
}

#[tauri::command]
fn delete_content(state: State<AppState>, id: String) -> CmdResult<()> {
    let svc = state.0.lock().map_err(to_err)?;
    svc.delete_content(&id).map_err(to_err)
}

#[tauri::command]
fn reorder_content(state: State<AppState>, ordered_ids: Vec<String>) -> CmdResult<()> {
    let svc = state.0.lock().map_err(to_err)?;
    let refs: Vec<&str> = ordered_ids.iter().map(String::as_str).collect();
    svc.reorder_content(&refs).map_err(to_err)
}

// --- Команды: поиск --------------------------------------------------------

#[tauri::command]
fn search(state: State<AppState>, query: String, limit: Option<i64>) -> CmdResult<Vec<SearchHit>> {
    let svc = state.0.lock().map_err(to_err)?;
    svc.search(&query, limit.unwrap_or(50)).map_err(to_err)
}

/// Инициализирует хранилище в каталоге данных приложения и запускает окно.
///
/// БД лежит в `app_data_dir/devnotes.sqlite`; при первом запуске применяются миграции.
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Каталог данных приложения (кроссплатформенный).
            let dir = app
                .path()
                .app_data_dir()
                .expect("не удалось определить каталог данных приложения");
            std::fs::create_dir_all(&dir).expect("не удалось создать каталог данных");
            let db_path = dir.join("devnotes.sqlite");

            let store = SqliteStore::open(
                db_path.to_str().expect("путь к БД не является UTF-8"),
                Box::new(SystemClock),
                Box::new(UuidV7Generator),
            )
            .expect("не удалось открыть БД");
            store.migrate().expect("не удалось применить миграции");

            app.manage(AppState(Mutex::new(NotesService::new(store))));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_projects,
            create_project,
            delete_project,
            list_series,
            create_series,
            delete_series,
            list_content,
            add_content,
            update_content,
            delete_content,
            reorder_content,
            search,
        ])
        .run(tauri::generate_context!())
        .expect("ошибка запуска приложения DEVNOTES");
}
