use std::fmt;

/// Нарушения правил предметной области. Тексты показываются пользователю.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainError {
    EmptyName,
    NameTooLong,
    DuplicateGroupName,
    BadColor,
    ProgressExceedsTotal,
    GroupMediumMismatch,
    BadMoveTarget,
    BadOrder,
    MediumImmutable,
    CoverTooLarge,
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            DomainError::EmptyName => "Название не может быть пустым",
            DomainError::NameTooLong => "Название слишком длинное",
            DomainError::DuplicateGroupName => "Группа с таким названием уже есть",
            DomainError::BadColor => "Цвет должен быть в формате #rrggbb",
            DomainError::ProgressExceedsTotal => "Прогресс не может быть больше общего числа",
            DomainError::GroupMediumMismatch => "Эта группа относится к другому разделу",
            DomainError::BadMoveTarget => "Выбери другую группу того же раздела",
            DomainError::BadOrder => "Список групп для сортировки не совпадает с текущим",
            DomainError::MediumImmutable => "Раздел (манга или аниме) у тайтла менять нельзя",
            DomainError::CoverTooLarge => "Обложка слишком большая",
        };
        f.write_str(text)
    }
}

impl std::error::Error for DomainError {}
