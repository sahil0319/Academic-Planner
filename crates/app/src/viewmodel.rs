//! View model: pure formatting, parsing, and validation for the desktop UI (docs/spec.md §2).
//!
//! No Slint types, no clock, no database, no file access. Every function here is unit tested
//! in `viewmodel/tests.rs`. Contract: `specs/004-m4-desktop-ui/contracts/viewmodel.md`.
#![allow(dead_code, unused_variables)] // removed in T026 once pages use every item (unused_variables: todo!() stubs, removed in T008)

use std::path::Path;

use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use planner_core::{
    Assessment, AssessmentFilter, AssessmentInput, AssessmentKind, AttendanceSummary, Course,
    CourseInput, Dashboard, Holiday, MakeupInput, SessionView, Settings, Slot, SlotInput, Status,
    TermInput,
};

#[cfg(test)]
mod tests;

// ---- Status and percentages ----

/// Picks chip colors for a session status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipKind {
    Attended,
    Absent,
    Cancelled,
    Pending,
}

/// Label shown when a course is below its attendance target.
/// R14: below-target indicator.
pub const BELOW_TARGET_LABEL: &str = "Below target";

/// R11: display label of a session status.
pub fn status_label(status: Status) -> &'static str {
    todo!()
}

/// R11: chip kind of a status; a rescheduled original (status `Cancelled`) shows as `Cancelled`.
pub fn chip_kind(status: Status) -> ChipKind {
    todo!()
}

/// R13: percentage with one decimal; `None` (nothing attended or absent) shows an em dash.
pub fn format_pct(pct: Option<f64>) -> String {
    todo!()
}

/// R14: attendance target as `75%`, or an em dash when there is none.
pub fn format_target(target: Option<i64>) -> String {
    todo!()
}

/// Trimmed text, or an em dash when absent or blank.
pub fn or_dash(value: Option<&str>) -> String {
    todo!()
}

/// R12: attendance can be marked only once the session has started.
pub fn can_mark_attendance(view: &SessionView) -> bool {
    todo!()
}

// ---- Dates and times ----

/// `2026-10-29`.
pub fn format_date(d: NaiveDate) -> String {
    todo!()
}

/// `09:05` (seconds dropped).
pub fn format_time(t: NaiveTime) -> String {
    todo!()
}

/// `09:00–10:30` (en dash, no spaces).
pub fn format_time_range(start: NaiveTime, end: NaiveTime) -> String {
    todo!()
}

/// `2026-10-30 23:59`.
pub fn format_datetime(dt: NaiveDateTime) -> String {
    todo!()
}

/// `Monday` … `Sunday`.
pub fn weekday_name(w: Weekday) -> &'static str {
    todo!()
}

/// Combo index 0 = Monday … 6 = Sunday; anything else is `None`.
pub fn weekday_from_index(i: i32) -> Option<Weekday> {
    todo!()
}

/// Inverse of [`weekday_from_index`].
pub fn weekday_index(w: Weekday) -> i32 {
    todo!()
}

/// Strict `YYYY-MM-DD` after trimming (FR-007).
pub fn parse_date(text: &str) -> Result<NaiveDate, String> {
    todo!()
}

/// Blank is `Ok(None)`; otherwise as [`parse_date`].
pub fn parse_optional_date(text: &str) -> Result<Option<NaiveDate>, String> {
    todo!()
}

/// Strict `HH:MM` after trimming (FR-007).
pub fn parse_time(text: &str) -> Result<NaiveTime, String> {
    todo!()
}

// ---- Text and numbers ----

/// Trimmed text; empty becomes `None` (optional fields are stored as absent).
pub fn optional_text(text: &str) -> Option<String> {
    todo!()
}

/// Optional whole-number target. The 0–100 range is checked by core (R2), not here.
pub fn parse_target(text: &str) -> Result<Option<i64>, String> {
    todo!()
}

/// Optional non-negative whole number of minutes.
pub fn parse_optional_minutes(text: &str) -> Result<Option<u32>, String> {
    todo!()
}

/// Required non-negative whole number.
pub fn parse_count(text: &str) -> Result<u32, String> {
    todo!()
}

// ---- Forms ----

/// Field errors of a form, as `(field, message)` pairs. Field keys are snake_case.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct FormErrors {
    pub errors: Vec<(String, String)>,
}

impl FormErrors {
    /// Message of `field`, or `""` when the field has no error.
    pub fn get(&self, field: &str) -> String {
        todo!()
    }

