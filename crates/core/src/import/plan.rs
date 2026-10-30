//! R42-R47: builds the import plan. Reads the database, never writes it.

use chrono::NaiveDate;
use rusqlite::Connection;

use super::parse::{self, Context, Field, ParsedCourse};
use super::types::{
    CourseAdd, CourseUpdate, FieldChange, FieldValue, ImportField, ImportPlan, ImportSlot,
    RuleSetRef, SlotChange,
};
use crate::db::{
    active_term_id, course_from_row, load_term, slot_from_row, COURSE_COLS, SLOT_COLS,
};
use crate::error::Result;
use crate::model::{Course, Slot, WeekParity};

/// R42-R48: parses and validates `text` against the database and returns the plan.
/// With errors, the plan carries only `errors` and `warnings` (R42: nothing may be written).
pub(super) fn build_plan(
    conn: &Connection,
    text: &str,
    effective_from: NaiveDate,
) -> Result<ImportPlan> {
    let active_term = match active_term_id(conn)? {
        Some(id) => Some(load_term(conn, id)?),
        None => None,
    };
    let courses = match &active_term {
        Some(term) => load_term_courses(conn, term.id)?,
        None => Vec::new(),
    };
    let rule_sets = load_rule_set_refs(conn)?;
    let ctx = Context {
        active_term: active_term.as_ref(),
        rule_sets: &rule_sets,
        courses: &courses,
        effective_from,
    };
    let parsed = parse::parse(text, &ctx);
    let mut plan = ImportPlan {
        term: None,
        add: Vec::new(),
        update: Vec::new(),
        warnings: parsed.warnings,
        errors: parsed.errors,
    };
    if !plan.errors.is_empty() {
        return Ok(plan);
    }

    // R47: the file's term is created only when no term is active.
    if active_term.is_none() {
        plan.term = parsed.term;
    }
    for course in parsed.courses {
        match course.matched {
            None => plan.add.extend(course_add(course)),
            Some(i) => {
                let stored_slots = load_course_slots(conn, courses[i].id)?;
                plan.update.extend(course_update(
                    &courses[i],
                    course,
                    &rule_sets,
                    &stored_slots,
                    effective_from,
                ));
            }
        }
    }
    Ok(plan)
}

/// The value of a field, `None` for absent and `null`.
fn value<T>(field: Field<T>) -> Option<T> {
    match field {
        Field::Value(v) => Some(v),
        Field::Absent | Field::Null => None,
    }
}

/// R44: the new value of a field when the file changes it: `None` = keep (absent, or equal to
/// the stored value), `Some(None)` = clear (`null` on a set value), `Some(Some(v))` = replace.
fn changed<T: PartialEq>(field: Field<T>, stored: Option<&T>) -> Option<Option<T>> {
    match field {
        Field::Absent => None,
        Field::Null => stored.is_some().then_some(None),
        Field::Value(v) => (stored != Some(&v)).then_some(Some(v)),
    }
}

/// R43-R46: the field and slot changes of a matched course; `None` when nothing differs.
/// The stored code, notes, and archived flag are never changed. R44: an archived match stays
/// archived.
fn course_update(
    stored: &Course,
    course: ParsedCourse,
    rule_sets: &[RuleSetRef],
    stored_slots: &[Slot],
    effective_from: NaiveDate,
) -> Option<CourseUpdate> {
    let mut changes = Vec::new();
    if let Some(new) = changed(course.name, Some(&stored.name)) {
        changes.push(FieldChange {
            field: ImportField::Name,
            old: FieldValue::Text(Some(stored.name.clone())),
            new: FieldValue::Text(new),
        });
    }
    if let Some(new) = changed(course.credits, stored.credits.as_ref()) {
        changes.push(FieldChange {
            field: ImportField::Credits,
            old: FieldValue::Number(stored.credits),
            new: FieldValue::Number(new),
        });
    }
    let stored_rule_set = stored
        .rule_set_id
        .and_then(|id| rule_sets.iter().find(|r| r.id == id).cloned());
    if let Some(new) = changed(course.rule_set, stored_rule_set.as_ref()) {
        changes.push(FieldChange {
            field: ImportField::RuleSet,
            old: FieldValue::RuleSet(stored_rule_set),
            new: FieldValue::RuleSet(new),
        });
    }
    if let Some(new) = changed(course.instructor, stored.instructor.as_ref()) {
        changes.push(FieldChange {
            field: ImportField::Instructor,
            old: FieldValue::Text(stored.instructor.clone()),
            new: FieldValue::Text(new),
        });
    }
    if let Some(new) = changed(course.room, stored.room.as_ref()) {
        changes.push(FieldChange {
            field: ImportField::Room,
            old: FieldValue::Text(stored.room.clone()),
            new: FieldValue::Text(new),
        });
    }
    if let Some(new) = changed(
        course.min_attendance_pct,
        stored.min_attendance_pct.as_ref(),
    ) {
        changes.push(FieldChange {
            field: ImportField::MinAttendancePct,
            old: FieldValue::Integer(stored.min_attendance_pct),
            new: FieldValue::Integer(new),
        });
    }
    // R45: a missing `slots` key leaves the slots untouched.
    let slots = match course.slots {
        Field::Value(file_slots) => slot_change(file_slots, stored_slots, effective_from),
        Field::Absent | Field::Null => None,
    };
    if changes.is_empty() && slots.is_none() {
        return None;
    }
    Some(CourseUpdate {
        course_id: stored.id,
        code: stored.code.clone(),
        archived: stored.archived,
        changes,
        slots,
    })
}

