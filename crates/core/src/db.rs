//! SQLite storage: opening and migrating the database, text codecs for dates and enums,
//! and the M1 operations (terms, holidays, courses, slots, settings, assessments).

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Timelike, Weekday};
use directories::ProjectDirs;
use rusqlite::types::Type;
use rusqlite::{params, params_from_iter, Connection, OptionalExtension, Row};

use crate::error::{Error, Result};
use crate::migrate;
use crate::model::*;

/// Environment variable that overrides the database path (R24).
pub const DB_ENV_VAR: &str = "ACADEMIC_PLANNER_DB";

/// File name of the database inside the user data directory (R24).
pub const DB_FILE_NAME: &str = "planner.sqlite";

/// Embedded migrations; entry `i` upgrades the schema from version `i` to `i + 1`.
const MIGRATIONS: &[&str] = &[
    include_str!("migrations/001_init.sql"),
    // Migration 002 creates the grading tables; the seeds follow in the same script (D2).
    concat!(
        include_str!("migrations/002_grading.sql"),
        include_str!("grading_seed.sql")
    ),
];

/// R41: the default rule sets, bands, and grade scale; run by migration 002 and by a
/// version-1 backup restore.
pub(crate) const GRADING_SEED: &str = include_str!("grading_seed.sql");

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
    /// R24, R25, R26: opens (creating if needed) the database at `path` and runs pending
    /// migrations.
    pub fn open(path: &Path) -> Result<Self> {
        ensure_parent_dir(path)?;
        Self::init(Connection::open(path)?, Some(path))
    }

    /// R25: opens a fresh in-memory database with the current schema; never writes a copy.
    pub fn open_in_memory() -> Result<Self> {
        Self::init(Connection::open_in_memory()?, None)
    }

    /// R25: enables foreign keys, then applies pending migrations through `migrate::run`
    /// (which refuses a newer schema before anything is written).
    fn init(mut conn: Connection, db_path: Option<&Path>) -> Result<Self> {
        conn.pragma_update(None, "foreign_keys", true)?;
        migrate::run(&mut conn, db_path, MIGRATIONS)?;
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
        input.validate()?;
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
        input.validate()?;
        let tx = self.conn.transaction()?;
        let term_id = active_term_id(&tx)?.ok_or(Error::NoActiveTerm)?;
        ensure_code_free(&tx, term_id, &input.code, None)?;
        ensure_rule_set_exists(&tx, input.rule_set_id)?;
        tx.execute(
            "INSERT INTO course (term_id, code, name, instructor, room, notes, min_attendance_pct,
                 credits, rule_set_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                term_id,
                input.code,
                input.name,
                input.instructor,
                input.room,
                input.notes,
                input.min_attendance_pct,
                input.credits,
                input.rule_set_id
            ],
        )?;
        let course = load_course(&tx, tx.last_insert_rowid())?;
        tx.commit()?;
        Ok(course)
    }

    /// R2: updates a course; its term and archived flag are unchanged.
    pub fn update_course(&mut self, id: i64, input: CourseInput) -> Result<Course> {
        input.validate()?;
        let tx = self.conn.transaction()?;
        let term_id = load_course(&tx, id)?.term_id;
        ensure_code_free(&tx, term_id, &input.code, Some(id))?;
        ensure_rule_set_exists(&tx, input.rule_set_id)?;
        tx.execute(
            "UPDATE course SET code = ?1, name = ?2, instructor = ?3, room = ?4, notes = ?5,
                 min_attendance_pct = ?6, credits = ?7, rule_set_id = ?8
             WHERE id = ?9",
            params![
                input.code,
                input.name,
                input.instructor,
                input.room,
                input.notes,
                input.min_attendance_pct,
                input.credits,
                input.rule_set_id,
                id
            ],
        )?;
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

    /// R51: permanently deletes the courses with their slots, sessions, assessments, and the
    /// `reminder_log` rows of those sessions and assessments, in one transaction. A makeup that
    /// replaced a deleted session keeps its row but loses the link. Any unknown id is
    /// `NotFound` and nothing is deleted; duplicate ids are allowed.
    pub fn delete_courses(&mut self, ids: &[i64]) -> Result<()> {
        let tx = self.conn.transaction()?;
        for &id in ids {
            load_course(&tx, id)?;
        }
        for &id in ids {
            tx.execute(
                "DELETE FROM reminder_log
                 WHERE (kind = 'session'
                        AND ref_id IN (SELECT id FROM session WHERE course_id = ?1))
                    OR (kind = 'assessment'
                        AND ref_id IN (SELECT id FROM assessment WHERE course_id = ?1))",
                [id],
            )?;
            tx.execute(
                "UPDATE session SET replaces_session_id = NULL
                 WHERE replaces_session_id IN (SELECT id FROM session WHERE course_id = ?1)",
                [id],
            )?;
            tx.execute("DELETE FROM session WHERE course_id = ?1", [id])?;
            tx.execute("DELETE FROM slot WHERE course_id = ?1", [id])?;
            tx.execute("DELETE FROM assessment WHERE course_id = ?1", [id])?;
            tx.execute("DELETE FROM course WHERE id = ?1", [id])?;
        }
        tx.commit()?;
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
        input.validate()?;
        let tx = self.conn.transaction()?;
        let slot = insert_slot(&tx, &input)?;
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
        // §5 Settings theme: a missing row reads as System; an unknown value is corrupt.
        let theme = match get(SETTING_THEME)? {
            Some(v) => ThemeMode::from_db(&v)
                .ok_or_else(|| Error::Corrupt(format!("setting {SETTING_THEME} = {v:?}")))?,
            None => defaults.theme,
        };
        // §3, R29: a missing row reads as on; anything but "0"/"1" is corrupt.
        let check_updates_on_start = match get(SETTING_CHECK_UPDATES)?.as_deref() {
            Some("1") => true,
            Some("0") => false,
            Some(v) => {
                return Err(Error::Corrupt(format!(
                    "setting {SETTING_CHECK_UPDATES} = {v:?}"
                )))
            }
            None => defaults.check_updates_on_start,
        };
        Ok(Settings {
            class_reminder_minutes: number(
                SETTING_CLASS_REMINDER,
                defaults.class_reminder_minutes,
            )?,
            upcoming_window_days: number(SETTING_UPCOMING_WINDOW, defaults.upcoming_window_days)?,
            notifications_enabled,
            theme,
            check_updates_on_start,
        })
    }

    /// Stores all settings in one transaction, `theme` (§5 Settings theme) and
    /// `check_updates_on_start` (R29) included.
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
            upsert.execute(params![SETTING_THEME, s.theme.as_db()])?;
            upsert.execute(params![
                SETTING_CHECK_UPDATES,
                if s.check_updates_on_start { "1" } else { "0" }
            ])?;
        }
        tx.commit()?;
        Ok(())
    }

    /// R2: creates an assessment.
    pub fn create_assessment(&mut self, input: AssessmentInput) -> Result<Assessment> {
        input.validate()?;
        let tx = self.conn.transaction()?;
        ensure_component_link(&tx, &input)?;
        tx.execute(
            "INSERT INTO assessment (course_id, kind, title, due_at, notes, remind_minutes_before,
                 component_id, score)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                input.course_id,
                input.kind.as_db(),
                input.title,
                fmt_datetime(input.due_at),
                input.notes,
                input.remind_minutes_before,
                input.component_id,
                input.score
            ],
        )?;
        let assessment = load_assessment(&tx, tx.last_insert_rowid())?;
        tx.commit()?;
        Ok(assessment)
    }

    /// R2: updates an assessment; its completed flag is unchanged.
    pub fn update_assessment(&mut self, id: i64, input: AssessmentInput) -> Result<Assessment> {
        input.validate()?;
        let tx = self.conn.transaction()?;
        ensure_component_link(&tx, &input)?;
        let changed = tx.execute(
            "UPDATE assessment SET course_id = ?1, kind = ?2, title = ?3, due_at = ?4, notes = ?5,
                 remind_minutes_before = ?6, component_id = ?7, score = ?8
             WHERE id = ?9",
            params![
                input.course_id,
                input.kind.as_db(),
                input.title,
                fmt_datetime(input.due_at),
                input.notes,
                input.remind_minutes_before,
                input.component_id,
                input.score,
                id
            ],
        )?;
        if changed == 0 {
            return Err(Error::NotFound {
                entity: "assessment",
                id,
            });
        }
        let assessment = load_assessment(&tx, id)?;
        tx.commit()?;
        Ok(assessment)
    }

    /// Marks an assessment completed or not completed.
    pub fn set_completed(&mut self, id: i64, done: bool) -> Result<()> {
        let changed = self.conn.execute(
            "UPDATE assessment SET completed = ?1 WHERE id = ?2",
            params![done, id],
        )?;
        if changed == 0 {
            return Err(Error::NotFound {
                entity: "assessment",
                id,
            });
        }
        Ok(())
    }

    /// Deletes an assessment.
    pub fn delete_assessment(&mut self, id: i64) -> Result<()> {
        let changed = self
            .conn
            .execute("DELETE FROM assessment WHERE id = ?1", [id])?;
        if changed == 0 {
            return Err(Error::NotFound {
                entity: "assessment",
                id,
            });
        }
        Ok(())
    }

    /// R3: assessments of non-archived active-term courses matching `filter`, sorted by
    /// due date, then id. R18: `Upcoming` = incomplete with
    /// `now <= due_at <= now + upcoming_window_days`; R19: `Overdue` = incomplete with
    /// `due_at < now`. `now` is compared at minute precision (research R-D11, R-D12).
    pub fn list_assessments(&self, filter: AssessmentFilter) -> Result<Vec<Assessment>> {
        let (condition, bounds): (&str, Vec<NaiveDateTime>) = match filter {
            AssessmentFilter::All => ("1", vec![]),
            AssessmentFilter::Completed => ("a.completed = 1", vec![]),
            AssessmentFilter::Upcoming { now } => {
                let now = to_minute(now);
                let days = i64::from(self.settings()?.upcoming_window_days);
                match now.checked_add_signed(chrono::TimeDelta::days(days)) {
                    Some(until) => (
                        "a.completed = 0 AND a.due_at >= ?1 AND a.due_at <= ?2",
                        vec![now, until],
                    ),
                    // A window past the calendar end has no upper bound.
                    None => ("a.completed = 0 AND a.due_at >= ?1", vec![now]),
                }
            }
            AssessmentFilter::Overdue { now } => {
                ("a.completed = 0 AND a.due_at < ?1", vec![to_minute(now)])
            }
        };
        let mut stmt = self.conn.prepare(&format!(
            "SELECT {ASSESSMENT_COLS} FROM assessment a
             JOIN course c ON c.id = a.course_id
             JOIN term t ON t.id = c.term_id
             WHERE t.is_active = 1 AND c.archived = 0 AND {condition}
             ORDER BY a.due_at, a.id"
        ))?;
        let assessments = stmt
            .query_map(
                params_from_iter(bounds.into_iter().map(fmt_datetime)),
                assessment_from_row,
            )?
            .collect::<rusqlite::Result<_>>()?;
        Ok(assessments)
    }
}

