//! Экспорт и импорт всей базы в JSON.

use std::collections::HashMap;

use rusqlite::{named_params, params, Connection};

use super::groups::{all_groups, groups_of};
use super::store::{now, Store};
use super::titles::all_titles;
use crate::domain::backup::{
    merge_titles, name_key, url_key, Backup, ImportMode, ImportReport, BACKUP_FORMAT,
    BACKUP_VERSION,
};
use crate::domain::groups::{validate_color, MAX_GROUP_NAME_CHARS};
use crate::domain::{Medium, Title, TitleDraft};
use crate::{Error, Result};

const FALLBACK_COLOR: &str = "#9a9ab5";

fn invalid(why: &str) -> Error {
    Error::InvalidBackup(why.to_string())
}

impl Store {
    /// Снимок всей базы (оба раздела).
    pub fn export_backup(&self) -> Result<Backup> {
        Ok(Backup {
            format: BACKUP_FORMAT.to_string(),
            version: BACKUP_VERSION,
            exported_at: now(),
            groups: all_groups(&self.conn)?,
            titles: all_titles(&self.conn)?,
        })
    }

    pub fn export_json(&self) -> Result<String> {
        serde_json::to_string(&self.export_backup()?).map_err(|e| invalid(&e.to_string()))
    }

    pub fn import_json(&mut self, json: &str, mode: ImportMode) -> Result<ImportReport> {
        let backup: Backup = serde_json::from_str(json).map_err(|e| invalid(&e.to_string()))?;
        self.import_backup(backup, mode)
    }

    /// Импорт одной транзакцией: при любой ошибке база остаётся как была.
    pub fn import_backup(&mut self, backup: Backup, mode: ImportMode) -> Result<ImportReport> {
        if backup.format != BACKUP_FORMAT {
            return Err(invalid("это не копия Manga Tracker"));
        }
        if backup.version > BACKUP_VERSION {
            return Err(invalid("файл создан более новой версией приложения"));
        }
        if mode == ImportMode::Replace && backup.groups.is_empty() && backup.titles.is_empty() {
            // пустой файл не должен стирать данные
            return Err(invalid("в файле нет данных"));
        }
        let tx = self.conn.transaction()?;
        let report = import_into(&tx, backup, mode)?;
        tx.commit()?;
        Ok(report)
    }
}

/// Для поиска дубликатов при слиянии: по названию и по ссылке.
#[derive(Default)]
struct Index {
    by_name: HashMap<String, usize>,
    by_url: HashMap<String, usize>,
}

impl Index {
    fn add(&mut self, position: usize, fields: &TitleDraft) {
        self.by_name.entry(name_key(fields)).or_insert(position);
        if let Some(key) = url_key(fields) {
            self.by_url.entry(key).or_insert(position);
        }
    }

    fn find(&self, fields: &TitleDraft) -> Option<usize> {
        self.by_name
            .get(&name_key(fields))
            .copied()
            .or_else(|| url_key(fields).and_then(|key| self.by_url.get(&key).copied()))
    }
}