/// A slot as the comparison sees it: weekday, start, end, room, week parity (R45, R59).
type SlotKey = (
    u32,
    chrono::NaiveTime,
    chrono::NaiveTime,
    Option<String>,
    Option<&'static str>,
);

fn file_key(s: &ImportSlot) -> SlotKey {
    (
        s.weekday.num_days_from_monday(),
        s.start,
        s.end,
        s.room.clone(),
        s.week_parity.map(WeekParity::as_db),
    )
}

fn stored_key(s: &Slot) -> SlotKey {
    (
        s.weekday.num_days_from_monday(),
        s.start_time,
        s.end_time,
        s.room.clone(),
        s.week_parity.map(WeekParity::as_db),
    )
}

/// R45, R7: compares the file's slots, as a sorted multiset of (day, start, end, room, parity), with
/// the stored slots valid on `effective_from` (a missing room equals a `null` room).
/// Equal: no change. Different: every stored slot not ended before `effective_from` is ended
/// (`valid_to = E - 1`, like `change_slot`), including slots that start after it so they cannot
/// duplicate the new ones (OQ-3, router ruling), and the file's slots are added from
/// `effective_from`. `"slots": []` ends all current slots and adds none.
fn slot_change(
    file_slots: Vec<ImportSlot>,
    stored_slots: &[Slot],
    effective_from: NaiveDate,
) -> Option<SlotChange> {
    let current: Vec<&Slot> = stored_slots
        .iter()
        .filter(|s| s.valid_to.is_none_or(|to| to >= effective_from))
        .collect();
    let mut valid_now: Vec<SlotKey> = current
        .iter()
        .filter(|s| s.valid_from <= effective_from)
        .map(|s| stored_key(s))
        .collect();
    let mut wanted: Vec<SlotKey> = file_slots.iter().map(file_key).collect();
    valid_now.sort();
    wanted.sort();
    let nothing_to_end = !wanted.is_empty() || current.is_empty();
    if valid_now == wanted && nothing_to_end {
        return None;
    }
    Some(SlotChange {
        end: current.into_iter().cloned().collect(),
        add: file_slots,
    })
}

/// R45: every slot of a course in id order, ended ones included.
fn load_course_slots(conn: &Connection, course_id: i64) -> Result<Vec<Slot>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {SLOT_COLS} FROM slot s WHERE s.course_id = ?1 ORDER BY s.id"
    ))?;
    let slots = stmt
        .query_map([course_id], slot_from_row)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(slots)
}

/// R43: a course whose code matches no stored course. Absent and `null` both mean "none".
/// The parser guarantees a name for a new course.
fn course_add(course: ParsedCourse) -> Option<CourseAdd> {
    Some(CourseAdd {
        code: course.code,
        name: value(course.name)?,
        instructor: value(course.instructor),
        room: value(course.room),
        min_attendance_pct: value(course.min_attendance_pct),
        credits: value(course.credits),
        rule_set: value(course.rule_set),
        slots: value(course.slots).unwrap_or_default(),
    })
}

/// R43: every course of the term, archived included, in id order.
fn load_term_courses(conn: &Connection, term_id: i64) -> Result<Vec<Course>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COURSE_COLS} FROM course WHERE term_id = ?1 ORDER BY id"
    ))?;
    let courses = stmt
        .query_map([term_id], course_from_row)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(courses)
}

/// R42: id and name of every rule set, for the exact-name lookup.
fn load_rule_set_refs(conn: &Connection) -> Result<Vec<RuleSetRef>> {
    let mut stmt = conn.prepare("SELECT id, name FROM rule_set ORDER BY id")?;
    let refs = stmt
        .query_map([], |r| {
            Ok(RuleSetRef {
                id: r.get(0)?,
                name: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(refs)
}
