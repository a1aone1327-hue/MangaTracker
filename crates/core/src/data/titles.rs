//! Операции над тайтлами.

use rusqlite::{named_params, params, Connection, OptionalExtension, Row};

use super::groups::get_group;
use super::store::{now, Store};
use crate::domain::{Counter, DomainError, Library, Medium, Title, TitleDraft};
use crate::{Error, Result};

const TITLE_COLS: &str = "id, medium, kind, library, group_id, name, original_name, cover, url, \
     note, total_main, total_volumes, done_main, done_volumes, created_at, updated_at, \
     progress_updated_at, library_changed_at";

fn title_from_row(r: &Row<'_>) -> rusqlite::Result<Title> {
    Ok(Title {
        id: r.get("id")?,
        fields: TitleDraft {
            medium: r.get("medium")?,
            kind: r.get("kind")?,
            library: r.get("library")?,
            group_id: r.get("group_id")?,
            name: r.get("name")?,
            original_name: r.get("original_name")?,
            cover: r.get("cover")?,
            url: r.get("url")?,
            note: r.get("note")?,
            total_main: r.get("total_main")?,
            total_volumes: r.get("total_volumes")?,
            done_main: r.get("done_main")?,
            done_volumes: r.get("done_volumes")?,
        },
        created_at: r.get("created_at")?,
        updated_at: r.get("updated_at")?,
        progress_updated_at: r.get("progress_updated_at")?,
        library_changed_at: r.get("library_changed_at")?,
    })
}

fn find_title(conn: &Connection, id: i64) -> Result<Option<Title>> {
    let sql = format!("SELECT {TITLE_COLS} FROM titles WHERE id = ?1");
    Ok(conn.query_row(&sql, params![id], title_from_row).optional()?)
}

fn get_title(conn: &Connection, id: i64) -> Result<Title> {
    find_title(conn, id)?.ok_or(Error::NotFound("тайтл"))
}

