//! R96-R98: exports courses and their routine in the import format of §13.4, so the document
//! can be pasted into another planner's Import panel. Reads the database, never writes it.

use chrono::{NaiveDate, Weekday};
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;

use crate::db::{
    active_term_id, course_from_row, fmt_date, fmt_time, load_term, slot_from_row, Planner,
    COURSE_COLS, SLOT_COLS,
};
use crate::error::{Error, Result};
use crate::model::{Course, Slot, WeekParity};
use crate::schedule::term_week_parity;

/// R96: the document root, in the key order of the §13.4 example.
#[derive(Serialize)]
struct Doc {
    format: &'static str,
    version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    term: Option<TermOut>,
    courses: Vec<CourseOut>,
}

#[derive(Serialize)]
struct TermOut {
    name: String,
    start: String,
    end: String,
}

/// R97: one course. `credits` and `rule_set` are omitted when the course is not graded;
/// `instructor`, `room`, and `min_attendance_pct` are `null` when empty. Notes, ids, and
/// everything personal are not part of the document.
#[derive(Serialize)]
struct CourseOut {
    code: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    credits: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rule_set: Option<String>,
    instructor: Option<String>,
    room: Option<String>,
    min_attendance_pct: Option<i64>,
    slots: Vec<SlotOut>,
}

/// R97, R54: one slot; `biweekly` is omitted for a weekly slot.
#[derive(Serialize)]
struct SlotOut {
    day: &'static str,
    start: String,
    end: String,
    room: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    biweekly: Option<&'static str>,
}

impl Planner {
    /// R96, R97: exports courses as pretty-printed import JSON (§13.4, version 1).
    ///
    /// `course_ids = None` exports every non-archived course of the active term, ordered by
    /// code; `Some(ids)` exports exactly those courses (also archived ones), ordered by code.
    /// An unknown id is `Error::NotFound`; an empty list gives an empty `courses` array.
    /// Slots are those that have not ended on `today` (`valid_to` NULL or on or after it).
    /// `biweekly` is `this_week` for the parity of `today`'s term week and `next_week` for the
    /// other one (R54, R59), so an import with the effective date `today` recreates the pattern
    /// (R98). `term` is the active term; it is omitted when there is none.
    pub fn export_courses_json(
        &self,
        course_ids: Option<&[i64]>,
        today: NaiveDate,
    ) -> Result<String> {
        let conn = &self.conn;
        let term = match active_term_id(conn)? {
            Some(id) => Some(load_term(conn, id)?),
            None => None,
        };
        let mut courses = match course_ids {
            None => match &term {
                Some(term) => active_courses(conn, term.id)?,
                None => Vec::new(),
            },
            Some(ids) => chosen_courses(conn, ids)?,
        };
        courses.sort_by(|a, b| a.code.cmp(&b.code).then(a.id.cmp(&b.id)));

        let mut out = Vec::with_capacity(courses.len());
        for course in courses {
            out.push(course_out(conn, course, today)?);
        }
        let doc = Doc {
            format: "academic-planner-courses",
            version: 1,
            term: term.map(|t| TermOut {
                name: t.name,
                start: fmt_date(t.start_date),
                end: fmt_date(t.end_date),
            }),
            courses: out,
        };
        serde_json::to_string_pretty(&doc)
            .map_err(|e| Error::Corrupt(format!("export_courses_json: {e}")))
    }
}

/// R96: non-archived courses of the term.
fn active_courses(conn: &Connection, term_id: i64) -> Result<Vec<Course>> {
    let mut stmt = conn.prepare(&format!(
        "SELECT {COURSE_COLS} FROM course WHERE term_id = ?1 AND archived = 0"
    ))?;
    let courses = stmt
        .query_map([term_id], course_from_row)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(courses)
}

/// R96: the listed courses; a repeated id counts once, an unknown id is `Error::NotFound`.
fn chosen_courses(conn: &Connection, ids: &[i64]) -> Result<Vec<Course>> {
    let mut seen = Vec::new();
    let mut courses = Vec::new();
    for &id in ids {
        if seen.contains(&id) {
            continue;
        }
        seen.push(id);
        let course = conn
            .query_row(
                &format!("SELECT {COURSE_COLS} FROM course WHERE id = ?1"),
                [id],
                course_from_row,
            )
            .optional()?
            .ok_or(Error::NotFound {
                entity: "course",
                id,
            })?;
        courses.push(course);
    }
    Ok(courses)
}

/// R97: one course with its rule set name and its slots that have not ended.
fn course_out(conn: &Connection, course: Course, today: NaiveDate) -> Result<CourseOut> {
    let rule_set = match course.rule_set_id {
        Some(id) => conn
            .query_row("SELECT name FROM rule_set WHERE id = ?1", [id], |r| {
                r.get::<_, String>(0)
            })
            .optional()?,
        None => None,
    };
    let term_start = load_term(conn, course.term_id)?.start_date;
    let mut stmt = conn.prepare(&format!(
        "SELECT {SLOT_COLS} FROM slot s
         WHERE s.course_id = ?1 AND (s.valid_to IS NULL OR s.valid_to >= ?2)
         ORDER BY s.weekday, s.start_time, s.id"
    ))?;
    let slots: Vec<Slot> = stmt
        .query_map(rusqlite::params![course.id, fmt_date(today)], slot_from_row)?
        .collect::<rusqlite::Result<_>>()?;
    let this_week = term_week_parity(term_start, today);
    Ok(CourseOut {
        code: course.code,
        name: course.name,
        credits: course.credits,
        rule_set,
        instructor: course.instructor,
        room: course.room,
        min_attendance_pct: course.min_attendance_pct,
        slots: slots
            .into_iter()
            .map(|s| slot_out(s, this_week))
            .collect(),
    })
}

/// R54, R59: a slot in the week of `today` is `this_week`, in the other parity `next_week`.
fn slot_out(slot: Slot, this_week: WeekParity) -> SlotOut {
    SlotOut {
        day: day_name(slot.weekday),
        start: fmt_time(slot.start_time),
        end: fmt_time(slot.end_time),
        room: slot.room,
        biweekly: slot.week_parity.map(|p| {
            if p == this_week {
                "this_week"
            } else {
                "next_week"
            }
        }),
    }
}

/// R97: the day names the importer reads (R42).
fn day_name(day: Weekday) -> &'static str {
    match day {
        Weekday::Mon => "Mon",
        Weekday::Tue => "Tue",
        Weekday::Wed => "Wed",
        Weekday::Thu => "Thu",
        Weekday::Fri => "Fri",
        Weekday::Sat => "Sat",
        Weekday::Sun => "Sun",
    }
}
