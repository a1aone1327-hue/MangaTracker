use std::fmt;

use crate::domain::DomainError;

/// Любая ошибка ядра. Слой Tauri превращает её в текст для интерфейса.
#[derive(Debug)]
pub enum Error {
    /// Нарушено правило предметной области (понятное пользователю сообщение).
    Domain(DomainError),
    /// Ошибка SQLite.
    Db(rusqlite::Error),
    /// Запрошенной записи нет.
    NotFound(&'static str),
    /// База создана более новой версией приложения: не трогаем, чтобы не испортить данные.
    SchemaTooNew { found: i64, supported: i64 },
    /// Файл копии не читается или не подходит.
    InvalidBackup(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Domain(e) => write!(f, "{e}"),
            Error::Db(e) => write!(f, "ошибка базы данных: {e}"),
            Error::NotFound(what) => write!(f, "не найдено: {what}"),
            Error::InvalidBackup(why) => write!(f, "Не удалось прочитать копию: {why}"),
            Error::SchemaTooNew { found, supported } => write!(
                f,
                "база создана более новой версией приложения (схема {found}, поддерживается {supported})"
            ),
        }
    }
}

impl std::error::Error for Error {}

impl From<DomainError> for Error {
    fn from(e: DomainError) -> Self {
        Error::Domain(e)
    }
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        Error::Db(e)
    }
}
