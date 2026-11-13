//! Библиотека (сайт), на которой лежит тайтл.

use serde::{Deserialize, Serialize};

use super::model::Medium;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Library {
    MangaLib,
    HentaiLib,
    AnimeLib,
    Other,
}

impl Library {
    pub fn as_str(self) -> &'static str {
        match self {
            Library::MangaLib => "mangalib",
            Library::HentaiLib => "hentailib",
            Library::AnimeLib => "animelib",
            Library::Other => "other",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "mangalib" => Some(Library::MangaLib),
            "hentailib" => Some(Library::HentaiLib),
            "animelib" => Some(Library::AnimeLib),
            "other" => Some(Library::Other),
            _ => None,
        }
    }

    /// Библиотека по умолчанию для нового тайтла раздела.
    pub fn default_for(medium: Medium) -> Self {
        match medium {
            Medium::Manga => Library::MangaLib,
            Medium::Anime => Library::AnimeLib,
        }
    }

    /// Определяет библиотеку по ссылке. Смотрим только на хост и ищем имя сайта
    /// в нём, а не точный домен: зеркала и домены у таких сайтов меняются.
    pub fn detect(url: &str) -> Option<Self> {
        let host = host_of(url);
        if host.contains("hentailib") {
            Some(Library::HentaiLib)
        } else if host.contains("mangalib") {
            Some(Library::MangaLib)
        } else if host.contains("animelib") || host.contains("anilib") {
            Some(Library::AnimeLib)
        } else {
            None
        }
    }
}

fn host_of(url: &str) -> String {
    let lower = url.trim().to_lowercase();
    let rest = match lower.find("://") {
        Some(i) => &lower[i + 3..],
        None => lower.as_str(),
    };
    let host = rest
        .split(|c: char| c == '/' || c == '?' || c == '#')
        .next()
        .unwrap_or("")
        .to_string();
    host
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_library_from_host() {
        assert_eq!(Library::detect("https://mangalib.me/ru/manga/1--berserk"), Some(Library::MangaLib));
        assert_eq!(Library::detect("https://hentailib.me/ru/manga/2--x"), Some(Library::HentaiLib));
        assert_eq!(Library::detect("https://anilib.me/ru/anime/3--y"), Some(Library::AnimeLib));
        assert_eq!(Library::detect("https://animelib.org/anime/3"), Some(Library::AnimeLib));
    }

    #[test]
    fn detection_survives_mirrors_and_missing_scheme() {
        assert_eq!(Library::detect("  HTTPS://WWW.MangaLib.example/x  "), Some(Library::MangaLib));
        assert_eq!(Library::detect("hentailib.me/ru/manga/2"), Some(Library::HentaiLib));
    }

    #[test]
    fn only_the_host_counts() {
        assert_eq!(Library::detect("https://example.com/?ref=mangalib.me"), None);
        assert_eq!(Library::detect("https://example.com/hentailib"), None);
        assert_eq!(Library::detect(""), None);
    }

    #[test]
    fn string_form_roundtrips() {
        for lib in [Library::MangaLib, Library::HentaiLib, Library::AnimeLib, Library::Other] {
            assert_eq!(Library::parse(lib.as_str()), Some(lib));
        }
        assert_eq!(Library::parse("nope"), None);
    }

    #[test]
    fn defaults_follow_medium() {
        assert_eq!(Library::default_for(Medium::Manga), Library::MangaLib);
        assert_eq!(Library::default_for(Medium::Anime), Library::AnimeLib);
    }
}
