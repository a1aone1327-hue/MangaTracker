//! Операции над группами.

use rusqlite::{params, Connection, OptionalExtension, Row};

use super::store::{now, Store};
use crate::domain::groups as rules;
use crate::domain::{Group, Medium, OnDelete};
use crate::{Error, Result};

const GROUP_COLS: &str = "id, medium, name, color, position";

fn group_from_row(r: &Row<'_>) -> rusqlite::Result<Group> {
    Ok(Group {
        id: r.get("id")?,
        medium: r.get("medium")?,
        name: r.get("name")?,
        color: r.get("color")?,
        position: r.get("position")?,
    })
}

pub(super) fn find_group(conn: &Connection, id: i64) -> Result<Option<Group>> {
    let sql = format!("SELECT {GROUP_COLS} FROM groups WHERE id = ?1");
    Ok(conn.query_row(&sql, params![id], group_from_row).optional()?)
}

pub(super) fn get_group(conn: &Connection, id: i64) -> Result<Group> {
    find_group(conn, id)?.ok_or(Error::NotFound("группа"))
}

pub(super) fn groups_of(conn: &Connection, medium: Medium) -> Result<Vec<Group>> {
    let sql = format!("SELECT {GROUP_COLS} FROM groups WHERE medium = ?1 ORDER BY position, id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map(params![medium], group_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Все группы обоих разделов (для экспорта).
pub(super) fn all_groups(conn: &Connection) -> Result<Vec<Group>> {
    let sql = format!("SELECT {GROUP_COLS} FROM groups ORDER BY medium, position, id");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], group_from_row)?;
    Ok(rows.collect::<rusqlite::Result<Vec<_>>>()?)
}

/// Делает позиции подряд: 0, 1, 2…
fn renumber(conn: &Connection, medium: Medium) -> Result<()> {
    let ids: Vec<i64> = groups_of(conn, medium)?.iter().map(|g| g.id).collect();
    for (pos, id) in ids.iter().enumerate() {
        conn.execute("UPDATE groups SET position = ?1 WHERE id = ?2", params![pos as i64, *id])?;
    }
    Ok(())
}

impl Store {
    pub fn list_groups(&self, medium: Medium) -> Result<Vec<Group>> {
        groups_of(&self.conn, medium)
    }

    /// Новая группа встаёт в конец списка своего раздела.
    pub fn create_group(&mut self, medium: Medium, name: &str, color: &str) -> Result<Group> {
        let existing = groups_of(&self.conn, medium)?;
        let others: Vec<&str> = existing.iter().map(|g| g.name.as_str()).collect();
        let name = rules::validate_group_name(name, &others)?;
        let color = rules::validate_color(color)?;
        let position = existing.len() as i64;
        self.conn.execute(
            "INSERT INTO groups (medium, name, color, position) VALUES (?1, ?2, ?3, ?4)",
            params![medium, name, color, position],
        )?;
        let id = self.conn.last_insert_rowid();
        get_group(&self.conn, id)
    }

    pub fn update_group(&mut self, id: i64, name: &str, color: &str) -> Result<Group> {
        let current = get_group(&self.conn, id)?;
        let siblings = groups_of(&self.conn, current.medium)?;
        let others: Vec<&str> =
            siblings.iter().filter(|g| g.id != id).map(|g| g.name.as_str()).collect();
        let name = rules::validate_group_name(name, &others)?;
        let color = rules::validate_color(color)?;
        self.conn.execute(
            "UPDATE groups SET name = ?1, color = ?2 WHERE id = ?3",
            params![name, color, id],
        )?;
        get_group(&self.conn, id)
    }

    /// `ordered_ids` — все группы раздела в новом порядке.
    pub fn reorder_groups(&mut self, medium: Medium, ordered_ids: &[i64]) -> Result<()> {
        let current: Vec<i64> = groups_of(&self.conn, medium)?.iter().map(|g| g.id).collect();
        rules::check_order(&current, ordered_ids)?;
        let tx = self.conn.transaction()?;
        for (pos, id) in ordered_ids.iter().enumerate() {
            tx.execute("UPDATE groups SET position = ?1 WHERE id = ?2", params![pos as i64, *id])?;
        }
        tx.commit()?;
        Ok(())
    }

    /// Удаляет группу. Тайтлы не пропадают: переносятся в другую группу того же
    /// раздела или остаются без группы. Всё в одной транзакции. Возвращает, сколько
    /// тайтлов затронуто.
    pub fn delete_group(&mut self, id: i64, action: OnDelete) -> Result<usize> {
        let tx = self.conn.transaction()?;
        let group = get_group(&tx, id)?;
        let target: Option<i64> = match action {
            OnDelete::MoveTo(target_id) => {
                let target = get_group(&tx, target_id)?;
                rules::check_move_target(&group, &target)?;
                Some(target.id)
            }
            OnDelete::Ungroup => None,
        };
        let moved = tx.execute(
            "UPDATE titles SET group_id = ?1, updated_at = ?2 WHERE group_id = ?3",
            params![target, now(), id],
        )?;
        tx.execute("DELETE FROM groups WHERE id = ?1", params![id])?;
        renumber(&tx, group.medium)?;
        tx.commit()?;
        Ok(moved)
    }
}
