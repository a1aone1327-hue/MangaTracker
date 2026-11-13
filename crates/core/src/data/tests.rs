//! Тесты хранилища на базе в памяти: удаление групп, массовые операции, прогресс, библиотеки.

use crate::data::Store;
use crate::domain::backup::BACKUP_VERSION;
use crate::domain::{
    Counter, DomainError, Group, ImportMode, Library, Medium, OnDelete, Title, TitleDraft,
};
use crate::Error;

fn store() -> Store {
    Store::open_in_memory().unwrap()
}

fn group_named(s: &Store, medium: Medium, name: &str) -> Group {
    s.list_groups(medium).unwrap().into_iter().find(|g| g.name == name).unwrap()
}

fn manga(s: &Store, name: &str) -> Group {
    group_named(s, Medium::Manga, name)
}

fn group_names(s: &Store, medium: Medium) -> Vec<String> {
    s.list_groups(medium).unwrap().into_iter().map(|g| g.name).collect()
}

fn add(s: &mut Store, name: &str, group: Option<i64>) -> Title {
    let mut d = TitleDraft::new(Medium::Manga, name);
    d.group_id = group;
    s.create_title(d).unwrap()
}

// ---------- стартовые данные ----------

#[test]
fn fresh_database_has_default_groups_for_both_sections() {
    let s = store();
    assert_eq!(group_names(&s, Medium::Manga), vec!["Читаю", "Брошено", "В планах", "Прочитано", "Любимые"]);
    assert_eq!(group_names(&s, Medium::Anime), vec!["Смотрю", "Брошено", "В планах", "Просмотрено", "Любимые"]);
}

#[test]
fn data_survives_reopen_and_defaults_are_not_reseeded() {
    let path = std::env::temp_dir().join(format!("manga-core-reopen-{}.db", std::process::id()));
    let _ = std::fs::remove_file(&path);
    {
        let mut s = Store::open(&path).unwrap();
        let reading = manga(&s, "Читаю");
        s.delete_group(reading.id, OnDelete::Ungroup).unwrap();
        s.create_title(TitleDraft::new(Medium::Manga, "Берсерк")).unwrap();
    }
    let s = Store::open(&path).unwrap();
    assert_eq!(group_names(&s, Medium::Manga), vec!["Брошено", "В планах", "Прочитано", "Любимые"]);
    assert_eq!(s.list_titles(Medium::Manga).unwrap().len(), 1);
    drop(s);
    let _ = std::fs::remove_file(&path);
}

// ---------- группы ----------

#[test]
fn new_group_goes_to_the_end_of_its_section() {
    let mut s = store();
    let g = s.create_group(Medium::Manga, "  Перечитываю ", "#AABBCC").unwrap();
    assert_eq!(g.name, "Перечитываю");
    assert_eq!(g.color, "#aabbcc");
    assert_eq!(g.position, 5);
    assert_eq!(group_names(&s, Medium::Manga).last().unwrap(), "Перечитываю");
    assert_eq!(group_names(&s, Medium::Anime).len(), 5);
}

#[test]
fn group_names_are_unique_per_section_ignoring_case() {
    let mut s = store();
    let dup = s.create_group(Medium::Manga, "читаю", "#ffffff");
    assert!(matches!(dup, Err(Error::Domain(DomainError::DuplicateGroupName))));
    // в другом разделе то же имя допустимо
    assert!(s.create_group(Medium::Anime, "Читаю", "#ffffff").is_ok());
}

#[test]
fn group_can_be_renamed_and_recolored() {
    let mut s = store();
    let reading = manga(&s, "Читаю");
    let updated = s.update_group(reading.id, "ЧИТАЮ СЕЙЧАС", "#112233").unwrap();
    assert_eq!(updated.name, "ЧИТАЮ СЕЙЧАС");
    assert_eq!(updated.color, "#112233");
    // смена регистра собственного имени допустима
    assert!(s.update_group(reading.id, "читаю сейчас", "#112233").is_ok());
    // чужое имя занято
    let taken = s.update_group(reading.id, "брошено", "#112233");
    assert!(matches!(taken, Err(Error::Domain(DomainError::DuplicateGroupName))));
}

