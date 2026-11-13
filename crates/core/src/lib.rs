//! Ядро приложения: домен и данные.
//!
//! Слои:
//! - [`domain`]: чистая логика без ввода-вывода (модели, правила прогресса,
//!   правила групп, определение библиотеки по ссылке);
//! - [`data`]: хранение в SQLite, миграции схемы, операции над группами и тайтлами.
//!
//! Tauri-слой (`src-tauri`) только вызывает это ядро и ничего не решает сам.

pub mod data;
pub mod domain;
mod error;

pub use error::{Error, Result};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_set() {
        assert!(!super::VERSION.is_empty());
    }
}
