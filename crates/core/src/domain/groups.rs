//! Правила групп: имена, цвета, порядок, перенос тайтлов при удалении.

use serde::{Deserialize, Serialize};

use super::error::DomainError;
use super::model::Group;

pub const MAX_GROUP_NAME_CHARS: usize = 40;

/// Что сделать с тайтлами удаляемой группы.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "action", content = "group_id", rename_all = "snake_case")]
pub enum OnDelete {
    /// Перенести в другую группу (того же раздела).
    MoveTo(i64),
    /// Оставить тайтлы без группы.
    Ungroup,
}

/// Проверяет имя и возвращает его обрезанным. `others` — имена остальных групп раздела.
pub fn validate_group_name(name: &str, others: &[&str]) -> Result<String, DomainError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(DomainError::EmptyName);
    }
    if name.chars().count() > MAX_GROUP_NAME_CHARS {
        return Err(DomainError::NameTooLong);
    }
    let lower = name.to_lowercase();
    if others.iter().any(|o| o.trim().to_lowercase() == lower) {
        return Err(DomainError::DuplicateGroupName);
    }
    Ok(name.to_string())
}

/// Цвет хранится как `#rrggbb` в нижнем регистре.
pub fn validate_color(color: &str) -> Result<String, DomainError> {
    let c = color.trim().to_lowercase();
    let ok = c.len() == 7 && c.starts_with('#') && c[1..].chars().all(|ch| ch.is_ascii_hexdigit());
    if ok {
        Ok(c)
    } else {
        Err(DomainError::BadColor)
    }
}

/// Куда можно переносить тайтлы при удалении: другая группа того же раздела.
pub fn check_move_target(deleting: &Group, target: &Group) -> Result<(), DomainError> {
    if target.id == deleting.id || target.medium != deleting.medium {
        Err(DomainError::BadMoveTarget)
    } else {
        Ok(())
    }
}

/// Новый порядок должен содержать ровно те же группы, без пропусков и повторов.
pub fn check_order(current: &[i64], ordered: &[i64]) -> Result<(), DomainError> {
    let mut a = current.to_vec();
    a.sort_unstable();
    let mut b = ordered.to_vec();
    b.sort_unstable();
    if a == b {
        Ok(())
    } else {
        Err(DomainError::BadOrder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Medium;

    fn group(id: i64, medium: Medium) -> Group {
        Group { id, medium, name: format!("g{id}"), color: "#ffffff".into(), position: 0 }
    }

    #[test]
    fn name_is_trimmed_and_must_not_be_empty() {
        assert_eq!(validate_group_name("  Читаю ", &[]), Ok("Читаю".to_string()));
        assert_eq!(validate_group_name("   ", &[]), Err(DomainError::EmptyName));
    }

    #[test]
    fn name_length_is_limited_in_characters_not_bytes() {
        let ok = "я".repeat(MAX_GROUP_NAME_CHARS);
        assert!(validate_group_name(&ok, &[]).is_ok());
        let long = "я".repeat(MAX_GROUP_NAME_CHARS + 1);
        assert_eq!(validate_group_name(&long, &[]), Err(DomainError::NameTooLong));
    }

    #[test]
    fn duplicate_names_are_rejected_ignoring_case_even_for_cyrillic() {
        assert_eq!(validate_group_name("читаю", &["Читаю", "Брошено"]), Err(DomainError::DuplicateGroupName));
        assert!(validate_group_name("Перечитываю", &["Читаю"]).is_ok());
    }

    #[test]
    fn colors_must_be_hex() {
        assert_eq!(validate_color(" #6EA8FF "), Ok("#6ea8ff".to_string()));
        assert_eq!(validate_color("6ea8ff"), Err(DomainError::BadColor));
        assert_eq!(validate_color("#6ea8f"), Err(DomainError::BadColor));
        assert_eq!(validate_color("#6ea8fg"), Err(DomainError::BadColor));
        assert_eq!(validate_color("#ыыыыы"), Err(DomainError::BadColor));
    }

    #[test]
    fn move_target_must_be_another_group_of_same_medium() {
        let a = group(1, Medium::Manga);
        assert_eq!(check_move_target(&a, &group(2, Medium::Manga)), Ok(()));
        assert_eq!(check_move_target(&a, &group(1, Medium::Manga)), Err(DomainError::BadMoveTarget));
        assert_eq!(check_move_target(&a, &group(3, Medium::Anime)), Err(DomainError::BadMoveTarget));
    }

    #[test]
    fn order_must_be_a_permutation_of_current_groups() {
        assert_eq!(check_order(&[1, 2, 3], &[3, 1, 2]), Ok(()));
        assert_eq!(check_order(&[1, 2, 3], &[1, 2]), Err(DomainError::BadOrder));
        assert_eq!(check_order(&[1, 2, 3], &[1, 1, 2]), Err(DomainError::BadOrder));
        assert_eq!(check_order(&[1, 2], &[1, 2, 4]), Err(DomainError::BadOrder));
    }
}
