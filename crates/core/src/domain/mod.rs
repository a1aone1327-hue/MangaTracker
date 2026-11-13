//! Чистая логика без ввода-вывода.

pub mod backup;
mod error;
pub mod groups;
pub mod library;
mod model;
pub mod progress;
pub mod remote;

pub use backup::{Backup, ImportMode, ImportReport};
pub use error::DomainError;
pub use groups::OnDelete;
pub use library::Library;
pub use model::{Counter, Group, MangaKind, Medium, Title, TitleDraft};
