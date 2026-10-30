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

/// R70: one inactive term with its course count, overall attendance, and GPA.
#[derive(Debug, Clone, PartialEq)]
pub struct PastTermSummary {
    pub term: Term,
    pub courses: u32,
    pub attendance_pct: Option<f64>,
    pub gpa: Option<f64>,
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
    /// R33: credits (may be fractional); `None` = not graded. Must be > 0.
    pub credits: Option<f64>,
    /// R33: grading rule set; `None` = not graded.
    pub rule_set_id: Option<i64>,
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
    /// R33: credits (may be fractional); `None` = not graded. Must be > 0.
    pub credits: Option<f64>,
    /// R33: grading rule set; `None` = not graded.
    pub rule_set_id: Option<i64>,
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
    /// R36: the rule component this assessment counts toward; `None` = not counted in marks.
    pub component_id: Option<i64>,
    /// R36: score earned, must be >= 0 (and <= the component's item max); `None` = not scored.
    pub score: Option<f64>,
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
    /// R36: the rule component this assessment counts toward; `None` = not counted in marks.
    pub component_id: Option<i64>,
    /// R36: score earned, must be >= 0 (and <= the component's item max); `None` = not scored.
    pub score: Option<f64>,
}

/// Appearance choice (§3 `theme`, §5 `Settings`; human decision 2026-10-29).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    /// Follow the operating system's appearance.
    #[default]
    System,
    Light,
    Dark,
}

/// User settings (`setting` table).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub class_reminder_minutes: u32,
    pub upcoming_window_days: u32,
    pub notifications_enabled: bool,
    pub theme: ThemeMode,
    /// §3, R29: check for an update once per launch. Default on.
    pub check_updates_on_start: bool,
}

impl Default for Settings {
    /// Defaults from docs/spec.md §3.
    fn default() -> Self {
        Settings {
            class_reminder_minutes: 15,
            upcoming_window_days: 7,
            notifications_enabled: true,
            theme: ThemeMode::System,
            check_updates_on_start: true,
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
    /// R2: code and name are required; the attendance target is 0–100. R33: present credits
    /// are positive. Duplicate codes are checked against the database in `db.rs`.
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
        // R33: credits, when present, must be positive. The rule set is checked in `db.rs`.
        if let Some(credits) = self.credits {
            if credits.is_nan() || credits <= 0.0 {
                return Err(Error::Validation(format!(
                    "credits {credits} must be greater than 0"
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
    /// R2: the title is required. R36: a present score is not negative.
    pub(crate) fn validate(&self) -> Result<()> {
        require_text(&self.title, "assessment title")?;
        // R36: a score, when present, must be 0 or more. The item maximum is checked with
        // the marks logic.
        if let Some(score) = self.score {
            if score.is_nan() || score < 0.0 {
                return Err(Error::Validation(format!(
                    "score {score} must be 0 or more"
                )));
            }
        }
        Ok(())
    }
}

// ---- M9: credits, rule sets, marks, GPA (§13.3) ----

/// R34: kind of a rule component (`rule_component.kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComponentKind {
    /// R35: earned from the course's attendance percentage through bands.
    Attendance,
    /// R36: earned from the scores of linked assessments.
    Items,
}

/// R35: one attendance band: at or above `min_pct` attendance earns `credit_pct` of the marks.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AttendanceBand {
    pub min_pct: f64,
    pub credit_pct: f64,
}

/// R34: a component of a rule set (`rule_component` row with its bands).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleComponent {
    pub id: i64,
    pub rule_set_id: i64,
    pub position: i64,
    pub name: String,
    pub kind: ComponentKind,
    /// R33: component marks = `weight_pct` / 100 x the course total.
    pub weight_pct: f64,
    /// R36: items only, number of items held.
    pub item_count: Option<u32>,
    /// R36: items only, best K items counted.
    pub best_count: Option<u32>,
    /// R36: items only, marks each item is out of; `None` = the component's marks for the course.
    pub item_max: Option<f64>,
    /// R34: the final component (exactly one per rule set, and it is `Items`).
    pub is_final: bool,
    /// R35: attendance only, sorted by `min_pct` descending.
    pub bands: Vec<AttendanceBand>,
}

/// R34: a rule set with its components ordered by position.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RuleSet {
    pub id: i64,
    pub name: String,
    pub components: Vec<RuleComponent>,
}

/// R34: input for one component of `save_rule_set`.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleComponentInput {
    /// R34: `Some` = update in place; `None` = new component.
    pub id: Option<i64>,
    pub name: String,
    pub kind: ComponentKind,
    pub weight_pct: f64,
    pub item_count: Option<u32>,
    pub best_count: Option<u32>,
    pub item_max: Option<f64>,
    pub is_final: bool,
    pub bands: Vec<AttendanceBand>,
}

/// R34: input for `save_rule_set` (`id: None` creates, `Some` updates).
#[derive(Debug, Clone, PartialEq)]
pub struct RuleSetInput {
    pub id: Option<i64>,
    pub name: String,
    pub components: Vec<RuleComponentInput>,
}

/// R39: one row of the grade scale.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GradeRow {
    pub letter: String,
    pub min_pct: f64,
    pub points: f64,
}

/// R35/R36: marks of one component of a graded course.
#[derive(Debug, Clone, PartialEq)]
pub struct ComponentMarks {
    pub component_id: i64,
    pub name: String,
    pub kind: ComponentKind,
    pub is_final: bool,
    /// R33: marks possible, `weight_pct` / 100 x total.
    pub marks: f64,
    /// R35/R36: marks earned.
    pub earned: f64,
    /// R35/R36: assessed fraction, 0 to 1.
    pub assessed_fraction: f64,
    /// R36: items, the effective item max; attendance: `None`.
    pub item_max: Option<f64>,
    /// R36: items, linked assessments with a score.
    pub scores_entered: u32,
    /// R38: items, `item_count` minus entered (at least 0); attendance: 0.
    pub missing_items: u32,
}

/// R38: the score needed in the final for one grade row.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Needed {
    /// Already secured without the final.
    Secured,
    /// Above the final item max.
    NotReachable,
    /// Score needed, rounded up to 1 decimal.
    Score(f64),
}