/// R33: a course's `rule_set_id`, when present, must name an existing rule set.
fn ensure_rule_set_exists(conn: &Connection, rule_set_id: Option<i64>) -> Result<()> {
    let Some(id) = rule_set_id else {
        return Ok(());
    };
    let exists: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM rule_set WHERE id = ?1)",
        [id],
        |r| r.get(0),
    )?;
    if exists {
        Ok(())
    } else {
        Err(Error::Validation(format!("rule set {id} does not exist")))
    }
}

/// R36: an assessment linked to a component needs a component of its course's rule set (also
/// for an ungraded course, so none can link), and a present score must not exceed the
/// component's effective item max (`item_max`, or the component's marks when NULL).
fn ensure_component_link(conn: &Connection, input: &AssessmentInput) -> Result<()> {
    let Some(component_id) = input.component_id else {
        return Ok(());
    };
    let course = load_course(conn, input.course_id)?;
    let component: Option<(i64, f64, Option<f64>)> = conn
        .query_row(
            "SELECT rule_set_id, weight_pct, item_max FROM rule_component WHERE id = ?1",
            [component_id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )
        .optional()?;
    let Some((rule_set_id, weight_pct, item_max)) = component else {
        return Err(Error::Validation(format!(
            "component {component_id} does not exist"
        )));
    };
    if course.rule_set_id != Some(rule_set_id) {
        return Err(Error::Validation(format!(
            "component {component_id} is not in the rule set of course {}",
            course.code
        )));
    }
    if let Some(score) = input.score {
        let max = item_max.or_else(|| {
            course
                .credits
                .map(|credits| crate::marks::component_total(weight_pct, 100.0 * credits))
        });
        if let Some(max) = max {
            if score > max + 1e-9 {
                return Err(Error::Validation(format!(
                    "score {score} is above the item maximum {max}"
                )));
            }
        }
    }
    Ok(())
}

/// R2: course codes are unique within a term. `except` is the course being updated.
fn ensure_code_free(
    conn: &Connection,
    term_id: i64,
    code: &str,
    except: Option<i64>,
) -> Result<()> {
    let taken: bool = conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM course WHERE term_id = ?1 AND code = ?2 AND id IS NOT ?3)",
        params![term_id, code, except],
        |r| r.get(0),
    )?;
    if taken {
        return Err(Error::Validation(format!(
            "course code {code:?} already exists in this term"
        )));
    }
    Ok(())
}