    /// True when no field has an error.
    pub fn is_empty(&self) -> bool {
        todo!()
    }
}

/// Checks `start` and `end`. R2 checks (end < start) belong to core.
pub fn parse_term_form(
    id: Option<i64>,
    name: &str,
    start: &str,
    end: &str,
) -> Result<TermInput, FormErrors> {
    todo!()
}

/// Checks `target`. R2 checks (empty code or name, range) belong to core.
pub fn parse_course_form(
    code: &str,
    name: &str,
    instructor: &str,
    room: &str,
    notes: &str,
    target: &str,
) -> Result<CourseInput, FormErrors> {
    todo!()
}

/// Checks `course`, `weekday`, `start`, `end`, `valid_from`, `valid_to`. R2: end <= start is core's.
pub fn parse_slot_form(
    course_id: Option<i64>,
    weekday: i32,
    start: &str,
    end: &str,
    room: &str,
    valid_from: &str,
    valid_to: &str,
) -> Result<SlotInput, FormErrors> {
    todo!()
}

/// R9, R10: checks `date`, `start`, `end` for reschedule and "Add makeup class".
pub fn parse_makeup_form(
    course_id: i64,
    date: &str,
    start: &str,
    end: &str,
    room: &str,
    note: &str,
) -> Result<MakeupInput, FormErrors> {
    todo!()
}

/// Checks `course`, `kind`, `due_date`, `due_time`, `remind`. `due_at` is date plus time.
pub fn parse_assessment_form(
    course_id: Option<i64>,
    kind: i32,
    title: &str,
    due_date: &str,
    due_time: &str,
    notes: &str,
    remind: &str,
) -> Result<AssessmentInput, FormErrors> {
    todo!()
}

/// Checks `start` and `end`; `end < start` is an error. Returns the expanded range.
pub fn parse_holiday_form(start: &str, end: &str, label: &str) -> Result<Vec<Holiday>, FormErrors> {
    todo!()
}

/// Checks `reminder` and `window` via [`parse_count`].
pub fn parse_settings_form(
    reminder_minutes: &str,
    window_days: &str,
    notifications: bool,
) -> Result<Settings, FormErrors> {
    todo!()
}

// ---- Holidays (single date or inclusive range) ----

/// One holiday for a single day, or one per day (same label, ascending) for an inclusive range.
pub fn expand_holiday_range(
    start: NaiveDate,
    end: Option<NaiveDate>,
    label: Option<String>,
) -> Result<Vec<Holiday>, String> {
    todo!()
}

/// Exactly one holiday per date, sorted; for a date in both lists the added one wins.
pub fn merge_holidays(existing: &[Holiday], added: &[Holiday]) -> Vec<Holiday> {
    todo!()
}

/// Removes only `date`; the others stay unchanged and sorted.
pub fn remove_holiday(existing: &[Holiday], date: NaiveDate) -> Vec<Holiday> {
    todo!()
}

/// A holiday as display text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolidayRow {
    pub date: String,
    pub label: String,
}

/// Label `None` shows an em dash.
pub fn holiday_row(h: &Holiday) -> HolidayRow {
    todo!()
}

// ---- Rows ----

/// A session as display text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionRow {
    pub id: i64,
    pub date: String,
    pub weekday: String,
    pub time: String,
    pub course: String,
    pub room: String,
    pub status_label: String,
    pub chip: ChipKind,
    pub can_mark: bool,
    pub note: String,
}

/// R11, R12: builds a session row; the status label keeps core's `display_status`.
pub fn session_row(view: &SessionView) -> SessionRow {
    todo!()
}

/// Attendance summary as display text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SummaryView {
    pub attended: String,
    pub absent: String,
    pub cancelled: String,
    pub pending: String,
    pub pct: String,
    pub target: String,
    pub below_target: bool,
}

/// R13–R15: counts, percentage, target, below-target flag.
pub fn summary_view(s: &AttendanceSummary) -> SummaryView {
    todo!()
}

/// R14: banner text, `CSE101 — 66.7% (target 75%)`.
pub fn banner_line(s: &AttendanceSummary) -> String {
    todo!()
}

/// A course as display text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CourseRow {
    pub id: i64,
    pub code: String,
    pub name: String,
    pub instructor: String,
    pub room: String,
    pub target: String,
    pub archived: bool,
}

/// R3: builds a course row; missing optional fields show an em dash.
pub fn course_row(c: &Course) -> CourseRow {
    todo!()
}

