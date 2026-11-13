//! Основные сущности: разделы, группы, тайтлы.

use serde::{Deserialize, Serialize};

use super::error::DomainError;
use super::library::Library;
use super::progress;

pub const MAX_TITLE_NAME_CHARS: usize = 200;
/// Локальная обложка хранится в самой базе (уменьшенная картинка, data URL).
pub const MAX_COVER_BYTES: usize = 400_000;

/// Раздел приложения. У манги счётчики «главы и тома», у аниме «серии».
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Medium {
    Manga,
    Anime,
}

impl Medium {
    pub fn as_str(self) -> &'static str {
        match self {
            Medium::Manga => "manga",
            Medium::Anime => "anime",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "manga" => Some(Medium::Manga),
            "anime" => Some(Medium::Anime),
            _ => None,
        }
    }
}

/// Тип произведения (только для манги).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MangaKind {
    Manga,
    Manhwa,
    Manhua,
}

impl MangaKind {
    pub fn as_str(self) -> &'static str {
        match self {
            MangaKind::Manga => "manga",
            MangaKind::Manhwa => "manhwa",
            MangaKind::Manhua => "manhua",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "manga" => Some(MangaKind::Manga),
            "manhwa" => Some(MangaKind::Manhwa),
            "manhua" => Some(MangaKind::Manhua),
            _ => None,
        }
    }
}

/// Какой счётчик меняем. `Main` — главы (манга) или серии (аниме).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Counter {
    Main,
    Volumes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Group {
    pub id: i64,
    pub medium: Medium,
    pub name: String,
    /// `#rrggbb`
    pub color: String,
    pub position: i64,
}

/// Редактируемые поля тайтла (для создания и полного обновления).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TitleDraft {
    pub medium: Medium,
    pub kind: Option<MangaKind>,
    pub library: Library,
    pub group_id: Option<i64>,
    pub name: String,
    pub original_name: Option<String>,
    /// Ссылка на обложку или путь к локальному файлу (решим на шаге 4).
    pub cover: Option<String>,
    /// Ссылка на страницу тайтла на сайте библиотеки.
    pub url: Option<String>,
    #[serde(default)]
    pub note: String,
    pub total_main: Option<u32>,
    pub total_volumes: Option<u32>,
    #[serde(default)]
    pub done_main: u32,
    #[serde(default)]
    pub done_volumes: u32,
}

impl TitleDraft {
    pub fn new(medium: Medium, name: impl Into<String>) -> Self {
        Self {
            medium,
            kind: None,
            library: Library::default_for(medium),
            group_id: None,
            name: name.into(),
            original_name: None,
            cover: None,
            url: None,
            note: String::new(),
            total_main: None,
            total_volumes: None,
            done_main: 0,
            done_volumes: 0,
        }
    }

    /// Приводит поля к каноничному виду и проверяет правила. Вызывается перед сохранением.
    pub fn prepare(&mut self) -> Result<(), DomainError> {
        self.name = self.name.trim().to_string();
        if self.name.is_empty() {
            return Err(DomainError::EmptyName);
        }
        if self.name.chars().count() > MAX_TITLE_NAME_CHARS {
            return Err(DomainError::NameTooLong);
        }
        self.original_name = clean(self.original_name.take());
        self.cover = clean(self.cover.take());
        self.url = clean(self.url.take());
        self.note = self.note.trim().to_string();
        if self.cover.as_ref().is_some_and(|c| c.len() > MAX_COVER_BYTES) {
            return Err(DomainError::CoverTooLarge);
        }

        // «0 всего» то же, что «неизвестно»
        self.total_main = self.total_main.filter(|&t| t > 0);
        self.total_volumes = self.total_volumes.filter(|&t| t > 0);

        // у аниме нет томов и типа манги
        if self.medium == Medium::Anime {
            self.kind = None;
            self.total_volumes = None;
            self.done_volumes = 0;
        }

        progress::check(self.done_main, self.total_main)?;
        progress::check(self.done_volumes, self.total_volumes)?;
        Ok(())
    }

    /// Прочитано и всего по выбранному счётчику.
    pub fn counter(&self, counter: Counter) -> (u32, Option<u32>) {
        match counter {
            Counter::Main => (self.done_main, self.total_main),
            Counter::Volumes => (self.done_volumes, self.total_volumes),
        }
    }

    /// Быстрая кнопка +1/−1. Возвращает `true`, если значение изменилось.
    pub fn step(&mut self, counter: Counter, delta: i32) -> bool {
        if counter == Counter::Volumes && self.medium == Medium::Anime {
            return false;
        }
        let (done, total) = self.counter(counter);
        let next = progress::step(done, total, delta);
        match counter {
            Counter::Main => self.done_main = next,
            Counter::Volumes => self.done_volumes = next,
        }
        next != done
    }

