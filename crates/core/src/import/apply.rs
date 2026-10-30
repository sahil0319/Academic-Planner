//! R48: applies the import plan in one transaction, then syncs sessions (R6).

use chrono::NaiveDate;
use rusqlite::{params, Connection};

use super::plan::build_plan;
use super::types::{
    CourseAdd, CourseUpdate, FieldChange, FieldValue, ImportField, ImportReport, ImportSlot,
    ImportTerm,
};
use crate::db::{active_term_id, fmt_date, insert_slot};
use crate::error::{Error, Result};
use crate::model::{SlotInput, SyncReport};
use crate::schedule::sync_in;

/// R48: plans `text` through the transaction (so the plan equals what `preview_import`
/// returns for the same database), writes exactly that plan, then syncs with
/// min(`effective_from`, `today`) like `change_slot` (R7, research D12), and commits.
/// R42: errors in the plan give `Error::Validation` listing every `path: message`, nothing
/// written. An empty plan writes nothing and does not sync.
pub(super) fn apply(
    conn: &mut Connection,
    text: &str,
    effective_from: NaiveDate,
    today: NaiveDate,
) -> Result<ImportReport> {
    let tx = conn.transaction()?;
    let plan = build_plan(&tx, text, effective_from)?;
    if !plan.errors.is_empty() {
        let lines: Vec<String> = plan.errors.iter().map(ToString::to_string).collect();
        return Err(Error::Validation(lines.join("\n")));
    }
    if plan.is_empty() {
        return Ok(ImportReport {
            plan,
            sync: SyncReport::default(),
        });
    }

    if let Some(term) = &plan.term {
        insert_term(&tx, term)?;
    }
    for add in &plan.add {
        insert_course(&tx, add, effective_from)?;
    }
    for update in &plan.update {
        update_course(&tx, update, effective_from)?;
    }
    let sync = sync_in(&tx, effective_from.min(today))?;
    tx.commit()?;
    Ok(ImportReport { plan, sync })
}

/// R47: inserts the term as the only active term, as `save_term` does.
fn insert_term(conn: &Connection, term: &ImportTerm) -> Result<()> {
    conn.execute("UPDATE term SET is_active = 0 WHERE is_active = 1", [])?;
    conn.execute(
        "INSERT INTO term (name, start_date, end_date, is_active) VALUES (?1, ?2, ?3, 1)",
        params![
            term.name,
            fmt_date(term.start_date),
            fmt_date(term.end_date)
        ],
    )?;
    Ok(())
}

/// R43, R45: inserts a new course in the active term and its slots from `effective_from`.
fn insert_course(conn: &Connection, add: &CourseAdd, effective_from: NaiveDate) -> Result<()> {
    let term_id = active_term_id(conn)?.ok_or(Error::NoActiveTerm)?;
    conn.execute(
        "INSERT INTO course (term_id, code, name, instructor, room, notes, min_attendance_pct,
             credits, rule_set_id)
         VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, ?8)",
        params![
            term_id,
            add.code,
            add.name,
            add.instructor,
            add.room,
            add.min_attendance_pct,
            add.credits,
            add.rule_set.as_ref().map(|r| r.id)
        ],
    )?;
    let course_id = conn.last_insert_rowid();
    for slot in &add.slots {
        insert_import_slot(conn, course_id, slot, effective_from)?;
    }
    Ok(())
}

/// R44: writes the changed fields of a matched course; every other column (code, notes,
/// archived) keeps its stored value. R45, R7: ends the replaced slots the day before
/// `effective_from` and inserts the file's slots from it.
fn update_course(
    conn: &Connection,
    update: &CourseUpdate,
    effective_from: NaiveDate,
) -> Result<()> {
    for change in &update.changes {
        write_change(conn, update.course_id, change)?;
    }
    if let Some(slots) = &update.slots {
        let last_day = effective_from.pred_opt().ok_or_else(|| {
            Error::Validation(format!(
                "effective date {effective_from} has no previous day"
            ))
        })?;
        for slot in &slots.end {
            conn.execute(
                "UPDATE slot SET valid_to = ?1 WHERE id = ?2",
                params![fmt_date(last_day), slot.id],
            )?;
        }
        for slot in &slots.add {
            insert_import_slot(conn, update.course_id, slot, effective_from)?;
        }
    }
    Ok(())
}

fn write_change(conn: &Connection, id: i64, change: &FieldChange) -> Result<()> {
    let sql = |column: &str| format!("UPDATE course SET {column} = ?1 WHERE id = ?2");
    match (change.field, &change.new) {
        (ImportField::Name, FieldValue::Text(v)) => conn.execute(&sql("name"), params![v, id]),
        (ImportField::Instructor, FieldValue::Text(v)) => {
            conn.execute(&sql("instructor"), params![v, id])
        }
        (ImportField::Room, FieldValue::Text(v)) => conn.execute(&sql("room"), params![v, id]),
        (ImportField::Credits, FieldValue::Number(v)) => {
            conn.execute(&sql("credits"), params![v, id])
        }
        (ImportField::MinAttendancePct, FieldValue::Integer(v)) => {
            conn.execute(&sql("min_attendance_pct"), params![v, id])
        }
        (ImportField::RuleSet, FieldValue::RuleSet(v)) => {
            conn.execute(&sql("rule_set_id"), params![v.as_ref().map(|r| r.id), id])
        }
        (field, _) => {
            return Err(Error::Validation(format!(
                "inconsistent value for import field {field:?}"
            )))
        }
    }?;
    Ok(())
}

/// R45: inserts a file slot valid from `effective_from` until the term ends.
fn insert_import_slot(
    conn: &Connection,
    course_id: i64,
    slot: &ImportSlot,
    effective_from: NaiveDate,
) -> Result<()> {
    insert_slot(
        conn,
        &SlotInput {
            course_id,
            weekday: slot.weekday,
            start_time: slot.start,
            end_time: slot.end,
            room: slot.room.clone(),
            valid_from: effective_from,
            valid_to: None,
            week_parity: None,
        },
    )?;
    Ok(())
}
