//! R42-R47: builds the import plan. Reads the database, never writes it.

use chrono::NaiveDate;
use rusqlite::Connection;

use super::parse::{self, Context, Field, ParsedCourse};
use super::types::{
    CourseAdd, CourseUpdate, FieldChange, FieldValue, ImportField, ImportPlan, RuleSetRef,
};
use crate::db::{active_term_id, course_from_row, load_term, COURSE_COLS};
use crate::error::Result;
use crate::model::Course;

/// R42-R48: parses and validates `text` against the database and returns the plan.
/// With errors, the plan carries only `errors` and `warnings` (R42: nothing may be written).
pub(super) fn build_plan(
    conn: &Connection,
    text: &str,
    _effective_from: NaiveDate,
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
            Some(i) => plan
                .update
                .extend(course_update(&courses[i], course, &rule_sets)),
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

/// R43, R44, R46: the field changes of a matched course; `None` when nothing differs. The
/// stored code, notes, and archived flag are never changed. R44: an archived match stays
/// archived.
fn course_update(
    stored: &Course,
    course: ParsedCourse,
    rule_sets: &[RuleSetRef],
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
    if changes.is_empty() {
        return None;
    }
    Some(CourseUpdate {
        course_id: stored.id,
        code: stored.code.clone(),
        archived: stored.archived,
        changes,
        slots: None,
    })
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