/// R38: one row of the needed-in-final table.
#[derive(Debug, Clone, PartialEq)]
pub struct NeededRow {
    pub letter: String,
    pub min_pct: f64,
    pub needed: Needed,
}

/// R38: missing items counted as 0 in the needed-in-final table.
#[derive(Debug, Clone, PartialEq)]
pub struct Assumption {
    pub component_name: String,
    pub missing_items: u32,
}

/// R38: what the final still needs, or the actual grade once the final has a score.
#[derive(Debug, Clone, PartialEq)]
pub enum FinalOutlook {
    Needed {
        rows: Vec<NeededRow>,
        final_item_max: f64,
        assumptions: Vec<Assumption>,
    },
    Actual {
        letter: Option<String>,
    },
}

/// R37: marks summary of one course.
#[derive(Debug, Clone, PartialEq)]
pub struct CourseMarks {
    pub course_id: i64,
    /// R33: has credits and a rule set.
    pub graded: bool,
    /// R33: 100 x credits; 0 when not graded.
    pub total: f64,
    pub components: Vec<ComponentMarks>,
    /// R37: sum of earned marks.
    pub secured: f64,
    /// R37: sum of component marks x assessed fraction.
    pub assessed: f64,
    /// R37: secured / assessed x 100, 1 decimal; `None` if nothing assessed.
    pub projected_pct: Option<f64>,
    /// R37: grade-scale letter for the projected pct.
    pub letter: Option<String>,
    /// R40: every component fully assessed.
    pub fully_assessed: bool,
    /// R38: `None` when not graded.
    pub outlook: Option<FinalOutlook>,
}

/// R40: whether the GPA is still projected or final.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpaStatus {
    Projected,
    Final,
}

/// R40: term GPA summary.
#[derive(Debug, Clone, PartialEq)]
pub struct GpaSummary {
    /// R40: 2 decimals; `None` if no course qualifies.
    pub gpa: Option<f64>,
    pub status: GpaStatus,
    /// R40: graded courses with a projected pct.
    pub included: u32,
    /// R40: graded courses left out for lack of marks.
    pub left_out: u32,
}