// ---- row loaders (take `&Connection` so they also work inside a transaction) ----

/// R1: id of the active term, if any.
pub(crate) fn active_term_id(conn: &Connection) -> Result<Option<i64>> {
    let id = conn
        .query_row("SELECT id FROM term WHERE is_active = 1", [], |r| r.get(0))
        .optional()?;
    Ok(id)
}

pub(crate) fn load_term(conn: &Connection, id: i64) -> Result<Term> {
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

pub(crate) const COURSE_COLS: &str =
    "id, term_id, code, name, instructor, room, notes, min_attendance_pct, archived, \
     credits, rule_set_id";

pub(crate) fn course_from_row(r: &Row) -> rusqlite::Result<Course> {
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
        credits: r.get(9)?,
        rule_set_id: r.get(10)?,
    })
}

pub(crate) fn load_course(conn: &Connection, id: i64) -> Result<Course> {
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

/// Inserts a slot row exactly as given (validation is the caller's job) and returns it.
pub(crate) fn insert_slot(conn: &Connection, input: &SlotInput) -> Result<Slot> {
    conn.execute(
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
    load_slot(conn, conn.last_insert_rowid())
}

pub(crate) const SLOT_COLS: &str =
    "s.id, s.course_id, s.weekday, s.start_time, s.end_time, s.room, \
     s.valid_from, s.valid_to";

pub(crate) fn slot_from_row(r: &Row) -> rusqlite::Result<Slot> {
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

pub(crate) fn load_slot(conn: &Connection, id: i64) -> Result<Slot> {
    conn.query_row(
        &format!("SELECT {SLOT_COLS} FROM slot s WHERE s.id = ?1"),
        [id],
        slot_from_row,
    )
    .optional()?
    .ok_or(Error::NotFound { entity: "slot", id })
}

const ASSESSMENT_COLS: &str = "a.id, a.course_id, c.code, a.kind, a.title, a.due_at, a.notes, \
     a.completed, a.remind_minutes_before, a.component_id, a.score";

fn assessment_from_row(r: &Row) -> rusqlite::Result<Assessment> {
    Ok(Assessment {
        id: r.get(0)?,
        course_id: r.get(1)?,
        course_code: r.get(2)?,
        kind: col_assessment_kind(r, 3)?,
        title: r.get(4)?,
        due_at: col_datetime(r, 5)?,
        notes: r.get(6)?,
        completed: r.get(7)?,
        remind_minutes_before: r.get(8)?,
        component_id: r.get(9)?,
        score: r.get(10)?,
    })
}

fn load_assessment(conn: &Connection, id: i64) -> Result<Assessment> {
    conn.query_row(
        &format!(
            "SELECT {ASSESSMENT_COLS} FROM assessment a JOIN course c ON c.id = a.course_id
             WHERE a.id = ?1"
        ),
        [id],
        assessment_from_row,
    )
    .optional()?
    .ok_or(Error::NotFound {
        entity: "assessment",
        id,
    })
}

/// §3: the `session s` columns `session_from_row` decodes, in order.
pub(crate) const SESSION_COLS: &str = "s.id, s.course_id, s.slot_id, s.kind, s.date, \
     s.start_time, s.end_time, s.room, s.status, s.replaces_session_id, s.note";

/// Decodes the `SESSION_COLS` columns of a `session s` row.
pub(crate) fn session_from_row(r: &Row) -> rusqlite::Result<Session> {
    Ok(Session {
        id: r.get(0)?,
        course_id: r.get(1)?,
        slot_id: r.get(2)?,
        kind: col_session_kind(r, 3)?,
        date: col_date(r, 4)?,
        start_time: col_time(r, 5)?,
        end_time: col_time(r, 6)?,
        room: r.get(7)?,
        status: col_status(r, 8)?,
        replaces_session_id: r.get(9)?,
        note: r.get(10)?,
    })
}

/// One session by id; `Error::NotFound` when it does not exist.
pub(crate) fn load_session(conn: &Connection, id: i64) -> Result<Session> {
    conn.query_row(
        &format!("SELECT {SESSION_COLS} FROM session s WHERE s.id = ?1"),
        [id],
        session_from_row,
    )
    .optional()?
    .ok_or(Error::NotFound {
        entity: "session",
        id,
    })
}

/// R12, R17: the moment a session starts.
pub(crate) fn session_start(s: &Session) -> NaiveDateTime {
    s.date.and_time(s.start_time)
}

/// R20: the moment a session ends.
pub(crate) fn session_end(s: &Session) -> NaiveDateTime {
    s.date.and_time(s.end_time)
}

/// Research R-D12: drops seconds (and below) so `now` compares at the minute precision of
/// stored values.
pub(crate) fn to_minute(dt: NaiveDateTime) -> NaiveDateTime {
    let t = dt.time();
    let minute = NaiveTime::from_hms_opt(t.hour(), t.minute(), 0).unwrap_or(t);
    dt.date().and_time(minute)
}

// ---- setting keys (docs/spec.md §3) ----

const SETTING_CLASS_REMINDER: &str = "class_reminder_minutes";
const SETTING_UPCOMING_WINDOW: &str = "upcoming_window_days";
const SETTING_NOTIFICATIONS: &str = "notifications_enabled";
/// §5 Settings theme: appearance choice, stored as `system`, `light`, or `dark`.
pub(crate) const SETTING_THEME: &str = "theme";
/// §3, R29: whether to check for an update once per launch, stored as `1` or `0`.
pub(crate) const SETTING_CHECK_UPDATES: &str = "check_updates_on_start";

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

/// §5 Settings theme: stored text of the appearance choice (exact lowercase).
impl ThemeMode {
    pub(crate) fn as_db(self) -> &'static str {
        match self {
            ThemeMode::System => "system",
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        }
    }

    pub(crate) fn from_db(s: &str) -> Option<Self> {
        match s {
            "system" => Some(ThemeMode::System),
            "light" => Some(ThemeMode::Light),
            "dark" => Some(ThemeMode::Dark),
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

    #[allow(
        dead_code,
        reason = "reminder_log kinds are only written; the codec round-trip test reads them back"
    )]
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
    fn to_minute_drops_seconds() {
        let day = NaiveDate::from_ymd_opt(2026, 1, 5).unwrap();
        let with_secs = day.and_hms_milli_opt(10, 0, 30, 500).unwrap();
        assert_eq!(to_minute(with_secs), day.and_hms_opt(10, 0, 0).unwrap());
        let exact = day.and_hms_opt(23, 59, 0).unwrap();
        assert_eq!(to_minute(exact), exact);
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
