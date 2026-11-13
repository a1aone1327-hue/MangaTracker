//! Подтягивание данных о тайтле с сайтов сети lib.social (MangaLib, HentaiLib, AnimeLib):
//! разбор ссылки и ответа их JSON-интерфейса.
//!
//! Сети здесь нет: запрос делает слой Tauri, ядро только понимает ссылку и ответ.
//! Интерфейс неофициальный, поэтому разбор ответа намеренно терпимый: берём то, что
//! нашли, а чего нет, оставляем пустым.

use serde::Serialize;
use serde_json::Value;

use super::library::Library;
use super::model::{MangaKind, Medium};

const API_BASE: &str = "https://api.cdnlibs.org/api";

/// Параметры сайтов. Если сайт поменяет идентификатор или адрес, правится только здесь.
/// MangaLib (Site-Id 1) подтверждён описанием открытых клиентов, остальные значения
/// взяты по аналогии и не проверялись вживую.
struct SiteConfig {
    medium: Medium,
    api_section: &'static str,
    site_id: u32,
    /// Дополнительные поля, которые просим у интерфейса (количество глав).
    query: &'static str,
}

fn site_config(library: Library) -> Option<SiteConfig> {
    match library {
        Library::MangaLib => Some(SiteConfig {
            medium: Medium::Manga,
            api_section: "manga",
            site_id: 1,
            query: "?fields%5B%5D=chap_count",
        }),
        Library::HentaiLib => Some(SiteConfig {
            medium: Medium::Manga,
            api_section: "manga",
            site_id: 4,
            query: "?fields%5B%5D=chap_count",
        }),
        Library::AnimeLib => Some(SiteConfig {
            medium: Medium::Anime,
            api_section: "anime",
            site_id: 5,
            query: "",
        }),
        Library::Other => None,
    }
}

/// Что нужно знать, чтобы запросить данные по ссылке.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibLink {
    pub library: Library,
    pub medium: Medium,
    pub api_url: String,
    pub site_id: u32,
    /// Адрес сайта из ссылки, нужен как Referer для картинок.
    pub origin: String,
}

/// Данные тайтла, найденные на сайте.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TitleInfo {
    pub medium: Medium,
    pub library: Library,
    pub name: String,
    pub original_name: Option<String>,
    pub total_main: Option<u32>,
    pub kind: Option<MangaKind>,
    pub cover_url: Option<String>,
}

fn split_url(url: &str) -> Option<(String, String)> {
    let trimmed = url.trim();
    let rest = match trimmed.find("://") {
        Some(i) => {
            let scheme = trimmed[..i].to_lowercase();
            if scheme != "http" && scheme != "https" {
                return None;
            }
            &trimmed[i + 3..]
        }
        None => trimmed,
    };
    let end = rest.find(|c: char| c == '?' || c == '#').unwrap_or(rest.len());
    let rest = &rest[..end];
    let (host, path) = match rest.find('/') {
        Some(i) => (&rest[..i], &rest[i..]),
        None => (rest, ""),
    };
    if host.is_empty() {
        None
    } else {
        Some((host.to_lowercase(), path.to_string()))
    }
}

/// Идентификатор тайтла в адресе страницы вида `7965--chainsaw-man`.
fn slug_of(path: &str) -> Option<String> {
    path.split('/').rev().find_map(|segment| {
        let (id, rest) = segment.split_once("--")?;
        let valid = !id.is_empty()
            && id.chars().all(|c| c.is_ascii_digit())
            && rest.chars().any(|c| c.is_ascii_alphanumeric())
            && segment
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.');
        valid.then(|| segment.to_string())
    })
}

/// Разбирает ссылку на страницу тайтла. `None`, если сайт не поддерживается
/// или в ссылке нет идентификатора тайтла.
pub fn parse_link(url: &str) -> Option<LibLink> {
    let library = Library::detect(url)?;
    let config = site_config(library)?;
    let (host, path) = split_url(url)?;
    let slug = slug_of(&path)?;
    Some(LibLink {
        library,
        medium: config.medium,
        api_url: format!("{API_BASE}/{}/{slug}{}", config.api_section, config.query),
        site_id: config.site_id,
        origin: format!("https://{host}"),
    })
}