#[test]
fn groups_can_be_reordered() {
    let mut s = store();
    let mut ids: Vec<i64> = s.list_groups(Medium::Manga).unwrap().iter().map(|g| g.id).collect();
    ids.reverse();
    s.reorder_groups(Medium::Manga, &ids).unwrap();
    let after: Vec<i64> = s.list_groups(Medium::Manga).unwrap().iter().map(|g| g.id).collect();
    assert_eq!(after, ids);

    let bad = s.reorder_groups(Medium::Manga, &ids[1..]);
    assert!(matches!(bad, Err(Error::Domain(DomainError::BadOrder))));
}

// ---------- удаление группы ----------

#[test]
fn deleting_group_moves_its_titles_to_the_chosen_group() {
    let mut s = store();
    let reading = manga(&s, "Читаю");
    let dropped = manga(&s, "Брошено");
    let a = add(&mut s, "A", Some(reading.id));
    let b = add(&mut s, "B", Some(reading.id));
    let c = add(&mut s, "C", Some(dropped.id));

    let moved = s.delete_group(reading.id, OnDelete::MoveTo(dropped.id)).unwrap();

    assert_eq!(moved, 2);
    assert_eq!(s.get_title(a.id).unwrap().fields.group_id, Some(dropped.id));
    assert_eq!(s.get_title(b.id).unwrap().fields.group_id, Some(dropped.id));
    assert_eq!(s.get_title(c.id).unwrap().fields.group_id, Some(dropped.id));
    assert!(!group_names(&s, Medium::Manga).contains(&"Читаю".to_string()));
}

#[test]
fn deleting_group_can_leave_titles_without_group() {
    let mut s = store();
    let reading = manga(&s, "Читаю");
    let a = add(&mut s, "A", Some(reading.id));

    let moved = s.delete_group(reading.id, OnDelete::Ungroup).unwrap();

    assert_eq!(moved, 1);
    assert_eq!(s.get_title(a.id).unwrap().fields.group_id, None);
    // тайтл не пропал
    assert_eq!(s.list_titles(Medium::Manga).unwrap().len(), 1);
}

#[test]
fn deleting_group_renumbers_the_rest() {
    let mut s = store();
    let reading = manga(&s, "Читаю");
    s.delete_group(reading.id, OnDelete::Ungroup).unwrap();
    let groups = s.list_groups(Medium::Manga).unwrap();
    let positions: Vec<i64> = groups.iter().map(|g| g.position).collect();
    assert_eq!(positions, vec![0, 1, 2, 3]);
    assert_eq!(groups[0].name, "Брошено");
}

#[test]
fn delete_rejects_target_from_other_section_and_changes_nothing() {
    let mut s = store();
    let reading = manga(&s, "Читаю");
    let watching = group_named(&s, Medium::Anime, "Смотрю");
    let a = add(&mut s, "A", Some(reading.id));

    let result = s.delete_group(reading.id, OnDelete::MoveTo(watching.id));

    assert!(matches!(result, Err(Error::Domain(DomainError::BadMoveTarget))));
    assert!(group_names(&s, Medium::Manga).contains(&"Читаю".to_string()));
    assert_eq!(s.get_title(a.id).unwrap().fields.group_id, Some(reading.id));
}

#[test]
fn delete_rejects_moving_into_the_same_group() {
    let mut s = store();
    let reading = manga(&s, "Читаю");
    let result = s.delete_group(reading.id, OnDelete::MoveTo(reading.id));
    assert!(matches!(result, Err(Error::Domain(DomainError::BadMoveTarget))));
    assert!(group_names(&s, Medium::Manga).contains(&"Читаю".to_string()));
}