fn import_into(conn: &Connection, backup: Backup, mode: ImportMode) -> Result<ImportReport> {
    let mut report = ImportReport::default();

    if mode == ImportMode::Replace {
        conn.execute("DELETE FROM titles", [])?;
        conn.execute("DELETE FROM groups", [])?;
    }

    // ---- группы: id в файле -> (id в базе, раздел) ----
    let mut group_map: HashMap<i64, (i64, Medium)> = HashMap::new();
    let mut groups = backup.groups;
    groups.sort_by(|a, b| {
        (a.medium.as_str(), a.position, a.id).cmp(&(b.medium.as_str(), b.position, b.id))
    });
    for group in &groups {
        let name: String = group.name.trim().chars().take(MAX_GROUP_NAME_CHARS).collect();
        if name.is_empty() {
            continue;
        }
        let color = validate_color(&group.color).unwrap_or_else(|_| FALLBACK_COLOR.to_string());
        let lower = name.to_lowercase();
        let siblings = groups_of(conn, group.medium)?;
        let existing = siblings.iter().find(|g| g.name.to_lowercase() == lower).map(|g| g.id);
        let id = match existing {
            Some(id) => id,
            None => {
                conn.execute(
                    "INSERT INTO groups (medium, name, color, position) VALUES (?1, ?2, ?3, ?4)",
                    params![group.medium, name, color, siblings.len() as i64],
                )?;
                report.groups_added += 1;
                conn.last_insert_rowid()
            }
        };
        group_map.insert(group.id, (id, group.medium));
    }

    // ---- тайтлы ----
    let dedupe = mode == ImportMode::Merge;
    let mut existing: Vec<Title> = if dedupe { all_titles(conn)? } else { Vec::new() };
    let mut index = Index::default();
    for (position, title) in existing.iter().enumerate() {
        index.add(position, &title.fields);
    }

    let mut incoming = backup.titles;
    incoming.sort_by_key(|t| t.id);
    let ts = now();

    for title in incoming {
        let mut fields = title.fields;
        let medium = fields.medium;
        // группа берётся только если она есть в базе и из того же раздела
        fields.group_id = fields
            .group_id
            .and_then(|old| group_map.get(&old).copied())
            .and_then(|(id, group_medium)| if group_medium == medium { Some(id) } else { None });
        if fields.prepare().is_err() {
            report.titles_skipped += 1;
            continue;
        }

        let duplicate = if dedupe { index.find(&fields) } else { None };
        match duplicate {
            Some(position) => {
                let current = &existing[position];
                let merged = merge_titles(&current.fields, &fields);
                let progress_changed = merged.done_main != current.fields.done_main
                    || merged.done_volumes != current.fields.done_volumes;
                let progress_updated_at = if progress_changed {
                    current.progress_updated_at.max(title.progress_updated_at)
                } else {
                    current.progress_updated_at
                };
                conn.execute(
                    "UPDATE titles SET kind = :kind, original_name = :original_name, \
                         cover = :cover, url = :url, note = :note, \
                         total_main = :total_main, total_volumes = :total_volumes, \
                         done_main = :done_main, done_volumes = :done_volumes, \
                         updated_at = :t, progress_updated_at = :progress_updated_at \
                     WHERE id = :id",
                    named_params! {
                        ":kind": merged.kind,
                        ":original_name": merged.original_name,
                        ":cover": merged.cover,
                        ":url": merged.url,
                        ":note": merged.note,
                        ":total_main": merged.total_main,
                        ":total_volumes": merged.total_volumes,
                        ":done_main": merged.done_main,
                        ":done_volumes": merged.done_volumes,
                        ":t": ts,
                        ":progress_updated_at": progress_updated_at,
                        ":id": current.id,
                    },
                )?;
                existing[position].fields = merged;
                existing[position].progress_updated_at = progress_updated_at;
                index.add(position, &existing[position].fields);
                report.titles_merged += 1;
            }
            None => {
                let id = insert_title(
                    conn,
                    &fields,
                    title.created_at,
                    title.updated_at,
                    title.progress_updated_at,
                    title.library_changed_at,
                )?;
                if dedupe {
                    index.add(existing.len(), &fields);
                    existing.push(Title {
                        id,
                        fields,
                        created_at: title.created_at,
                        updated_at: title.updated_at,
                        progress_updated_at: title.progress_updated_at,
                        library_changed_at: title.library_changed_at,
                    });
                }
                report.titles_added += 1;
            }
        }
    }

    Ok(report)
}

/// Вставка с сохранением исходных меток времени (в отличие от `create_title`).
fn insert_title(
    conn: &Connection,
    f: &TitleDraft,
    created_at: i64,
    updated_at: i64,
    progress_updated_at: i64,
    library_changed_at: Option<i64>,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO titles (medium, kind, library, group_id, name, original_name, cover, url, \
             note, total_main, total_volumes, done_main, done_volumes, \
             created_at, updated_at, progress_updated_at, library_changed_at) \
         VALUES (:medium, :kind, :library, :group_id, :name, :original_name, :cover, :url, \
             :note, :total_main, :total_volumes, :done_main, :done_volumes, \
             :created_at, :updated_at, :progress_updated_at, :library_changed_at)",
        named_params! {
            ":medium": f.medium,
            ":kind": f.kind,
            ":library": f.library,
            ":group_id": f.group_id,
            ":name": f.name,
            ":original_name": f.original_name,
            ":cover": f.cover,
            ":url": f.url,
            ":note": f.note,
            ":total_main": f.total_main,
            ":total_volumes": f.total_volumes,
            ":done_main": f.done_main,
            ":done_volumes": f.done_volumes,
            ":created_at": created_at,
            ":updated_at": updated_at,
            ":progress_updated_at": progress_updated_at,
            ":library_changed_at": library_changed_at,
        },
    )?;
    Ok(conn.last_insert_rowid())
}
