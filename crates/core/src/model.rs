//! Domain types for the core API (docs/spec.md §3, §5). Dates and times are chrono naive
//! local types; weekdays use ISO numbering (Monday = 1) when stored.

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// R11: attendance status of a session (`session.status`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Pending,
    Attended,
    Absent,
    Cancelled,
}

/// Whether a session comes from a slot (`regular`) or was added by hand (`makeup`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionKind {
    Regular,
    Makeup,
}

/// `assessment.kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssessmentKind {
    Quiz,
    Exam,
    Assignment,
    Other,
}

/// `reminder_log.kind` (M5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReminderKind {
    Session,
    Assessment,
}

/// Which assessments `list_assessments` returns. Time-dependent filters carry `now`
/// because core never reads the clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssessmentFilter {
    /// R18: incomplete, `now <= due_at <= now + upcoming_window_days`.
    Upcoming { now: NaiveDateTime },
    /// R19: incomplete, `due_at < now`.
    Overdue { now: NaiveDateTime },
    /// Completed assessments.
    Completed,
    /// Every assessment.
    All,
}

/// Input for `save_term`: `id = None` creates a term, `Some` updates it.
#[derive(Debug, Clone, PartialEq)]
pub struct TermInput {
    pub id: Option<i64>,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

/// A teaching period (`term` table) with its holidays.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Term {
    pub id: i64,
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    /// R1: at most one term is active.
    pub is_active: bool,
    /// Holidays of this term, sorted by date.
    pub holidays: Vec<Holiday>,
}

/// A day without classes (`holiday` table).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Holiday {
    pub date: NaiveDate,
    pub label: Option<String>,
}

/// Input for `create_course` / `update_course`.
#[derive(Debug, Clone, PartialEq)]
pub struct CourseInput {
    pub code: String,
    pub name: String,
    pub instructor: Option<String>,
    pub room: Option<String>,
    pub notes: Option<String>,
    /// Attendance target in percent, 0–100; `None` = no target.
    pub min_attendance_pct: Option<i64>,
}

/// A course in a term (`course` table).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Course {
    pub id: i64,
    pub term_id: i64,
    pub code: String,
    pub name: String,
    pub instructor: Option<String>,
    pub room: Option<String>,
    pub notes: Option<String>,
    pub min_attendance_pct: Option<i64>,
    /// R3: archived courses are hidden from the dashboard, timetable, and default lists.
    pub archived: bool,
}

/// Input for `add_slot` / `change_slot`.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotInput {
    pub course_id: i64,
    pub weekday: Weekday,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    /// `None` = use the course room.
    pub room: Option<String>,
    /// First date the slot applies (inclusive).
    pub valid_from: NaiveDate,
    /// Last date the slot applies (inclusive); `None` = until term end.
    pub valid_to: Option<NaiveDate>,
}

/// A recurring weekly timetable rule (`slot` table).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Slot {
    pub id: i64,
    pub course_id: i64,
    pub weekday: Weekday,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub room: Option<String>,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
}

/// One dated class occurrence (`session` table). Sessions are snapshots: their date, time,
/// and room do not follow later slot changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub id: i64,
    pub course_id: i64,
    /// `None` for makeup sessions.
    pub slot_id: Option<i64>,
    pub kind: SessionKind,
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub room: Option<String>,
    pub status: Status,
    /// Set on a makeup that reschedules an original session.
    pub replaces_session_id: Option<i64>,
    pub note: Option<String>,
}

/// A session with the details lists and the dashboard show.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionView {
    pub session: Session,
    pub course_code: String,
    pub course_name: String,
    /// "Pending" / "Attended" / "Absent" / "Cancelled" / "Rescheduled → 2026-01-15".
    pub display_status: String,
    /// `now >= session start`: attendance may be marked (R12).
    pub is_past: bool,
}

/// Input for `reschedule` / `add_makeup`.
#[derive(Debug, Clone, PartialEq)]
pub struct MakeupInput {
    pub course_id: i64,
    pub date: NaiveDate,
    pub start_time: NaiveTime,
    pub end_time: NaiveTime,
    pub room: Option<String>,
    pub note: Option<String>,
}

