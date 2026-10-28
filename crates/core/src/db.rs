//! SQLite storage: opening and migrating the database, text codecs for dates and enums,
//! and the M1 operations (terms, holidays, courses, slots, settings, assessments).

// TEMP(M1 skeleton): removed in T024 once the M1 bodies use these items.
#![allow(dead_code, unused_imports)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use directories::ProjectDirs;
use rusqlite::types::Type;
use rusqlite::{params, Connection, OptionalExtension, Row};

use crate::error::{Error, Result};
use crate::model::*;

/// Environment variable that overrides the database path (R24).
pub const DB_ENV_VAR: &str = "ACADEMIC_PLANNER_DB";

/// File name of the database inside the user data directory (R24).
pub const DB_FILE_NAME: &str = "planner.sqlite";

/// Embedded migrations; entry `i` upgrades the schema from version `i` to `i + 1`.
const MIGRATIONS: &[&str] = &[include_str!("migrations/001_init.sql")];

/// Schema version this app writes: the number of embedded migrations (R25).
pub const SCHEMA_VERSION: i64 = MIGRATIONS.len() as i64;

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
    if let Some(path) = env_override.filter(|p| !p.is_empty()) {
        return Ok(PathBuf::from(path));
    }
    data_dir
        .map(|dir| dir.join(DB_FILE_NAME))
        .ok_or(Error::NoDataDir)
}

/// R24: the database path for this user, from `ACADEMIC_PLANNER_DB` or the OS data directory.
pub fn default_db_path() -> Result<PathBuf> {
    let data_dir =
        ProjectDirs::from("", "", "Academic Planner").map(|d| d.data_dir().to_path_buf());
    resolve_db_path(std::env::var_os(DB_ENV_VAR), data_dir)
}

/// R24: creates the parent directory of `path` if it is missing.
fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

impl Planner {
    /// R24, R25: opens (creating if needed) the database at `path` and runs pending migrations.
    pub fn open(path: &Path) -> Result<Self> {
        ensure_parent_dir(path)?;
        Self::init(Connection::open(path)?)
    }