    /// Процент по основному счётчику (главы или серии).
    pub fn main_percent(&self) -> Option<u8> {
        progress::percent(self.done_main, self.total_main)
    }
}

fn clean(value: Option<String>) -> Option<String> {
    value.map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}

/// Сохранённый тайтл: поля формы плюс служебные метки времени (unix-секунды).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Title {
    pub id: i64,
    #[serde(flatten)]
    pub fields: TitleDraft,
    pub created_at: i64,
    pub updated_at: i64,
    /// Когда последний раз менялся прогресс (для сортировки «недавно обновлённые»).
    pub progress_updated_at: i64,
    /// Когда тайтл последний раз переносили в другую библиотеку.
    pub library_changed_at: Option<i64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepare_trims_and_normalizes() {
        let mut d = TitleDraft::new(Medium::Manga, "  Берсерк  ");
        d.original_name = Some("   ".into());
        d.url = Some(" https://mangalib.me/x ".into());
        d.note = "  заметка \n".into();
        d.total_main = Some(0);
        d.prepare().unwrap();
        assert_eq!(d.name, "Берсерк");
        assert_eq!(d.original_name, None);
        assert_eq!(d.url.as_deref(), Some("https://mangalib.me/x"));
        assert_eq!(d.note, "заметка");
        assert_eq!(d.total_main, None);
    }

    #[test]
    fn prepare_rejects_empty_and_too_long_names() {
        let mut d = TitleDraft::new(Medium::Manga, "   ");
        assert_eq!(d.prepare(), Err(DomainError::EmptyName));
        let mut d = TitleDraft::new(Medium::Manga, "я".repeat(MAX_TITLE_NAME_CHARS + 1));
        assert_eq!(d.prepare(), Err(DomainError::NameTooLong));
    }

    #[test]
    fn prepare_rejects_oversized_cover() {
        let mut d = TitleDraft::new(Medium::Manga, "A");
        d.cover = Some("a".repeat(MAX_COVER_BYTES + 1));
        assert_eq!(d.prepare(), Err(DomainError::CoverTooLarge));

        let mut d = TitleDraft::new(Medium::Manga, "A");
        d.cover = Some("a".repeat(MAX_COVER_BYTES));
        assert_eq!(d.prepare(), Ok(()));
    }

    #[test]
    fn prepare_rejects_progress_above_total() {
        let mut d = TitleDraft::new(Medium::Manga, "A");
        d.total_main = Some(10);
        d.done_main = 11;
        assert_eq!(d.prepare(), Err(DomainError::ProgressExceedsTotal));

        let mut d = TitleDraft::new(Medium::Manga, "A");
        d.total_volumes = Some(2);
        d.done_volumes = 3;
        assert_eq!(d.prepare(), Err(DomainError::ProgressExceedsTotal));
    }

    #[test]
    fn anime_has_no_volumes_or_manga_kind() {
        let mut d = TitleDraft::new(Medium::Anime, "Frieren");
        d.kind = Some(MangaKind::Manhwa);
        d.total_volumes = Some(5);
        d.done_volumes = 2;
        d.prepare().unwrap();
        assert_eq!(d.kind, None);
        assert_eq!(d.total_volumes, None);
        assert_eq!(d.done_volumes, 0);
        assert_eq!(d.library, Library::AnimeLib);
    }

    #[test]
    fn step_changes_only_the_chosen_counter_and_reports_change() {
        let mut d = TitleDraft::new(Medium::Manga, "A");
        d.total_main = Some(2);
        assert!(d.step(Counter::Main, 1));
        assert!(d.step(Counter::Main, 1));
        assert!(!d.step(Counter::Main, 1)); // уже на максимуме
        assert_eq!(d.done_main, 2);
        assert_eq!(d.done_volumes, 0);
        assert!(d.step(Counter::Volumes, 1)); // томов «всего» неизвестно
        assert_eq!(d.done_volumes, 1);
        assert!(!d.step(Counter::Main, 0));
    }

    #[test]
    fn anime_ignores_volume_steps() {
        let mut d = TitleDraft::new(Medium::Anime, "A");
        assert!(!d.step(Counter::Volumes, 1));
        assert_eq!(d.done_volumes, 0);
        assert!(d.step(Counter::Main, 1));
    }

    #[test]
    fn main_percent_uses_main_counter() {
        let mut d = TitleDraft::new(Medium::Manga, "A");
        assert_eq!(d.main_percent(), None);
        d.total_main = Some(4);
        d.done_main = 1;
        assert_eq!(d.main_percent(), Some(25));
    }
}