/// Все тайтлы обоих разделов (для экспорта и слияния).
pub(super) fn all_titles(conn: &Connection) -> Result<Vec<Title>> {
    let sql = format!("SELECT {TITLE_COLS} FROM titles ORDER BY id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], title_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Группа должна существовать и относиться к тому же разделу, что и тайтл.
fn check_group_for(conn: &Connection, group_id: Option<i64>, medium: Medium) -> Result<()> {
    if let Some(gid) = group_id {
        let group = get_group(conn, gid)?;
        if group.medium != medium {
            return Err(DomainError::GroupMediumMismatch.into());
        }
    }
    Ok(())
}

impl Store {
    pub fn get_title(&self, id: i64) -> Result<Title> {
        get_title(&self.conn, id)
    }

    pub fn list_titles(&self, medium: Medium) -> Result<Vec<Title>> {
        let sql = format!("SELECT {TITLE_COLS} FROM titles WHERE medium = ?1 ORDER BY id");
        let mut stmt = self.conn.prepare(&sql)?;
        let rows = stmt.query_map(params![medium], title_from_row)?;
        Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
    }

    pub fn create_title(&mut self, mut draft: TitleDraft) -> Result<Title> {
        draft.prepare()?;
        check_group_for(&self.conn, draft.group_id, draft.medium)?;
        let t = now();
        self.conn.execute(
            "INSERT INTO titles (medium, kind, library, group_id, name, original_name, cover, url, \
                 note, total_main, total_volumes, done_main, done_volumes, \
                 created_at, updated_at, progress_updated_at, library_changed_at) \
             VALUES (:medium, :kind, :library, :group_id, :name, :original_name, :cover, :url, \
                 :note, :total_main, :total_volumes, :done_main, :done_volumes, \
                 :t, :t, :t, NULL)",
            named_params! {
                ":medium": draft.medium,
                ":kind": draft.kind,
                ":library": draft.library,
                ":group_id": draft.group_id,
                ":name": draft.name,
                ":original_name": draft.original_name,
                ":cover": draft.cover,
                ":url": draft.url,
                ":note": draft.note,
                ":total_main": draft.total_main,
                ":total_volumes": draft.total_volumes,
                ":done_main": draft.done_main,
                ":done_volumes": draft.done_volumes,
                ":t": t,
            },
        )?;
        let id = self.conn.last_insert_rowid();
        get_title(&self.conn, id)
    }

    /// Полное обновление полей из формы. Раздел менять нельзя.
    pub fn update_title(&mut self, id: i64, mut draft: TitleDraft) -> Result<Title> {
        let current = get_title(&self.conn, id)?;
        if draft.medium != current.fields.medium {
            return Err(DomainError::MediumImmutable.into());
        }
        draft.prepare()?;
        check_group_for(&self.conn, draft.group_id, draft.medium)?;

        let t = now();
        let library_changed_at = if draft.library != current.fields.library {
            Some(t)
        } else {
            current.library_changed_at
        };
        let progress_changed = draft.done_main != current.fields.done_main
            || draft.done_volumes != current.fields.done_volumes;
        let progress_updated_at = if progress_changed { t } else { current.progress_updated_at };

        self.conn.execute(
            "UPDATE titles SET kind = :kind, library = :library, group_id = :group_id, \
                 name = :name, original_name = :original_name, cover = :cover, url = :url, \
                 note = :note, total_main = :total_main, total_volumes = :total_volumes, \
                 done_main = :done_main, done_volumes = :done_volumes, \
                 updated_at = :t, progress_updated_at = :progress_updated_at, \
                 library_changed_at = :library_changed_at \
             WHERE id = :id",
            named_params! {
                ":kind": draft.kind,
                ":library": draft.library,
                ":group_id": draft.group_id,
                ":name": draft.name,
                ":original_name": draft.original_name,
                ":cover": draft.cover,
                ":url": draft.url,
                ":note": draft.note,
                ":total_main": draft.total_main,
                ":total_volumes": draft.total_volumes,
                ":done_main": draft.done_main,
                ":done_volumes": draft.done_volumes,
                ":t": t,
                ":progress_updated_at": progress_updated_at,
                ":library_changed_at": library_changed_at,
                ":id": id,
            },
        )?;
        get_title(&self.conn, id)
    }

    /// Быстрые кнопки +1/−1 глава/том/серия.
    pub fn step_progress(&mut self, id: i64, counter: Counter, delta: i32) -> Result<Title> {
        let current = get_title(&self.conn, id)?;
        let mut fields = current.fields.clone();
        if !fields.step(counter, delta) {
            return Ok(current);
        }
        let t = now();
        self.conn.execute(
            "UPDATE titles SET done_main = ?1, done_volumes = ?2, \
                 progress_updated_at = ?3, updated_at = ?3 WHERE id = ?4",
            params![fields.done_main, fields.done_volumes, t, id],
        )?;
        get_title(&self.conn, id)
    }

    /// Массовый перенос в группу (`None` значит «без группы»).
    /// Всё или ничего: если хоть один тайтл из другого раздела, не меняется ничего.
    pub fn move_titles(&mut self, ids: &[i64], group_id: Option<i64>) -> Result<usize> {
        let tx = self.conn.transaction()?;
        if let Some(gid) = group_id {
            let group = get_group(&tx, gid)?;
            for id in ids {
                if let Some(title) = find_title(&tx, *id)? {
                    if title.fields.medium != group.medium {
                        return Err(DomainError::GroupMediumMismatch.into());
                    }
                }
            }
        }
        let ts = now();
        let mut changed = 0;
        for id in ids {
            changed += tx.execute(
                "UPDATE titles SET group_id = ?1, updated_at = ?2 WHERE id = ?3",
                params![group_id, ts, *id],
            )?;
        }
        tx.commit()?;
        Ok(changed)
    }

    /// Массовая смена библиотеки (тайтлы переехали, например, с MangaLib на HentaiLib).
    /// Возвращает, у скольких тайтлов библиотека реально поменялась.
    pub fn set_library(&mut self, ids: &[i64], library: Library) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let ts = now();
        let mut changed = 0;
        for id in ids {
            changed += tx.execute(
                "UPDATE titles SET library = ?1, library_changed_at = ?2, updated_at = ?2 \
                 WHERE id = ?3 AND library <> ?1",
                params![library, ts, *id],
            )?;
        }
        tx.commit()?;
        Ok(changed)
    }

    pub fn delete_titles(&mut self, ids: &[i64]) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let mut deleted = 0;
        for id in ids {
            deleted += tx.execute("DELETE FROM titles WHERE id = ?1", params![*id])?;
        }
        tx.commit()?;
        Ok(deleted)
    }
}