#[test]
fn deleting_missing_group_is_not_found() {
    let mut s = store();
    assert!(matches!(s.delete_group(9999, OnDelete::Ungroup), Err(Error::NotFound(_))));
}

// ---------- тайтлы ----------

#[test]
fn title_cannot_join_a_group_of_another_section() {
    let mut s = store();
    let watching = group_named(&s, Medium::Anime, "Смотрю");
    let mut d = TitleDraft::new(Medium::Manga, "A");
    d.group_id = Some(watching.id);
    assert!(matches!(s.create_title(d), Err(Error::Domain(DomainError::GroupMediumMismatch))));
}

#[test]
fn create_rejects_progress_above_total() {
    let mut s = store();
    let mut d = TitleDraft::new(Medium::Manga, "A");
    d.total_main = Some(10);
    d.done_main = 11;
    assert!(matches!(s.create_title(d), Err(Error::Domain(DomainError::ProgressExceedsTotal))));
}

#[test]
fn title_roundtrips_through_the_database() {
    let mut s = store();
    let mut d = TitleDraft::new(Medium::Manga, "Соло лвлинг");
    d.original_name = Some("Solo Leveling".into());
    d.url = Some("https://mangalib.me/ru/manga/1".into());
    d.note = "перечитать с 100 главы".into();
    d.total_main = Some(200);
    d.total_volumes = Some(14);
    d.done_main = 120;
    d.done_volumes = 8;
    let saved = s.create_title(d.clone()).unwrap();
    assert_eq!(saved.fields, d);
    assert_eq!(s.get_title(saved.id).unwrap(), saved);
    assert_eq!(saved.library_changed_at, None);
}

#[test]
fn quick_buttons_respect_zero_and_total() {
    let mut s = store();
    let mut d = TitleDraft::new(Medium::Manga, "A");
    d.total_main = Some(3);
    d.total_volumes = Some(2);
    let t = s.create_title(d).unwrap();

    for _ in 0..5 {
        s.step_progress(t.id, Counter::Main, 1).unwrap();
    }
    assert_eq!(s.get_title(t.id).unwrap().fields.done_main, 3);

    assert_eq!(s.step_progress(t.id, Counter::Main, -1).unwrap().fields.done_main, 2);

    let after_volume = s.step_progress(t.id, Counter::Volumes, 1).unwrap();
    assert_eq!(after_volume.fields.done_volumes, 1);
    assert_eq!(after_volume.fields.done_main, 2);

    assert_eq!(s.step_progress(t.id, Counter::Volumes, -5).unwrap().fields.done_volumes, 0);
}

#[test]
fn anime_has_episodes_only() {
    let mut s = store();
    let mut d = TitleDraft::new(Medium::Anime, "Frieren");
    d.total_volumes = Some(5);
    let t = s.create_title(d).unwrap();
    assert_eq!(t.fields.total_volumes, None);
    assert_eq!(t.fields.library, Library::AnimeLib);

    let t = s.step_progress(t.id, Counter::Volumes, 1).unwrap();
    assert_eq!(t.fields.done_volumes, 0);
    let t = s.step_progress(t.id, Counter::Main, 1).unwrap();
    assert_eq!(t.fields.done_main, 1);
}

#[test]
fn update_changes_fields_but_not_section() {
    let mut s = store();
    let t = add(&mut s, "A", None);

    let mut d = t.fields.clone();
    d.note = "новая заметка".into();
    let updated = s.update_title(t.id, d).unwrap();
    assert_eq!(updated.fields.note, "новая заметка");
    assert_eq!(updated.library_changed_at, None);

    let mut d = updated.fields.clone();
    d.medium = Medium::Anime;
    assert!(matches!(s.update_title(t.id, d), Err(Error::Domain(DomainError::MediumImmutable))));
}

