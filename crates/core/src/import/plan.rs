//! R42-R47: builds the import plan. Reads the database, never writes it.

use chrono::NaiveDate;
use rusqlite::Connection;

use super::parse::{self, Context};
use super::types::{ImportPlan, RuleSetRef};
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
    Ok(ImportPlan {
        term: None,
        add: Vec::new(),
        update: Vec::new(),
        warnings: parsed.warnings,
        errors: parsed.errors,
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
