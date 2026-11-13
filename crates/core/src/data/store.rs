//! Подключение к базе. Операции над группами и тайтлами лежат в `groups.rs` и `titles.rs`.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;

use super::migrations;
use crate::Result;

pub struct Store {
    pub(super) conn: Connection,
}

impl Store {
    /// Открывает (или создаёт) файл базы и доводит схему до актуальной.
    pub fn open(path: &Path) -> Result<Self> {
        Self::init(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    fn init(mut conn: Connection) -> Result<Self> {
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        migrations::migrate(&mut conn)?;
        Ok(Self { conn })
    }
}

/// Текущее время в unix-секундах.
pub(super) fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
