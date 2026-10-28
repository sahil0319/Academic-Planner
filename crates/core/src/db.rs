//! SQLite storage: opening and migrating the database, text codecs for dates and enums,
//! and the M1 operations (terms, holidays, courses, slots, settings, assessments).

// TEMP(M1 skeleton): removed in T024 once the M1 bodies use these items.
#![allow(dead_code, unused_imports)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use rusqlite::types::Type;
use rusqlite::{Connection, Row};

use crate::error::{Error, Result};
use crate::model::*;

/// Environment variable that overrides the database path (R24).
pub const DB_ENV_VAR: &str = "ACADEMIC_PLANNER_DB";

/// File name of the database inside the user data directory (R24).
pub const DB_FILE_NAME: &str = "planner.sqlite";

/// Schema version this app writes: the number of embedded migrations (R25).
pub const SCHEMA_VERSION: i64 = 1;

/// The planner: one open SQLite connection. Every mutation commits before returning.
pub struct Planner {
    pub(crate) conn: Connection,
}

/// R24: picks the database path. A non-empty `env_override` wins; otherwise the file is
/// `planner.sqlite` in `data_dir`.
pub fn resolve_db_path(
    env_override: Option<OsString>,
    data_dir: Option<PathBuf>,
) -> Result<PathBuf> {
    let _ = (env_override, data_dir);
    todo!("M1")
}

/// R24: the database path for this user, from `ACADEMIC_PLANNER_DB` or the OS data directory.
pub fn default_db_path() -> Result<PathBuf> {
    todo!("M1")
}

impl Planner {
    /// R24, R25: opens (creating if needed) the database at `path` and runs pending migrations.
    pub fn open(path: &Path) -> Result<Self> {
        let _ = path;
        todo!("M1")
    }

    /// R25: opens a fresh in-memory database with the current schema.
    pub fn open_in_memory() -> Result<Self> {
        todo!("M1")
    }

    /// R1: the active term, if any.
    pub fn active_term(&self) -> Result<Option<Term>> {
        todo!("M1")
    }

    /// R1, R2: creates or updates a term and makes it the only active term.
    pub fn save_term(&mut self, input: TermInput) -> Result<Term> {
        let _ = input;
        todo!("M1")
    }

    /// Replaces the holidays of a term with `days`.
    pub fn set_holidays(&mut self, term_id: i64, days: &[Holiday]) -> Result<()> {
        let _ = (term_id, days);
        todo!("M1")
    }

    /// R1, R2: creates a course in the active term.
    pub fn create_course(&mut self, input: CourseInput) -> Result<Course> {
        let _ = input;
        todo!("M1")
    }

    /// R2: updates a course.
    pub fn update_course(&mut self, id: i64, input: CourseInput) -> Result<Course> {
        let _ = (id, input);
        todo!("M1")
    }

    /// R3: archives or unarchives a course. Its sessions and assessments are kept.
    pub fn set_archived(&mut self, id: i64, archived: bool) -> Result<()> {
        let _ = (id, archived);
        todo!("M1")
    }

    /// R1, R3: courses of the active term, ordered by code.
    pub fn list_courses(&self, include_archived: bool) -> Result<Vec<Course>> {
        let _ = include_archived;
        todo!("M1")
    }

    /// R2: adds a recurring timetable slot.
    pub fn add_slot(&mut self, input: SlotInput) -> Result<Slot> {
        let _ = input;
        todo!("M1")
    }

    /// Slots of non-archived active-term courses valid on `on` (both ends inclusive).
    pub fn list_slots(&self, on: NaiveDate) -> Result<Vec<Slot>> {
        let _ = on;
        todo!("M1")
    }

    /// Current settings; missing keys fall back to the §3 defaults.
    pub fn settings(&self) -> Result<Settings> {
        todo!("M1")
    }

    /// Stores all settings.
    pub fn save_settings(&mut self, s: &Settings) -> Result<()> {
        let _ = s;
        todo!("M1")
    }

    /// R2: creates an assessment.
    pub fn create_assessment(&mut self, input: AssessmentInput) -> Result<Assessment> {
        let _ = input;
        todo!("M1")
    }

    /// R2: updates an assessment.
    pub fn update_assessment(&mut self, id: i64, input: AssessmentInput) -> Result<Assessment> {
        let _ = (id, input);
        todo!("M1")
    }

    /// Marks an assessment completed or not completed.
    pub fn set_completed(&mut self, id: i64, done: bool) -> Result<()> {
        let _ = (id, done);
        todo!("M1")
    }

