//! Как перечисления домена хранятся в SQLite: обычным текстом.

use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

use crate::domain::{Library, MangaKind, Medium};

macro_rules! sql_enum {
    ($ty:ty) => {
        impl ToSql for $ty {
            fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
                Ok(ToSqlOutput::from(self.as_str()))
            }
        }

        impl FromSql for $ty {
            fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
                let text = value.as_str()?;
                <$ty>::parse(text).ok_or_else(|| {
                    FromSqlError::Other(format!("неизвестное значение в базе: {text}").into())
                })
            }
        }
    };
}

sql_enum!(Medium);
sql_enum!(Library);
sql_enum!(MangaKind);