/// Result of `sync_sessions`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SyncReport {
    pub created: usize,
    pub deleted: usize,
}

/// R15: per-course attendance counts and percentage.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AttendanceSummary {
    pub course_id: i64,
    pub course_code: String,
    pub course_name: String,
    pub attended: u32,
    pub absent: u32,
    pub cancelled: u32,
    pub pending: u32,
    /// R13: `None` when nothing is attended or absent yet.
    pub pct: Option<f64>,
    pub target: Option<i64>,
    /// R14.
    pub below_target: bool,
}

/// R16–R20: everything the dashboard page shows.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Dashboard {
    /// R16.
    pub today: Vec<SessionView>,
    /// R17.
    pub next_class: Option<SessionView>,
    /// R18.
    pub upcoming: Vec<Assessment>,
    /// R19.
    pub overdue: Vec<Assessment>,
    /// R20.
    pub awaiting: Vec<SessionView>,
    /// R14: courses below their attendance target, for the banner.
    pub below_target: Vec<AttendanceSummary>,
}

/// Input for `create_assessment` / `update_assessment`.
#[derive(Debug, Clone, PartialEq)]
pub struct AssessmentInput {
    pub course_id: i64,
    pub kind: AssessmentKind,
    pub title: String,
    pub due_at: NaiveDateTime,
    pub notes: Option<String>,
    /// `None` = no reminder.
    pub remind_minutes_before: Option<u32>,
}

/// A quiz, exam, assignment, or other deliverable (`assessment` table).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Assessment {
    pub id: i64,
    pub course_id: i64,
    /// Code of the course, joined for lists.
    pub course_code: String,
    pub kind: AssessmentKind,
    pub title: String,
    pub due_at: NaiveDateTime,
    pub notes: Option<String>,
    pub completed: bool,
    pub remind_minutes_before: Option<u32>,
}

/// User settings (`setting` table).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub class_reminder_minutes: u32,
    pub upcoming_window_days: u32,
    pub notifications_enabled: bool,
}

impl Default for Settings {
    /// Defaults from docs/spec.md §3.
    fn default() -> Self {
        Settings {
            class_reminder_minutes: 15,
            upcoming_window_days: 7,
            notifications_enabled: true,
        }
    }
}

/// A notification that is due (M5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Reminder {
    pub kind: ReminderKind,
    pub ref_id: i64,
    pub fire_at: NaiveDateTime,
    pub title: String,
}

// ---- R2 input validation: every check runs before anything is written ----

fn require_text(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(Error::Validation(format!("{field} must not be empty")));
    }
    Ok(())
}

impl TermInput {
    /// R2: the term must not end before it starts.
    pub(crate) fn validate(&self) -> Result<()> {
        if self.end_date < self.start_date {
            return Err(Error::Validation(format!(
                "term end date {} is before its start date {}",
                self.end_date, self.start_date
            )));
        }
        Ok(())
    }
}

impl CourseInput {
    /// R2: code and name are required; the attendance target is 0–100. Duplicate codes
    /// are checked against the database in `db.rs`.
    pub(crate) fn validate(&self) -> Result<()> {
        require_text(&self.code, "course code")?;
        require_text(&self.name, "course name")?;
        if let Some(pct) = self.min_attendance_pct {
            if !(0..=100).contains(&pct) {
                return Err(Error::Validation(format!(
                    "attendance target {pct}% is outside 0–100"
                )));
            }
        }
        Ok(())
    }
}

impl SlotInput {
    /// R2: a slot must end after it starts.
    pub(crate) fn validate(&self) -> Result<()> {
        if self.end_time <= self.start_time {
            return Err(Error::Validation(format!(
                "slot end time {} is not after its start time {}",
                self.end_time.format("%H:%M"),
                self.start_time.format("%H:%M")
            )));
        }
        Ok(())
    }
}

impl AssessmentInput {
    /// R2: the title is required.
    pub(crate) fn validate(&self) -> Result<()> {
        require_text(&self.title, "assessment title")
    }
}
