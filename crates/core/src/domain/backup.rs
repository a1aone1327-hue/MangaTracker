//! Формат копии данных (экспорт и импорт JSON) и правила слияния дубликатов.

use serde::{Deserialize, Serialize};

use super::model::{Group, Title, TitleDraft};

pub const BACKUP_FORMAT: &str = "manga-tracker-backup";
pub const BACKUP_VERSION: u32 = 1;

/// Копия всей базы. `id` групп в файле нужны только чтобы связать с ними тайтлы:
/// при импорте группы и тайтлы получают новые id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Backup {
    pub format: String,
    pub version: u32,
    pub exported_at: i64,
    #[serde(default)]
    pub groups: Vec<Group>,
    #[serde(default)]
    pub titles: Vec<Title>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImportMode {
    /// Удалить всё текущее и заменить содержимым файла.
    Replace,
    /// Добавить новое, дубликаты объединить, ничего не удалять.
    Merge,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct ImportReport {
    pub groups_added: usize,
    pub titles_added: usize,
    pub titles_merged: usize,
    pub titles_skipped: usize,
}

fn collapse(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Ключ для поиска дубликата по названию: раздел, без регистра, «ё» как «е».
pub fn name_key(d: &TitleDraft) -> String {
    format!("{}:{}", d.medium.as_str(), collapse(&d.name.to_lowercase().replace('ё', "е")))
}

/// Ключ по ссылке: без схемы, `www.`, параметров и хвостового слэша.
pub fn url_key(d: &TitleDraft) -> Option<String> {
    let raw = d.url.as_deref()?.trim().to_lowercase();
    let rest = raw
        .strip_prefix("https://")
        .or_else(|| raw.strip_prefix("http://"))
        .unwrap_or(&raw);
    let rest = rest.strip_prefix("www.").unwrap_or(rest);
    let cut = rest.find(|c: char| c == '?' || c == '#').unwrap_or(rest.len());
    let path = rest[..cut].trim_end_matches('/');
    if path.is_empty() {
        None
    } else {
        Some(format!("{}:{}", d.medium.as_str(), path))
    }
}

/// «Всего» берём большее из двух, но если оно меньше уже пройденного, считаем его неизвестным.
fn merge_total(a: Option<u32>, b: Option<u32>, done: u32) -> Option<u32> {
    let merged = match (a, b) {
        (Some(x), Some(y)) => Some(x.max(y)),
        (x, None) => x,
        (None, y) => y,
    };
    merged.filter(|&t| done <= t)
}

/// Объединяет тайтл из базы с тайтлом из файла. Твои данные главные: название, группа,
/// библиотека и заметка остаются как есть. Прогресс берётся больший, пустые поля
/// дозаполняются из файла.
pub fn merge_titles(existing: &TitleDraft, incoming: &TitleDraft) -> TitleDraft {
    let mut merged = existing.clone();
    merged.done_main = existing.done_main.max(incoming.done_main);
    merged.done_volumes = existing.done_volumes.max(incoming.done_volumes);
    merged.total_main = merge_total(existing.total_main, incoming.total_main, merged.done_main);
    merged.total_volumes =
        merge_total(existing.total_volumes, incoming.total_volumes, merged.done_volumes);
    if merged.kind.is_none() {
        merged.kind = incoming.kind;
    }
    if merged.original_name.is_none() {
        merged.original_name = incoming.original_name.clone();
    }
    if merged.cover.is_none() {
        merged.cover = incoming.cover.clone();
    }
    if merged.url.is_none() {
        merged.url = incoming.url.clone();
    }
    if merged.note.is_empty() {
        merged.note = incoming.note.clone();
    }
    merged
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{Library, MangaKind, Medium};

    fn draft(name: &str) -> TitleDraft {
        TitleDraft::new(Medium::Manga, name)
    }

    #[test]
    fn merge_takes_bigger_progress_and_fills_empty_fields() {
        let mut mine = draft("Берсерк");
        mine.done_main = 100;
        mine.done_volumes = 40;
        mine.note = "моя заметка".into();
        mine.library = Library::MangaLib;

        let mut theirs = draft("Берсерк");
        theirs.done_main = 250;
        theirs.done_volumes = 10;
        theirs.total_main = Some(400);
        theirs.original_name = Some("Berserk".into());
        theirs.cover = Some("https://example.com/c.jpg".into());
        theirs.url = Some("https://hentailib.me/x".into());
        theirs.kind = Some(MangaKind::Manga);
        theirs.note = "чужая заметка".into();
        theirs.library = Library::HentaiLib;

        let merged = merge_titles(&mine, &theirs);
        assert_eq!(merged.done_main, 250);
        assert_eq!(merged.done_volumes, 40);
        assert_eq!(merged.total_main, Some(400));
        assert_eq!(merged.original_name.as_deref(), Some("Berserk"));
        assert_eq!(merged.cover.as_deref(), Some("https://example.com/c.jpg"));
        assert_eq!(merged.url.as_deref(), Some("https://hentailib.me/x"));
        assert_eq!(merged.kind, Some(MangaKind::Manga));
        // моё остаётся моим
        assert_eq!(merged.note, "моя заметка");
        assert_eq!(merged.library, Library::MangaLib);
    }

    #[test]
    fn merge_does_not_overwrite_existing_values() {
        let mut mine = draft("A");
        mine.cover = Some("моя".into());
        mine.url = Some("https://a".into());
        let mut theirs = draft("A");
        theirs.cover = Some("чужая".into());
        theirs.url = Some("https://b".into());
        let merged = merge_titles(&mine, &theirs);
        assert_eq!(merged.cover.as_deref(), Some("моя"));
        assert_eq!(merged.url.as_deref(), Some("https://a"));
    }

    #[test]
    fn merged_total_below_progress_becomes_unknown() {
        let mut mine = draft("A");
        mine.done_main = 80; // у онгоинга «всего» неизвестно
        let mut theirs = draft("A");
        theirs.done_main = 10;
        theirs.total_main = Some(50);
        let merged = merge_titles(&mine, &theirs);
        assert_eq!(merged.done_main, 80);
        assert_eq!(merged.total_main, None);
    }

    #[test]
    fn merged_titles_always_pass_validation() {
        let mut mine = draft("A");
        mine.total_main = Some(30);
        mine.done_main = 30;
        let mut theirs = draft("A");
        theirs.total_main = Some(20);
        theirs.done_main = 5;
        let mut merged = merge_titles(&mine, &theirs);
        assert!(merged.prepare().is_ok());
        assert_eq!(merged.total_main, Some(30));
    }

    #[test]
    fn name_key_ignores_case_spacing_and_yo() {
        assert_eq!(name_key(&draft("  Ёжик   В Тумане ")), name_key(&draft("ежик в тумане")));
        assert_ne!(name_key(&draft("A")), name_key(&TitleDraft::new(Medium::Anime, "A")));
    }

    #[test]
    fn url_key_ignores_scheme_www_query_and_trailing_slash() {
        let with = |u: &str| {
            let mut d = draft("x");
            d.url = Some(u.to_string());
            url_key(&d)
        };
        let expected = Some("manga:hentailib.me/ru/manga/1".to_string());
        assert_eq!(with("https://hentailib.me/ru/manga/1"), expected);
        assert_eq!(with("HTTP://www.HentaiLib.me/ru/manga/1/?section=info#top"), expected);
        assert_eq!(with("hentailib.me/ru/manga/1"), expected);
        assert_eq!(with("   "), None);
        assert_eq!(url_key(&draft("no url")), None);
    }

    #[test]
    fn backup_survives_json_roundtrip() {
        let backup = Backup {
            format: BACKUP_FORMAT.to_string(),
            version: BACKUP_VERSION,
            exported_at: 123,
            groups: vec![Group {
                id: 1,
                medium: Medium::Manga,
                name: "Читаю".into(),
                color: "#6ea8ff".into(),
                position: 0,
            }],
            titles: vec![Title {
                id: 7,
                fields: draft("Берсерк"),
                created_at: 1,
                updated_at: 2,
                progress_updated_at: 3,
                library_changed_at: Some(4),
            }],
        };
        let json = serde_json::to_string(&backup).unwrap();
        let back: Backup = serde_json::from_str(&json).unwrap();
        assert_eq!(back, backup);
    }
}