/// `CSE101 · Intro to Programming` for combo boxes.
pub fn course_option_label(c: &Course) -> String {
    todo!()
}

/// A timetable slot as display text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotRow {
    pub id: i64,
    pub weekday: String,
    pub time: String,
    pub course: String,
    pub room: String,
    pub valid: String,
}

/// Course is the matching code, or an em dash when not found.
pub fn slot_row(slot: &Slot, courses: &[Course]) -> SlotRow {
    todo!()
}

/// An assessment as display text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssessmentRow {
    pub id: i64,
    pub course_code: String,
    pub kind: String,
    pub title: String,
    pub due: String,
    pub completed: bool,
    pub notes: String,
}

/// Builds an assessment row; `due` uses [`format_datetime`].
pub fn assessment_row(a: &Assessment) -> AssessmentRow {
    todo!()
}

/// `Quiz`, `Exam`, `Assignment`, `Other`.
pub fn assessment_kind_label(k: AssessmentKind) -> &'static str {
    todo!()
}

/// Combo index 0 Quiz, 1 Exam, 2 Assignment, 3 Other; anything else is `None`.
pub fn assessment_kind_from_index(i: i32) -> Option<AssessmentKind> {
    todo!()
}

/// Inverse of [`assessment_kind_from_index`].
pub fn assessment_kind_index(k: AssessmentKind) -> i32 {
    todo!()
}

// ---- Assessment filters ----

/// Filter choice of the assessments page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterChoice {
    Upcoming,
    Overdue,
    Completed,
    All,
}

/// 0 Upcoming, 1 Overdue, 2 Completed, 3 All; anything else is Upcoming.
pub fn filter_from_index(i: i32) -> FilterChoice {
    todo!()
}

/// R18, R19: Upcoming and Overdue carry `now`.
pub fn assessment_filter(choice: FilterChoice, now: NaiveDateTime) -> AssessmentFilter {
    todo!()
}

// ---- Week view ----

/// Monday on or before `date`.
pub fn week_start(date: NaiveDate) -> NaiveDate {
    todo!()
}

/// Sunday of the week starting at `week_start`.
pub fn week_end(week_start: NaiveDate) -> NaiveDate {
    todo!()
}

/// Moves by whole weeks, across month and year ends.
pub fn shift_week(week_start: NaiveDate, weeks: i64) -> NaiveDate {
    todo!()
}

/// `2026-10-26 – 2026-11-01` (en dash with spaces).
pub fn week_title(week_start: NaiveDate) -> String {
    todo!()
}

/// Sessions of one day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayGroup {
    pub title: String,
    pub rows: Vec<SessionRow>,
}

/// One group per distinct date in date order, keeping input order inside a day.
pub fn group_week(views: &[SessionView]) -> Vec<DayGroup> {
    todo!()
}

// ---- Dashboard ----

/// Everything the dashboard shows, as display text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DashboardView {
    pub today: Vec<SessionRow>,
    pub next_class: Option<SessionRow>,
    pub upcoming: Vec<AssessmentRow>,
    pub overdue: Vec<AssessmentRow>,
    pub awaiting: Vec<SessionRow>,
    pub banner: Vec<String>,
}

/// R14, R16–R20: keeps core's order; the banner has one line per below-target course.
pub fn dashboard_view(d: &Dashboard) -> DashboardView {
    todo!()
}

// ---- Errors, files, timer ----

/// R2, R25: user-facing error text; `Validation` messages are capitalized.
pub fn error_text(e: &planner_core::Error) -> String {
    todo!()
}

/// R22: `pre-restore-20261029-1430.sqlite` (seconds ignored).
pub fn pre_restore_file_name(now: NaiveDateTime) -> String {
    todo!()
}

/// R22: the base name if free, else the first free `-2`, `-3`, … suffix; unrelated names ignored.
pub fn free_pre_restore_name(now: NaiveDateTime, existing: &[String]) -> String {
    todo!()
}

/// `academic-planner-backup-20261029-1430.json`.
pub fn backup_file_name(now: NaiveDateTime) -> String {
    todo!()
}

/// R6: true once the calendar date moved past the last sync date.
pub fn date_rolled_over(last_sync: NaiveDate, now: NaiveDateTime) -> bool {
    todo!()
}

/// R24: display text of the database path.
pub fn db_path_text(path: &Path) -> String {
    todo!()
}

/// Message shown after a restore (R22).
pub fn restored_message(copy: &Path) -> String {
    todo!()
}
