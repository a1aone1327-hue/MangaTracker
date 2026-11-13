//! Тонкий слой между интерфейсом и ядром: команды без собственной логики.
//! Все правила живут в `manga-core`; здесь только открываем базу и пробрасываем вызовы.

use std::io::Read;
use std::sync::Mutex;
use std::time::Duration;

use manga_core::data::Store;
use base64::Engine as _;
use manga_core::domain::remote::{parse_link, parse_title_info, LibLink, TitleInfo};
use manga_core::domain::{
    Counter, Group, ImportMode, ImportReport, Library, Medium, OnDelete, Title, TitleDraft,
};
use tauri::{Manager, State};

/// Если базу открыть не удалось (например, она от более новой версии приложения),
/// приложение всё равно запускается и показывает причину, а файл базы не трогается.
struct AppState {
    store: Mutex<Result<Store, String>>,
}

/// Берёт базу под замок, выполняет операцию, превращает ошибку ядра в текст для интерфейса.
fn with_store<T>(
    state: &State<'_, AppState>,
    f: impl FnOnce(&mut Store) -> manga_core::Result<T>,
) -> Result<T, String> {
    let mut guard = state
        .store
        .lock()
        .map_err(|_| "Внутренняя ошибка: база недоступна".to_string())?;
    match guard.as_mut() {
        Ok(store) => f(store).map_err(|e| e.to_string()),
        Err(message) => Err(message.clone()),
    }
}

#[tauri::command]
fn list_groups(state: State<'_, AppState>, medium: Medium) -> Result<Vec<Group>, String> {
    with_store(&state, |s| s.list_groups(medium))
}

#[tauri::command]
fn list_titles(state: State<'_, AppState>, medium: Medium) -> Result<Vec<Title>, String> {
    with_store(&state, |s| s.list_titles(medium))
}

#[tauri::command]
fn create_group(
    state: State<'_, AppState>,
    medium: Medium,
    name: String,
    color: String,
) -> Result<Group, String> {
    with_store(&state, |s| s.create_group(medium, &name, &color))
}

#[tauri::command]
fn update_group(
    state: State<'_, AppState>,
    id: i64,
    name: String,
    color: String,
) -> Result<Group, String> {
    with_store(&state, |s| s.update_group(id, &name, &color))
}

#[tauri::command]
fn reorder_groups(
    state: State<'_, AppState>,
    medium: Medium,
    ordered_ids: Vec<i64>,
) -> Result<(), String> {
    with_store(&state, |s| s.reorder_groups(medium, &ordered_ids))
}

/// Возвращает, сколько тайтлов затронуто (перенесено или оставлено без группы).
#[tauri::command]
fn delete_group(
    state: State<'_, AppState>,
    id: i64,
    on_delete: OnDelete,
) -> Result<usize, String> {
    with_store(&state, |s| s.delete_group(id, on_delete))
}

#[tauri::command]
fn create_title(state: State<'_, AppState>, draft: TitleDraft) -> Result<Title, String> {
    with_store(&state, |s| s.create_title(draft))
}

#[tauri::command]
fn update_title(
    state: State<'_, AppState>,
    id: i64,
    draft: TitleDraft,
) -> Result<Title, String> {
    with_store(&state, |s| s.update_title(id, draft))
}

/// Быстрые кнопки +1/−1 (глава, том или серия).
#[tauri::command]
fn step_progress(
    state: State<'_, AppState>,
    id: i64,
    counter: Counter,
    delta: i32,
) -> Result<Title, String> {
    with_store(&state, |s| s.step_progress(id, counter, delta))
}

/// `group_id = null` значит «без группы».
#[tauri::command]
fn move_titles(
    state: State<'_, AppState>,
    ids: Vec<i64>,
    group_id: Option<i64>,
) -> Result<usize, String> {
    with_store(&state, |s| s.move_titles(&ids, group_id))
}

#[tauri::command]
fn set_library(
    state: State<'_, AppState>,
    ids: Vec<i64>,
    library: Library,
) -> Result<usize, String> {
    with_store(&state, |s| s.set_library(&ids, library))
}

#[tauri::command]
fn delete_titles(state: State<'_, AppState>, ids: Vec<i64>) -> Result<usize, String> {
    with_store(&state, |s| s.delete_titles(&ids))
}

/// Библиотека по ссылке (нужна форме, чтобы сразу подставить MangaLib/HentaiLib/AnimeLib).
#[tauri::command]
fn detect_library(url: String) -> Option<Library> {
    Library::detect(&url)
}

/// Имя файла копии: только безопасные символы, расширение .json.
fn safe_file_name(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .take(80)
        .collect();
    let base = cleaned.trim_matches('.');
    if base.is_empty() {
        "manga-tracker.json".to_string()
    } else if base.to_lowercase().ends_with(".json") {
        base.to_string()
    } else {
        format!("{base}.json")
    }
}