#[test]
fn changing_library_in_the_form_records_the_move() {
    let mut s = store();
    let t = add(&mut s, "A", None);
    assert_eq!(t.fields.library, Library::MangaLib);

    let mut d = t.fields.clone();
    d.library = Library::HentaiLib;
    d.url = Some("https://hentailib.me/ru/manga/1".into());
    let updated = s.update_title(t.id, d).unwrap();

    assert_eq!(updated.fields.library, Library::HentaiLib);
    assert!(updated.library_changed_at.is_some());
}

#[test]
fn delete_titles_removes_only_the_selected() {
    let mut s = store();
    let a = add(&mut s, "A", None);
    let b = add(&mut s, "B", None);
    let c = add(&mut s, "C", None);
    assert_eq!(s.delete_titles(&[a.id, b.id]).unwrap(), 2);
    let left: Vec<i64> = s.list_titles(Medium::Manga).unwrap().iter().map(|t| t.id).collect();
    assert_eq!(left, vec![c.id]);
}

// ---------- массовые операции ----------

#[test]
fn bulk_move_puts_all_selected_titles_into_the_group() {
    let mut s = store();
    let planned = manga(&s, "В планах");
    let reading = manga(&s, "Читаю");
    let a = add(&mut s, "A", Some(reading.id));
    let b = add(&mut s, "B", None);
    let c = add(&mut s, "C", Some(reading.id));

    assert_eq!(s.move_titles(&[a.id, b.id], Some(planned.id)).unwrap(), 2);

    assert_eq!(s.get_title(a.id).unwrap().fields.group_id, Some(planned.id));
    assert_eq!(s.get_title(b.id).unwrap().fields.group_id, Some(planned.id));
    assert_eq!(s.get_title(c.id).unwrap().fields.group_id, Some(reading.id));
}

#[test]
fn bulk_move_to_none_removes_group() {
    let mut s = store();
    let reading = manga(&s, "Читаю");
    let a = add(&mut s, "A", Some(reading.id));
    assert_eq!(s.move_titles(&[a.id], None).unwrap(), 1);
    assert_eq!(s.get_title(a.id).unwrap().fields.group_id, None);
}

#[test]
fn bulk_move_is_all_or_nothing_across_sections() {
    let mut s = store();
    let planned = manga(&s, "В планах");
    let manga_title = add(&mut s, "A", None);
    let anime_title = s.create_title(TitleDraft::new(Medium::Anime, "X")).unwrap();

    let result = s.move_titles(&[manga_title.id, anime_title.id], Some(planned.id));

    assert!(matches!(result, Err(Error::Domain(DomainError::GroupMediumMismatch))));
    assert_eq!(s.get_title(manga_title.id).unwrap().fields.group_id, None);
}

#[test]
fn bulk_library_change_counts_only_real_changes() {
    let mut s = store();
    let a = add(&mut s, "A", None);
    let b = add(&mut s, "B", None);
    let mut already = TitleDraft::new(Medium::Manga, "C");
    already.library = Library::HentaiLib;
    let c = s.create_title(already).unwrap();

    let changed = s.set_library(&[a.id, b.id, c.id], Library::HentaiLib).unwrap();

    assert_eq!(changed, 2);
    let a = s.get_title(a.id).unwrap();
    assert_eq!(a.fields.library, Library::HentaiLib);
    assert!(a.library_changed_at.is_some());
    // у того, что уже было на HentaiLib, отметки о переносе нет
    assert_eq!(s.get_title(c.id).unwrap().library_changed_at, None);
}

// ---------- экспорт и импорт ----------

/// База с пользовательской группой, тайтлами обоих разделов и переносом между библиотеками.
fn seeded_source() -> Store {
    let mut s = store();
    let custom = s.create_group(Medium::Manga, "Перечитываю", "#aabbcc").unwrap();
    let reading = manga(&s, "Читаю");

    let mut d = TitleDraft::new(Medium::Manga, "Берсерк");
    d.group_id = Some(reading.id);
    d.original_name = Some("Berserk".into());
    d.url = Some("https://hentailib.me/ru/manga/1".into());
    d.library = Library::HentaiLib;
    d.total_main = Some(400);
    d.done_main = 250;
    d.done_volumes = 30;
    s.create_title(d).unwrap();

    let mut d = TitleDraft::new(Medium::Manga, "Ван-Пис");
    d.group_id = Some(custom.id);
    s.create_title(d).unwrap();

    s.create_title(TitleDraft::new(Medium::Anime, "Frieren")).unwrap();
    s
}