    /// Deletes an assessment.
    pub fn delete_assessment(&mut self, id: i64) -> Result<()> {
        let _ = id;
        todo!("M1")
    }

    /// Assessments of non-archived active-term courses matching `filter`, sorted by due date.
    pub fn list_assessments(&self, filter: AssessmentFilter) -> Result<Vec<Assessment>> {
        let _ = filter;
        todo!("M1")
    }
}

// ---- text codecs (docs/spec.md §1: YYYY-MM-DD, HH:MM, YYYY-MM-DDTHH:MM) ----

const DATE_FMT: &str = "%Y-%m-%d";
const TIME_FMT: &str = "%H:%M";
const DATETIME_FMT: &str = "%Y-%m-%dT%H:%M";

pub(crate) fn fmt_date(d: NaiveDate) -> String {
    d.format(DATE_FMT).to_string()
}

pub(crate) fn fmt_time(t: NaiveTime) -> String {
    t.format(TIME_FMT).to_string()
}

pub(crate) fn fmt_datetime(dt: NaiveDateTime) -> String {
    dt.format(DATETIME_FMT).to_string()
}

pub(crate) fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, DATE_FMT).ok()
}

pub(crate) fn parse_time(s: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(s, TIME_FMT).ok()
}

pub(crate) fn parse_datetime(s: &str) -> Option<NaiveDateTime> {
    NaiveDateTime::parse_from_str(s, DATETIME_FMT).ok()
}

/// ISO weekday number: Monday = 1 … Sunday = 7.
pub(crate) fn weekday_to_db(w: Weekday) -> i64 {
    i64::from(w.number_from_monday())
}

pub(crate) fn weekday_from_db(n: i64) -> Option<Weekday> {
    const DAYS: [Weekday; 7] = [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
        Weekday::Sat,
        Weekday::Sun,
    ];
    usize::try_from(n - 1)
        .ok()
        .and_then(|i| DAYS.get(i).copied())
}

impl Status {
    pub(crate) fn as_db(self) -> &'static str {
        match self {
            Status::Pending => "pending",
            Status::Attended => "attended",
            Status::Absent => "absent",
            Status::Cancelled => "cancelled",
        }
    }

    pub(crate) fn from_db(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Status::Pending),
            "attended" => Some(Status::Attended),
            "absent" => Some(Status::Absent),
            "cancelled" => Some(Status::Cancelled),
            _ => None,
        }
    }
}

impl SessionKind {
    pub(crate) fn as_db(self) -> &'static str {
        match self {
            SessionKind::Regular => "regular",
            SessionKind::Makeup => "makeup",
        }
    }

    pub(crate) fn from_db(s: &str) -> Option<Self> {
        match s {
            "regular" => Some(SessionKind::Regular),
            "makeup" => Some(SessionKind::Makeup),
            _ => None,
        }
    }
}

impl AssessmentKind {
    pub(crate) fn as_db(self) -> &'static str {
        match self {
            AssessmentKind::Quiz => "quiz",
            AssessmentKind::Exam => "exam",
            AssessmentKind::Assignment => "assignment",
            AssessmentKind::Other => "other",
        }
    }

    pub(crate) fn from_db(s: &str) -> Option<Self> {
        match s {
            "quiz" => Some(AssessmentKind::Quiz),
            "exam" => Some(AssessmentKind::Exam),
            "assignment" => Some(AssessmentKind::Assignment),
            "other" => Some(AssessmentKind::Other),
            _ => None,
        }
    }
}

impl ReminderKind {
    pub(crate) fn as_db(self) -> &'static str {
        match self {
            ReminderKind::Session => "session",
            ReminderKind::Assessment => "assessment",
        }
    }

    pub(crate) fn from_db(s: &str) -> Option<Self> {
        match s {
            "session" => Some(ReminderKind::Session),
            "assessment" => Some(ReminderKind::Assessment),
            _ => None,
        }
    }
}

// ---- column decoders: a value that does not parse becomes `Error::Corrupt` ----

fn bad_value(idx: usize, what: &str, value: &str) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(
        idx,
        Type::Text,
        format!("{what} {value:?} in column {idx}").into(),
    )
}

fn decode<T>(
    row: &Row,
    idx: usize,
    what: &str,
    parse: fn(&str) -> Option<T>,
) -> rusqlite::Result<T> {
    let s: String = row.get(idx)?;
    parse(&s).ok_or_else(|| bad_value(idx, what, &s))
}