/// Собирает копию всей базы в файл во внутренней папке приложения и возвращает путь.
/// Дальше интерфейс показывает системное меню «Поделиться» с этим файлом.
#[tauri::command]
fn export_backup(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    file_name: String,
) -> Result<String, String> {
    let json = with_store(&state, |s| s.export_json())?;
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join("exports");
    std::fs::create_dir_all(&dir).map_err(|e| format!("Не удалось подготовить файл: {e}"))?;
    // старые копии не копим: они могут быть большими
    if let Ok(entries) = std::fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let _ = std::fs::remove_file(entry.path());
        }
    }
    let path = dir.join(safe_file_name(&file_name));
    std::fs::write(&path, json).map_err(|e| format!("Не удалось сохранить файл: {e}"))?;
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command]
fn import_backup(
    state: State<'_, AppState>,
    json: String,
    mode: ImportMode,
) -> Result<ImportReport, String> {
    with_store(&state, |s| s.import_json(&json, mode))
}

// ---------- подтягивание данных с сайтов (единственное место, где приложение ходит в сеть) ----------

const USER_AGENT: &str =
    "Mozilla/5.0 (Linux; Android 14) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0 Mobile Safari/537.36";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(12);
const MAX_COVER_DOWNLOAD: u64 = 3_000_000;

/// Данные с сайта плюс обложка, скачанная в виде data URL (уменьшит её интерфейс).
#[derive(serde::Serialize)]
struct FetchedTitle {
    #[serde(flatten)]
    info: TitleInfo,
    cover_data: Option<String>,
}

fn http_get(url: &str, link: &LibLink, accept: &str) -> Option<ureq::Response> {
    ureq::get(url)
        .timeout(REQUEST_TIMEOUT)
        .set("User-Agent", USER_AGENT)
        .set("Accept", accept)
        .set("Site-Id", &link.site_id.to_string())
        .set("Referer", &format!("{}/", link.origin))
        .call()
        .ok()
}

fn download_cover(url: &str, link: &LibLink) -> Option<String> {
    let response = http_get(url, link, "image/*")?;
    let mime = response
        .header("Content-Type")
        .and_then(|v| v.split(';').next())
        .map(|v| v.trim().to_string())
        .unwrap_or_else(|| "image/jpeg".to_string());
    if !mime.starts_with("image/") {
        return None;
    }
    let mut bytes = Vec::new();
    response.into_reader().take(MAX_COVER_DOWNLOAD).read_to_end(&mut bytes).ok()?;
    if bytes.is_empty() {
        return None;
    }
    let encoded = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:{mime};base64,{encoded}"))
}

fn fetch_blocking(url: &str) -> Option<FetchedTitle> {
    let link = parse_link(url)?;
    let body = http_get(&link.api_url, &link, "application/json")?.into_string().ok()?;
    let info = parse_title_info(&body, &link)?;
    let cover_data = info.cover_url.as_deref().and_then(|u| download_cover(u, &link));
    Some(FetchedTitle { info, cover_data })
}

/// По ссылке на страницу тайтла (MangaLib, HentaiLib, AnimeLib) пробует получить название,
/// количество глав, тип и обложку. Любая неудача (нет сети, сайт изменился, ссылка
/// незнакомая) даёт `null`: интерфейс просто ничего не заполняет.
#[tauri::command]
async fn fetch_title_info(url: String) -> Option<FetchedTitle> {
    tauri::async_runtime::spawn_blocking(move || fetch_blocking(&url))
        .await
        .ok()
        .flatten()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_sharekit::init())
        .setup(|app| {
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let store = Store::open(&dir.join("manga.db")).map_err(|e| e.to_string());
            app.manage(AppState { store: Mutex::new(store) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_groups,
            list_titles,
            create_group,
            update_group,
            reorder_groups,
            delete_group,
            create_title,
            update_title,
            step_progress,
            move_titles,
            set_library,
            delete_titles,
            detect_library,
            export_backup,
            import_backup,
            fetch_title_info,
        ])
        .run(tauri::generate_context!())
        .expect("не удалось запустить приложение");
}

#[cfg(test)]
mod tests {
    use super::safe_file_name;

    #[test]
    fn file_names_are_made_safe() {
        assert_eq!(safe_file_name("manga-tracker-2026-11-12.json"), "manga-tracker-2026-11-12.json");
        assert_eq!(safe_file_name("../../etc/passwd"), "_.._etc_passwd.json");
        assert_eq!(safe_file_name("копия"), "_____.json");
        assert_eq!(safe_file_name(""), "manga-tracker.json");
        assert_eq!(safe_file_name("..."), "manga-tracker.json");
        assert_eq!(safe_file_name("backup"), "backup.json");
    }
}