/// (название, имя группы, библиотека, прочитано глав, прочитано томов, дата добавления)
type Row = (String, Option<String>, Library, u32, u32, i64);

fn rows(s: &Store, medium: Medium) -> Vec<Row> {
    let groups = s.list_groups(medium).unwrap();
    s.list_titles(medium)
        .unwrap()
        .into_iter()
        .map(|t| {
            let group = t
                .fields
                .group_id
                .and_then(|id| groups.iter().find(|g| g.id == id))
                .map(|g| g.name.clone());
            (t.fields.name, group, t.fields.library, t.fields.done_main, t.fields.done_volumes, t.created_at)
        })
        .collect()
}

fn group_colors(s: &Store, medium: Medium) -> Vec<String> {
    s.list_groups(medium).unwrap().into_iter().map(|g| g.color).collect()
}

#[test]
fn export_then_replace_import_restores_everything() {
    let source = seeded_source();
    let json = source.export_json().unwrap();

    let mut target = store();
    add(&mut target, "Лишний тайтл", None); // должен исчезнуть
    let report = target.import_json(&json, ImportMode::Replace).unwrap();

    assert_eq!(report.titles_added, 3);
    assert_eq!(report.titles_skipped, 0);
    for medium in [Medium::Manga, Medium::Anime] {
        assert_eq!(group_names(&target, medium), group_names(&source, medium));
        assert_eq!(group_colors(&target, medium), group_colors(&source, medium));
        assert_eq!(rows(&target, medium), rows(&source, medium));
    }
}

#[test]
fn replace_keeps_duplicates_but_merge_joins_them() {
    let mut source = store();
    add(&mut source, "Дубль", None);
    add(&mut source, "Дубль", None);
    let json = source.export_json().unwrap();

    let mut replaced = store();
    replaced.import_json(&json, ImportMode::Replace).unwrap();
    assert_eq!(replaced.list_titles(Medium::Manga).unwrap().len(), 2);

    let mut merged = store();
    merged.import_json(&json, ImportMode::Merge).unwrap();
    assert_eq!(merged.list_titles(Medium::Manga).unwrap().len(), 1);
}

#[test]
fn merge_adds_new_titles_and_keeps_the_bigger_progress() {
    let mut target = store();
    let reading = manga(&target, "Читаю");
    let mut mine = TitleDraft::new(Medium::Manga, "Берсерк");
    mine.group_id = Some(reading.id);
    mine.done_main = 100;
    target.create_title(mine).unwrap();

    let json = seeded_source().export_json().unwrap();
    let report = target.import_json(&json, ImportMode::Merge).unwrap();

    assert_eq!(report.titles_merged, 1);
    assert_eq!(report.titles_added, 2); // Ван-Пис и Frieren
    assert_eq!(report.groups_added, 1); // «Перечитываю», остальные уже есть

    let titles = target.list_titles(Medium::Manga).unwrap();
    assert_eq!(titles.len(), 2);
    let berserk = titles.iter().find(|t| t.fields.name == "Берсерк").unwrap();
    assert_eq!(berserk.fields.done_main, 250);
    assert_eq!(berserk.fields.done_volumes, 30);
    assert_eq!(berserk.fields.total_main, Some(400));
    assert_eq!(berserk.fields.original_name.as_deref(), Some("Berserk"));
    // группа и библиотека остаются как у меня
    assert_eq!(berserk.fields.group_id, Some(reading.id));
    assert_eq!(berserk.fields.library, Library::MangaLib);
    assert_eq!(target.list_titles(Medium::Anime).unwrap().len(), 1);
}

