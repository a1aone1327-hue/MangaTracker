//! Подсчёт прогресса. Один набор правил для глав, серий и томов.
//! `total == None` означает «всего неизвестно» (онгоинг).

use super::error::DomainError;

/// Не даёт прочитанному превысить известное «всего».
pub fn clamp_done(done: u32, total: Option<u32>) -> u32 {
    match total {
        Some(t) => done.min(t),
        None => done,
    }
}

/// Быстрые кнопки +1/−1: никогда не уходит ниже нуля и выше «всего».
pub fn step(done: u32, total: Option<u32>, delta: i32) -> u32 {
    let moved = if delta >= 0 {
        done.saturating_add(delta as u32)
    } else {
        done.saturating_sub(delta.unsigned_abs())
    };
    clamp_done(moved, total)
}

/// Процент выполнения (с округлением вниз). `None`, если «всего» неизвестно.
pub fn percent(done: u32, total: Option<u32>) -> Option<u8> {
    let total = total.filter(|&t| t > 0)?;
    Some((u64::from(done.min(total)) * 100 / u64::from(total)) as u8)
}

/// Дочитано/досмотрено до конца: «всего» известно и достигнуто.
pub fn is_complete(done: u32, total: Option<u32>) -> bool {
    matches!(total, Some(t) if t > 0 && done >= t)
}

/// Проверка значений из формы: прочитано не больше «всего».
pub fn check(done: u32, total: Option<u32>) -> Result<(), DomainError> {
    match total {
        Some(t) if done > t => Err(DomainError::ProgressExceedsTotal),
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_up_and_down() {
        assert_eq!(step(5, Some(10), 1), 6);
        assert_eq!(step(5, Some(10), -1), 4);
        assert_eq!(step(5, None, 3), 8);
    }

    #[test]
    fn step_is_clamped_to_zero_and_total() {
        assert_eq!(step(0, None, -1), 0);
        assert_eq!(step(2, Some(10), -50), 0);
        assert_eq!(step(10, Some(10), 1), 10);
        assert_eq!(step(9, Some(10), 5), 10);
    }

    #[test]
    fn step_does_not_overflow() {
        assert_eq!(step(u32::MAX, None, 1), u32::MAX);
        assert_eq!(step(0, None, i32::MIN), 0);
    }

    #[test]
    fn percent_rounds_down_and_handles_unknown_total() {
        assert_eq!(percent(5, Some(10)), Some(50));
        assert_eq!(percent(1, Some(3)), Some(33));
        assert_eq!(percent(12, Some(10)), Some(100));
        assert_eq!(percent(7, None), None);
        assert_eq!(percent(3, Some(0)), None);
    }

    #[test]
    fn completion_needs_known_total() {
        assert!(is_complete(10, Some(10)));
        assert!(!is_complete(9, Some(10)));
        assert!(!is_complete(100, None));
        assert!(!is_complete(0, Some(0)));
    }

    #[test]
    fn check_rejects_progress_above_total() {
        assert_eq!(check(11, Some(10)), Err(DomainError::ProgressExceedsTotal));
        assert_eq!(check(10, Some(10)), Ok(()));
        assert_eq!(check(500, None), Ok(()));
    }
}