fn text<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
    data.get(key).and_then(Value::as_str).map(str::trim).filter(|s| !s.is_empty())
}

fn as_count(value: &Value) -> Option<u32> {
    let n = match value {
        Value::Number(n) => n.as_u64()?,
        Value::String(s) => s.trim().parse::<u64>().ok()?,
        _ => return None,
    };
    u32::try_from(n).ok().filter(|&n| n > 0)
}

fn kind_from_label(label: &str) -> Option<MangaKind> {
    let l = label.to_lowercase();
    if l.contains("манхв") || l.contains("manhwa") {
        Some(MangaKind::Manhwa)
    } else if l.contains("маньхуа") || l.contains("manhua") {
        Some(MangaKind::Manhua)
    } else if l.contains("манга") || l.contains("manga") {
        Some(MangaKind::Manga)
    } else {
        None
    }
}

/// Адрес обложки: только https, `//host/...` дополняем схемой.
fn cover_of(data: &Value) -> Option<String> {
    let candidate = ["/cover/default", "/cover/md", "/cover/thumbnail"]
        .iter()
        .find_map(|p| data.pointer(p).and_then(Value::as_str))
        .or_else(|| data.get("cover").and_then(Value::as_str))?
        .trim();
    let url = match candidate.strip_prefix("//") {
        Some(rest) => format!("https://{rest}"),
        None => candidate.to_string(),
    };
    url.starts_with("https://").then_some(url)
}

