//! Хранилище: SQLite, миграции и операции над группами и тайтлами.

mod backup;
mod groups;
mod migrations;
mod sql_types;
mod store;
mod titles;

pub use migrations::SCHEMA_VERSION;
pub use store::Store;

#[cfg(test)]
mod tests;
