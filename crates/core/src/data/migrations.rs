//! Миграции по `PRAGMA user_version`: номер схемы хранится в самом файле базы.

use rusqlite::Connection;

use crate::{Error, Result};

/// Миграции по порядку. Выпущенные не редактируем, только добавляем новые.
const MIGRATIONS: &[&str] = &[include_str!("migrations/0001_init.sql")];

/// Версия схемы, которую понимает эта сборка приложения.
pub const SCHEMA_VERSION: i64 = MIGRATIONS.len() as i64;

pub fn migrate(conn: &mut Connection) -> Result<()> {
    let current: i64 = conn.query_row("PRAGMA user_version", [], |row| row.get(0))?;
    if current > SCHEMA_VERSION {
        // База от более новой версии: ничего не трогаем, чтобы не испортить данные
        return Err(Error::SchemaTooNew { found: current, supported: SCHEMA_VERSION });
    }
    for (index, sql) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.execute_batch(&format!("PRAGMA user_version = {}", index + 1))?;
        tx.commit()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group_count(conn: &Connection) -> i64 {
        conn.query_row("SELECT COUNT(*) FROM groups", [], |r| r.get(0)).unwrap()
    }

    fn version(conn: &Connection) -> i64 {
        conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap()
    }

    #[test]
    fn fresh_database_gets_latest_schema() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        assert_eq!(version(&conn), SCHEMA_VERSION);
        assert_eq!(group_count(&conn), 10);
    }

    #[test]
    fn migrating_twice_changes_nothing() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&mut conn).unwrap();
        migrate(&mut conn).unwrap();
        assert_eq!(version(&conn), SCHEMA_VERSION);
        assert_eq!(group_count(&conn), 10);
    }

    #[test]
    fn database_from_newer_app_is_left_untouched() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA user_version = 99").unwrap();
        let result = migrate(&mut conn);
        assert!(matches!(result, Err(Error::SchemaTooNew { found: 99, .. })));
        assert_eq!(version(&conn), 99);
    }
}