#[test]
fn merge_recognizes_the_same_title_by_link() {
    let mut target = store();
    let mut mine = TitleDraft::new(Medium::Manga, "Совсем другое название");
    mine.url = Some("https://hentailib.me/ru/manga/1?section=info".into());
    target.create_title(mine).unwrap();

    let json = seeded_source().export_json().unwrap();
    let report = target.import_json(&json, ImportMode::Merge).unwrap();

    assert_eq!(report.titles_merged, 1);
    // мой тайтл и Ван-Пис, Берсерк не продублировался
    assert_eq!(target.list_titles(Medium::Manga).unwrap().len(), 2);
}

#[test]
fn merge_reuses_groups_with_the_same_name_ignoring_case() {
    let mut target = store();
    let reading = manga(&target, "Читаю");
    target.update_group(reading.id, "ЧИТАЮ", "#112233").unwrap();

    let json = seeded_source().export_json().unwrap();
    target.import_json(&json, ImportMode::Merge).unwrap();

    let berserk = target
        .list_titles(Medium::Manga)
        .unwrap()
        .into_iter()
        .find(|t| t.fields.name == "Берсерк")
        .unwrap();
    assert_eq!(berserk.fields.group_id, Some(reading.id));
    let same_name = group_names(&target, Medium::Manga)
        .iter()
        .filter(|n| n.to_lowercase() == "читаю")
        .count();
    assert_eq!(same_name, 1);
}

#[test]
fn import_rejects_foreign_or_broken_files_and_changes_nothing() {
    let mut s = store();
    add(&mut s, "A", None);

    let broken = s.import_json("{ not json", ImportMode::Replace);
    assert!(matches!(broken, Err(Error::InvalidBackup(_))));

    let foreign = s.import_json(
        r#"{"format":"something-else","version":1,"exported_at":0}"#,
        ImportMode::Replace,
    );
    assert!(matches!(foreign, Err(Error::InvalidBackup(_))));

    let newer = s.import_json(
        &format!(
            r#"{{"format":"manga-tracker-backup","version":{},"exported_at":0}}"#,
            BACKUP_VERSION + 1
        ),
        ImportMode::Merge,
    );
    assert!(matches!(newer, Err(Error::InvalidBackup(_))));

    // пустой файл не должен стирать данные
    let empty = s.import_json(
        r#"{"format":"manga-tracker-backup","version":1,"exported_at":0}"#,
        ImportMode::Replace,
    );
    assert!(matches!(empty, Err(Error::InvalidBackup(_))));

    assert_eq!(s.list_titles(Medium::Manga).unwrap().len(), 1);
    assert_eq!(group_names(&s, Medium::Manga).len(), 5);
}

#[test]
fn import_skips_titles_that_break_the_rules() {
    let mut source = store();
    add(&mut source, "Нормальный", None);
    let mut backup = source.export_backup().unwrap();
    let mut bad = backup.titles[0].clone();
    bad.id = 999;
    bad.fields.name = "   ".into();
    backup.titles.push(bad);

    let mut target = store();
    let report = target.import_backup(backup, ImportMode::Replace).unwrap();

    assert_eq!(report.titles_added, 1);
    assert_eq!(report.titles_skipped, 1);
}

#[test]
fn title_pointing_to_a_missing_group_is_imported_without_group() {
    let mut source = store();
    let reading = manga(&source, "Читаю");
    add(&mut source, "A", Some(reading.id));
    let mut backup = source.export_backup().unwrap();
    backup.groups.clear(); // группы потерялись

    let mut target = store();
    target.import_backup(backup, ImportMode::Merge).unwrap();

    let titles = target.list_titles(Medium::Manga).unwrap();
    assert_eq!(titles.len(), 1);
    assert_eq!(titles[0].fields.group_id, None);
}