fn decode_opt<T>(
    row: &Row,
    idx: usize,
    what: &str,
    parse: fn(&str) -> Option<T>,
) -> rusqlite::Result<Option<T>> {
    let s: Option<String> = row.get(idx)?;
    s.map(|s| parse(&s).ok_or_else(|| bad_value(idx, what, &s)))
        .transpose()
}

pub(crate) fn col_date(row: &Row, idx: usize) -> rusqlite::Result<NaiveDate> {
    decode(row, idx, "date", parse_date)
}

pub(crate) fn col_date_opt(row: &Row, idx: usize) -> rusqlite::Result<Option<NaiveDate>> {
    decode_opt(row, idx, "date", parse_date)
}

pub(crate) fn col_time(row: &Row, idx: usize) -> rusqlite::Result<NaiveTime> {
    decode(row, idx, "time", parse_time)
}

pub(crate) fn col_datetime(row: &Row, idx: usize) -> rusqlite::Result<NaiveDateTime> {
    decode(row, idx, "date-time", parse_datetime)
}

pub(crate) fn col_weekday(row: &Row, idx: usize) -> rusqlite::Result<Weekday> {
    let n: i64 = row.get(idx)?;
    weekday_from_db(n).ok_or_else(|| bad_value(idx, "weekday", &n.to_string()))
}

pub(crate) fn col_status(row: &Row, idx: usize) -> rusqlite::Result<Status> {
    decode(row, idx, "status", Status::from_db)
}

pub(crate) fn col_session_kind(row: &Row, idx: usize) -> rusqlite::Result<SessionKind> {
    decode(row, idx, "session kind", SessionKind::from_db)
}

pub(crate) fn col_assessment_kind(row: &Row, idx: usize) -> rusqlite::Result<AssessmentKind> {
    decode(row, idx, "assessment kind", AssessmentKind::from_db)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_time_codecs_round_trip() {
        let d = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
        assert_eq!(fmt_date(d), "2026-01-05");
        assert_eq!(parse_date("2026-01-05"), Some(d));

        let t = NaiveTime::from_hms_opt(9, 5, 0).unwrap();
        assert_eq!(fmt_time(t), "09:05");
        assert_eq!(parse_time("09:05"), Some(t));

        let dt = d.and_time(t);
        assert_eq!(fmt_datetime(dt), "2026-01-05T09:05");
        assert_eq!(parse_datetime("2026-01-05T09:05"), Some(dt));
    }

    #[test]
    fn seconds_are_dropped_on_write() {
        let t = NaiveTime::from_hms_opt(9, 5, 42).unwrap();
        assert_eq!(fmt_time(t), "09:05");
        let dt = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap().and_time(t);
        assert_eq!(fmt_datetime(dt), "2026-01-05T09:05");
    }

    #[test]
    fn malformed_text_does_not_parse() {
        assert_eq!(parse_date("05/01/2026"), None);
        assert_eq!(parse_time("9am"), None);
        assert_eq!(parse_datetime("2026-01-05 09:05"), None);
    }

    #[test]
    fn weekday_uses_iso_numbering() {
        assert_eq!(weekday_to_db(Weekday::Mon), 1);
        assert_eq!(weekday_to_db(Weekday::Sun), 7);
        for n in 1..=7 {
            assert_eq!(weekday_from_db(n).map(weekday_to_db), Some(n));
        }
        assert_eq!(weekday_from_db(0), None);
        assert_eq!(weekday_from_db(8), None);
    }

    #[test]
    fn enum_text_round_trips() {
        for s in [
            Status::Pending,
            Status::Attended,
            Status::Absent,
            Status::Cancelled,
        ] {
            assert_eq!(Status::from_db(s.as_db()), Some(s));
        }
        for k in [SessionKind::Regular, SessionKind::Makeup] {
            assert_eq!(SessionKind::from_db(k.as_db()), Some(k));
        }
        for k in [
            AssessmentKind::Quiz,
            AssessmentKind::Exam,
            AssessmentKind::Assignment,
            AssessmentKind::Other,
        ] {
            assert_eq!(AssessmentKind::from_db(k.as_db()), Some(k));
        }
        for k in [ReminderKind::Session, ReminderKind::Assessment] {
            assert_eq!(ReminderKind::from_db(k.as_db()), Some(k));
        }
        assert_eq!(Status::from_db("rescheduled"), None);
    }
}