    /// R25: opens a fresh in-memory database with the current schema.
    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?)
    }

    /// R25: sets pragmas and applies pending migrations. A database newer than
    /// `SCHEMA_VERSION` is refused before anything is written; WAL is enabled only after
    /// that check because switching to WAL rewrites the file header.
    fn init(mut conn: Connection) -> Result<Self> {
        conn.pragma_update(None, "foreign_keys", true)?;
        let found: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
        if found > SCHEMA_VERSION {
            return Err(Error::SchemaTooNew {
                found,
                supported: SCHEMA_VERSION,
            });
        }
        // In-memory databases report "memory"; either answer is fine.
        let _mode: String =
            conn.pragma_update_and_check(None, "journal_mode", "WAL", |r| r.get(0))?;
        for (version, sql) in (1_i64..).zip(MIGRATIONS).skip(found as usize) {
            let tx = conn.transaction()?;
            tx.execute_batch(sql)?;
            tx.pragma_update(None, "user_version", version)?;
            tx.commit()?;
        }
        Ok(Planner { conn })
    }

    /// R1: the active term, if any.
    pub fn active_term(&self) -> Result<Option<Term>> {
        match active_term_id(&self.conn)? {
            Some(id) => load_term(&self.conn, id).map(Some),
            None => Ok(None),
        }
    }

    /// R1, R2: creates (`input.id = None`) or updates a term and makes it the only active
    /// term, in one transaction.
    pub fn save_term(&mut self, input: TermInput) -> Result<Term> {
        let tx = self.conn.transaction()?;
        tx.execute("UPDATE term SET is_active = 0 WHERE is_active = 1", [])?;
        let (start, end) = (fmt_date(input.start_date), fmt_date(input.end_date));
        let id = match input.id {
            Some(id) => {
                let changed = tx.execute(
                    "UPDATE term SET name = ?1, start_date = ?2, end_date = ?3, is_active = 1
                     WHERE id = ?4",
                    params![input.name, start, end, id],
                )?;
                if changed == 0 {
                    return Err(Error::NotFound { entity: "term", id });
                }
                id
            }
            None => {
                tx.execute(
                    "INSERT INTO term (name, start_date, end_date, is_active)
                     VALUES (?1, ?2, ?3, 1)",
                    params![input.name, start, end],
                )?;
                tx.last_insert_rowid()
            }
        };
        let term = load_term(&tx, id)?;
        tx.commit()?;
        Ok(term)
    }

    /// Replaces the holidays of a term with `days`, in one transaction.
    pub fn set_holidays(&mut self, term_id: i64, days: &[Holiday]) -> Result<()> {
        let tx = self.conn.transaction()?;
        let exists: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM term WHERE id = ?1)",
            [term_id],
            |r| r.get(0),
        )?;
        if !exists {
            return Err(Error::NotFound {
                entity: "term",
                id: term_id,
            });
        }
        tx.execute("DELETE FROM holiday WHERE term_id = ?1", [term_id])?;
        {
            let mut insert =
                tx.prepare("INSERT INTO holiday (term_id, date, label) VALUES (?1, ?2, ?3)")?;
            for day in days {
                insert.execute(params![term_id, fmt_date(day.date), day.label])?;
            }
        }
        tx.commit()?;
        Ok(())
    }

    /// R1, R2: creates a course in the active term.
    pub fn create_course(&mut self, input: CourseInput) -> Result<Course> {
        let tx = self.conn.transaction()?;
        let term_id = active_term_id(&tx)?.ok_or(Error::NoActiveTerm)?;
        tx.execute(
            "INSERT INTO course (term_id, code, name, instructor, room, notes, min_attendance_pct)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                term_id,
                input.code,
                input.name,
                input.instructor,
                input.room,
                input.notes,
                input.min_attendance_pct
            ],
        )?;
        let course = load_course(&tx, tx.last_insert_rowid())?;
        tx.commit()?;
        Ok(course)
    }

    /// R2: updates a course; its term and archived flag are unchanged.
    pub fn update_course(&mut self, id: i64, input: CourseInput) -> Result<Course> {
        let tx = self.conn.transaction()?;
        let changed = tx.execute(
            "UPDATE course SET code = ?1, name = ?2, instructor = ?3, room = ?4, notes = ?5,
                 min_attendance_pct = ?6
             WHERE id = ?7",
            params![
                input.code,
                input.name,
                input.instructor,
                input.room,
                input.notes,
                input.min_attendance_pct,
                id
            ],
        )?;
        if changed == 0 {
            return Err(Error::NotFound {
                entity: "course",
                id,
            });
        }
        let course = load_course(&tx, id)?;
        tx.commit()?;
        Ok(course)
    }

    /// R3: archives or unarchives a course. Its sessions and assessments are kept.
    pub fn set_archived(&mut self, id: i64, archived: bool) -> Result<()> {
        let changed = self.conn.execute(
            "UPDATE course SET archived = ?1 WHERE id = ?2",
            params![archived, id],
        )?;
        if changed == 0 {
            return Err(Error::NotFound {
                entity: "course",
                id,
            });
        }
        Ok(())
    }

    /// R1, R3: courses of the active term, ordered by code. Archived courses are included
    /// only when `include_archived` is set.
    pub fn list_courses(&self, include_archived: bool) -> Result<Vec<Course>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {COURSE_COLS} FROM course
             WHERE term_id = (SELECT id FROM term WHERE is_active = 1)
               AND (?1 OR archived = 0)
             ORDER BY code"
        ))?;
        let courses = stmt
            .query_map([include_archived], course_from_row)?
            .collect::<rusqlite::Result<_>>()?;
        Ok(courses)
    }

    /// R2: adds a recurring timetable slot.
    pub fn add_slot(&mut self, input: SlotInput) -> Result<Slot> {
        let tx = self.conn.transaction()?;
        tx.execute(
            "INSERT INTO slot (course_id, weekday, start_time, end_time, room, valid_from, valid_to)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                input.course_id,
                weekday_to_db(input.weekday),
                fmt_time(input.start_time),
                fmt_time(input.end_time),
                input.room,
                fmt_date(input.valid_from),
                input.valid_to.map(fmt_date)
            ],
        )?;
        let slot = load_slot(&tx, tx.last_insert_rowid())?;
        tx.commit()?;
        Ok(slot)
    }

    /// R3: slots of non-archived active-term courses valid on `on`. Both validity ends are
    /// inclusive; a slot without `valid_to` runs until the term ends. Ordered by weekday,
    /// then start time.
    pub fn list_slots(&self, on: NaiveDate) -> Result<Vec<Slot>> {
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {SLOT_COLS} FROM slot s
             JOIN course c ON c.id = s.course_id
             JOIN term t ON t.id = c.term_id
             WHERE t.is_active = 1 AND c.archived = 0
               AND s.valid_from <= ?1 AND ?1 <= coalesce(s.valid_to, t.end_date)
             ORDER BY s.weekday, s.start_time, s.id"
        ))?;
        let slots = stmt
            .query_map([fmt_date(on)], slot_from_row)?
            .collect::<rusqlite::Result<_>>()?;
        Ok(slots)
    }

    /// Current settings; a missing key falls back to its §3 default.
    pub fn settings(&self) -> Result<Settings> {
        let defaults = Settings::default();
        let get = |key: &str| -> Result<Option<String>> {
            let value = self
                .conn
                .query_row("SELECT value FROM setting WHERE key = ?1", [key], |r| {
                    r.get(0)
                })
                .optional()?;
            Ok(value)
        };
        let number = |key: &str, default: u32| -> Result<u32> {
            match get(key)? {
                Some(v) => v
                    .parse()
                    .map_err(|_| Error::Corrupt(format!("setting {key} = {v:?}"))),
                None => Ok(default),
            }
        };
        let notifications_enabled = match get(SETTING_NOTIFICATIONS)?.as_deref() {
            Some("1") => true,
            Some("0") => false,
            Some(v) => {
                return Err(Error::Corrupt(format!(
                    "setting {SETTING_NOTIFICATIONS} = {v:?}"
                )))
            }
            None => defaults.notifications_enabled,
        };
        Ok(Settings {
            class_reminder_minutes: number(
                SETTING_CLASS_REMINDER,
                defaults.class_reminder_minutes,
            )?,
            upcoming_window_days: number(SETTING_UPCOMING_WINDOW, defaults.upcoming_window_days)?,
            notifications_enabled,
        })
    }

    /// Stores all settings in one transaction.
    pub fn save_settings(&mut self, s: &Settings) -> Result<()> {
        let tx = self.conn.transaction()?;
        {
            let mut upsert = tx.prepare(
                "INSERT INTO setting (key, value) VALUES (?1, ?2)
                 ON CONFLICT (key) DO UPDATE SET value = excluded.value",
            )?;
            upsert.execute(params![
                SETTING_CLASS_REMINDER,
                s.class_reminder_minutes.to_string()
            ])?;
            upsert.execute(params![
                SETTING_UPCOMING_WINDOW,
                s.upcoming_window_days.to_string()
            ])?;
            upsert.execute(params![
                SETTING_NOTIFICATIONS,
                if s.notifications_enabled { "1" } else { "0" }
            ])?;
        }
        tx.commit()?;
        Ok(())
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

// ---- row loaders (take `&Connection` so they also work inside a transaction) ----

/// R1: id of the active term, if any.
fn active_term_id(conn: &Connection) -> Result<Option<i64>> {
    let id = conn
        .query_row("SELECT id FROM term WHERE is_active = 1", [], |r| r.get(0))
        .optional()?;
    Ok(id)
}

fn load_term(conn: &Connection, id: i64) -> Result<Term> {
    let term = conn
        .query_row(
            "SELECT id, name, start_date, end_date, is_active FROM term WHERE id = ?1",
            [id],
            |r| {
                Ok(Term {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    start_date: col_date(r, 2)?,
                    end_date: col_date(r, 3)?,
                    is_active: r.get(4)?,
                    holidays: Vec::new(),
                })
            },
        )
        .optional()?
        .ok_or(Error::NotFound { entity: "term", id })?;
    let mut stmt =
        conn.prepare("SELECT date, label FROM holiday WHERE term_id = ?1 ORDER BY date")?;
    let holidays = stmt
        .query_map([id], |r| {
            Ok(Holiday {
                date: col_date(r, 0)?,
                label: r.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(Term { holidays, ..term })
}

const COURSE_COLS: &str =
    "id, term_id, code, name, instructor, room, notes, min_attendance_pct, archived";

fn course_from_row(r: &Row) -> rusqlite::Result<Course> {
    Ok(Course {
        id: r.get(0)?,
        term_id: r.get(1)?,
        code: r.get(2)?,
        name: r.get(3)?,
        instructor: r.get(4)?,
        room: r.get(5)?,
        notes: r.get(6)?,
        min_attendance_pct: r.get(7)?,
        archived: r.get(8)?,
    })
}

fn load_course(conn: &Connection, id: i64) -> Result<Course> {
    conn.query_row(
        &format!("SELECT {COURSE_COLS} FROM course WHERE id = ?1"),
        [id],
        course_from_row,
    )
    .optional()?
    .ok_or(Error::NotFound {
        entity: "course",
        id,
    })
}

const SLOT_COLS: &str = "s.id, s.course_id, s.weekday, s.start_time, s.end_time, s.room, \
     s.valid_from, s.valid_to";

fn slot_from_row(r: &Row) -> rusqlite::Result<Slot> {
    Ok(Slot {
        id: r.get(0)?,
        course_id: r.get(1)?,
        weekday: col_weekday(r, 2)?,
        start_time: col_time(r, 3)?,
        end_time: col_time(r, 4)?,
        room: r.get(5)?,
        valid_from: col_date(r, 6)?,
        valid_to: col_date_opt(r, 7)?,
    })
}

fn load_slot(conn: &Connection, id: i64) -> Result<Slot> {
    conn.query_row(
        &format!("SELECT {SLOT_COLS} FROM slot s WHERE s.id = ?1"),
        [id],
        slot_from_row,
    )
    .optional()?
    .ok_or(Error::NotFound { entity: "slot", id })
}

// ---- setting keys (docs/spec.md §3) ----

const SETTING_CLASS_REMINDER: &str = "class_reminder_minutes";
const SETTING_UPCOMING_WINDOW: &str = "upcoming_window_days";
const SETTING_NOTIFICATIONS: &str = "notifications_enabled";

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
    fn r24_override_wins() {
        let path = resolve_db_path(
            Some(OsString::from("/tmp/custom.sqlite")),
            Some(PathBuf::from("/data")),
        )
        .unwrap();
        assert_eq!(path, PathBuf::from("/tmp/custom.sqlite"));
    }

    #[test]
    fn r24_default_is_data_dir_planner_sqlite() {
        let path = resolve_db_path(None, Some(PathBuf::from("/data"))).unwrap();
        assert_eq!(path, PathBuf::from("/data").join("planner.sqlite"));
        // An empty override counts as unset.
        let path = resolve_db_path(Some(OsString::new()), Some(PathBuf::from("/data"))).unwrap();
        assert_eq!(path, PathBuf::from("/data").join("planner.sqlite"));
    }

    #[test]
    fn r24_no_override_no_data_dir_is_error() {
        assert!(matches!(resolve_db_path(None, None), Err(Error::NoDataDir)));
    }

    #[test]
    fn r24_ensure_parent_dir_creates_missing_dirs() {
        let root = std::env::temp_dir().join(format!("planner-r24-{}", std::process::id()));
        let file = root.join("a").join("b").join(DB_FILE_NAME);
        assert!(!root.exists());
        ensure_parent_dir(&file).unwrap();
        assert!(file.parent().unwrap().is_dir());
        assert!(!file.exists(), "only the directory is created");
        std::fs::remove_dir_all(&root).unwrap();
        // A bare file name or an SQLite URI has no directory to create.
        ensure_parent_dir(Path::new("planner.sqlite")).unwrap();
    }

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