/// Разбирает ответ интерфейса. `None`, если в нём нет даже названия.
pub fn parse_title_info(json: &str, link: &LibLink) -> Option<TitleInfo> {
    let root: Value = serde_json::from_str(json).ok()?;
    let data = root.get("data").unwrap_or(&root);

    let name = text(data, "rus_name")
        .or_else(|| text(data, "eng_name"))
        .or_else(|| text(data, "name"))?
        .to_string();
    let original_name = [text(data, "name"), text(data, "eng_name")]
        .into_iter()
        .flatten()
        .find(|o| *o != name)
        .map(str::to_string);

    let total_main = data
        .get("chap_count")
        .and_then(as_count)
        .or_else(|| data.get("episodes_count").and_then(as_count))
        .or_else(|| data.pointer("/items_count/total").and_then(as_count))
        .or_else(|| data.pointer("/items_count/uploaded").and_then(as_count));

    let kind = if link.medium == Medium::Manga {
        data.pointer("/type/label").and_then(Value::as_str).and_then(kind_from_label)
    } else {
        None
    };

    Some(TitleInfo {
        medium: link.medium,
        library: link.library,
        name,
        original_name,
        total_main,
        kind,
        cover_url: cover_of(data),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_links_of_the_three_sites() {
        let m = parse_link("https://mangalib.me/ru/manga/7965--chainsaw-man?section=info").unwrap();
        assert_eq!(m.library, Library::MangaLib);
        assert_eq!(m.medium, Medium::Manga);
        assert_eq!(m.site_id, 1);
        assert_eq!(m.origin, "https://mangalib.me");
        assert_eq!(
            m.api_url,
            "https://api.cdnlibs.org/api/manga/7965--chainsaw-man?fields%5B%5D=chap_count"
        );

        let h = parse_link("hentailib.me/ru/manga/123--some-title").unwrap();
        assert_eq!(h.library, Library::HentaiLib);
        assert!(h.api_url.contains("/manga/123--some-title"));

        let a = parse_link("https://anilib.me/ru/anime/555--frieren/watch").unwrap();
        assert_eq!(a.library, Library::AnimeLib);
        assert_eq!(a.medium, Medium::Anime);
        assert_eq!(a.api_url, "https://api.cdnlibs.org/api/anime/555--frieren");
    }

    #[test]
    fn rejects_unknown_sites_and_links_without_title_id() {
        assert_eq!(parse_link("https://example.com/manga/1--x"), None);
        assert_eq!(parse_link("https://mangalib.me/ru/catalog"), None);
        assert_eq!(parse_link("https://mangalib.me/ru/manga/chainsaw-man"), None);
        assert_eq!(parse_link("ftp://mangalib.me/ru/manga/1--x"), None);
        assert_eq!(parse_link("https://example.com/?ref=mangalib.me/1--x"), None);
        assert_eq!(parse_link(""), None);
    }

    #[test]
    fn slug_must_be_plain_ascii() {
        assert_eq!(parse_link("https://mangalib.me/ru/manga/12--../../etc"), None);
        assert_eq!(parse_link("https://mangalib.me/ru/manga/12--глава"), None);
    }

    fn manga_link() -> LibLink {
        parse_link("https://mangalib.me/ru/manga/7965--chainsaw-man").unwrap()
    }

    #[test]
    fn parses_a_full_response() {
        let json = r#"{"data":{
            "id": 7965, "name": "Chainsaw Man", "rus_name": "Человек-бензопила",
            "eng_name": "Chainsaw Man", "chap_count": 190,
            "type": {"id": 1, "label": "Манга"},
            "cover": {"default": "https://cover.example.org/c/default.jpg", "thumbnail": "x"}
        }}"#;
        let info = parse_title_info(json, &manga_link()).unwrap();
        assert_eq!(info.name, "Человек-бензопила");
        assert_eq!(info.original_name.as_deref(), Some("Chainsaw Man"));
        assert_eq!(info.total_main, Some(190));
        assert_eq!(info.kind, Some(MangaKind::Manga));
        assert_eq!(info.cover_url.as_deref(), Some("https://cover.example.org/c/default.jpg"));
        assert_eq!(info.library, Library::MangaLib);
    }

    #[test]
    fn tolerates_missing_and_oddly_typed_fields() {
        // без обёртки data, количество строкой, обложка без схемы, тип манхвы
        let json = r#"{"name":"Solo Leveling","chap_count":"200",
            "type":{"label":"Манхва"},"cover":{"md":"//cover.example.org/m.jpg"}}"#;
        let info = parse_title_info(json, &manga_link()).unwrap();
        assert_eq!(info.name, "Solo Leveling");
        assert_eq!(info.original_name, None);
        assert_eq!(info.total_main, Some(200));
        assert_eq!(info.kind, Some(MangaKind::Manhwa));
        assert_eq!(info.cover_url.as_deref(), Some("https://cover.example.org/m.jpg"));
    }

    #[test]
    fn falls_back_to_items_count_and_skips_unsafe_cover() {
        let json = r#"{"data":{"eng_name":"X","items_count":{"uploaded":12,"total":15},
            "cover":{"default":"http://insecure.example.org/c.jpg"}}}"#;
        let info = parse_title_info(json, &manga_link()).unwrap();
        assert_eq!(info.name, "X");
        assert_eq!(info.total_main, Some(15));
        assert_eq!(info.cover_url, None);
    }

    #[test]
    fn zero_chapter_count_means_unknown() {
        let json = r#"{"data":{"name":"X","chap_count":0}}"#;
        assert_eq!(parse_title_info(json, &manga_link()).unwrap().total_main, None);
    }

    #[test]
    fn response_without_a_name_is_rejected() {
        assert_eq!(parse_title_info(r#"{"data":{"chap_count":5}}"#, &manga_link()), None);
        assert_eq!(parse_title_info("<html>blocked</html>", &manga_link()), None);
        assert_eq!(parse_title_info("", &manga_link()), None);
    }

    #[test]
    fn anime_has_no_manga_kind() {
        let link = parse_link("https://anilib.me/ru/anime/555--frieren").unwrap();
        let json = r#"{"data":{"rus_name":"Фрирен","type":{"label":"Манга"},"episodes_count":28}}"#;
        let info = parse_title_info(json, &link).unwrap();
        assert_eq!(info.kind, None);
        assert_eq!(info.total_main, Some(28));
        assert_eq!(info.medium, Medium::Anime);
    }

    #[test]
    fn kind_labels() {
        assert_eq!(kind_from_label("Маньхуа"), Some(MangaKind::Manhua));
        assert_eq!(kind_from_label("OEL-манга"), Some(MangaKind::Manga));
        assert_eq!(kind_from_label("Руманга"), Some(MangaKind::Manga));
        assert_eq!(kind_from_label("Комикс"), None);
    }
}
