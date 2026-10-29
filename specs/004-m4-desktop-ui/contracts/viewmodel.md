# Contract: `crates/app/src/viewmodel.rs`

Pure functions only: no Slint types, no clock, no database, no file access. Inputs are core types
from `planner_core` and `chrono`, or `&str` form text. Every public item below has unit tests in
`crates/app/src/viewmodel/tests.rs` (declared in `viewmodel.rs` as `#[cfg(test)] mod tests;`).
Doc comments cite rule IDs. Test names are descriptive (e.g. `pct_none_shows_em_dash`); the `t_`
prefix is not required in the app crate.

The tester first adds these signatures with `todo!()` bodies (types complete), then writes the
tests, which must compile and fail. The builder replaces the bodies. Signatures may only change by
updating this contract first.

Conventions: the em dash is `—` (U+2014); ranges use the en dash `–` (U+2013) with no spaces for
times (`09:00–10:30`) and with spaces for date ranges (`October 26 – November 1, 2026`).

Date format change (human decision 2026-10-29, spec.md "Change 2026-10-29"): dates shown as text
use the long form ("October 26, 2026"); date inputs and pre-filled date fields use `DD-MM-YYYY`.
ISO `YYYY-MM-DD` stays only in storage, JSON backup, CSV, and file names (`pre_restore_file_name`,
`backup_file_name`) and the database path. Every example below that shows a date for display uses
the long form; examples for form input use `DD-MM-YYYY`.

## Status and percentages

| Signature | Behavior (examples are test cases) |
| --- | --- |
| `pub enum ChipKind { Attended, Absent, Cancelled, Pending }` | Picks chip colors; derive `Debug, Clone, Copy, PartialEq, Eq` |
| `pub fn chip_kind(status: Status) -> ChipKind` | One-to-one. A rescheduled original has status `Cancelled` → `Cancelled` |
| `pub const BELOW_TARGET_LABEL: &str` | `"Below target"` (R14, DESIGN.md) |
| `pub fn format_pct(pct: Option<f64>) -> String` | R13: `Some(50.0)`→`"50.0%"`, `Some(66.666…)`→`"66.7%"`, `Some(100.0)`→`"100.0%"`, `Some(0.0)`→`"0.0%"`, `None`→`"—"` (never `"0%"` or `"NaN"`) |
| `pub fn format_target(target: Option<i64>) -> String` | `Some(75)`→`"75%"`, `None`→`"—"` |
| `pub fn or_dash(value: Option<&str>) -> String` | `None` or blank after trim → `"—"`, else the trimmed text |
| `pub fn can_mark_attendance(view: &SessionView) -> bool` | R12: equals `view.is_past`; Cancelled and Pending actions are always enabled (no function needed) |

## Dates and times

| Signature | Behavior |
| --- | --- |
| `pub fn format_date(d: NaiveDate) -> String` | Long display form: `"October 29, 2026"`, `"January 5, 2026"` (day not zero-padded) |
| `pub fn format_date_input(d: NaiveDate) -> String` | Zero-padded `DD-MM-YYYY` for pre-filling fields: `"26-10-2026"`, `"05-01-2026"` |
| `pub fn format_time(t: NaiveTime) -> String` | `"09:05"` (seconds dropped) |
| `pub fn format_time_range(start: NaiveTime, end: NaiveTime) -> String` | `"09:00–10:30"` |
| `pub fn format_datetime(dt: NaiveDateTime) -> String` | `"October 30, 2026, 23:59"` (long date, comma, 24-hour time) |
| `pub fn weekday_name(w: Weekday) -> &'static str` | `"Monday"` … `"Sunday"` |
| `pub fn weekday_from_index(i: i32) -> Option<Weekday>` | Combo index `0`→Monday … `6`→Sunday; anything else → `None` |
| `pub fn weekday_index(w: Weekday) -> i32` | Inverse of the above |
| `pub fn parse_date(text: &str) -> Result<NaiveDate, String>` | `DD-MM-YYYY`: `-` separator, 1 or 2 digit day and month, 4-digit year; trims. `"29-10-2026"` ok; `"5-1-2026"` and `"05-01-2026"` → 2026-01-05. Empty, `"2026-10-29"`, `"29/10/2026"`, `"30-02-2026"`, `"29-10-26"`, `"abc"` → `Err("Use DD-MM-YYYY, e.g. 29-10-2026")` |
| `pub fn parse_optional_date(text: &str) -> Result<Option<NaiveDate>, String>` | Empty/blank → `Ok(None)`; otherwise as `parse_date` |
| `pub fn parse_time(text: &str) -> Result<NaiveTime, String>` | Trims. `"09:30"`, `"00:00"`, `"23:59"` ok. Empty, `"9:30"`, `"24:00"`, `"12:60"`, `"0930"` → `Err("Use HH:MM, e.g. 09:30")` |

## Text and numbers

| Signature | Behavior |
| --- | --- |
| `pub fn optional_text(text: &str) -> Option<String>` | Trimmed; empty → `None` (optional fields are stored as absent) |
| `pub fn parse_target(text: &str) -> Result<Option<i64>, String>` | Empty → `Ok(None)`; `"75"` → `Ok(Some(75))`; `"-5"` → `Ok(Some(-5))` and `"150"` → `Ok(Some(150))` (range is core R2); `"7.5"`, `"abc"` → `Err("Enter a whole number")` |
| `pub fn parse_optional_minutes(text: &str) -> Result<Option<u32>, String>` | Empty → `Ok(None)`; `"30"` → `Ok(Some(30))`; `"-1"`, `"x"` → `Err("Enter a whole number of minutes")` |
| `pub fn parse_count(text: &str) -> Result<u32, String>` | Required non-negative whole number; empty → `Err("Required")`; invalid → `Err("Enter a whole number")` |

## Forms

`pub struct FormErrors` holds `(field, message)` pairs. Methods: `pub fn get(&self, field: &str) ->
String` (`""` when the field has no error) and `pub fn is_empty(&self) -> bool`. Field keys are the
snake_case names below. Every parse function checks all fields and reports every error at once.
Rules owned by core (R2: empty course code/name/title, target range, slot end ≤ start, term end <
start, duplicate code) are **not** checked here; core's message is shown as the form error.
Exception: the holiday range `end < start` check is the view model's (human decision, §6), since
holidays have no core range call.

| Signature | Fields checked |
| --- | --- |
| `pub fn parse_term_form(id: Option<i64>, name: &str, start: &str, end: &str) -> Result<TermInput, FormErrors>` | `start`, `end` (date format). Name is passed through trimmed |
| `pub fn parse_course_form(code: &str, name: &str, instructor: &str, room: &str, notes: &str, target: &str) -> Result<CourseInput, FormErrors>` | `target`. Code and name trimmed; optional fields via `optional_text` |
| `pub fn parse_slot_form(course_id: Option<i64>, weekday: i32, start: &str, end: &str, room: &str, valid_from: &str, valid_to: &str) -> Result<SlotInput, FormErrors>` | `course` (`"Choose a course"`), `weekday` (`"Choose a weekday"`), `start`, `end`, `valid_from`, `valid_to` (optional) |
| `pub fn parse_makeup_form(course_id: i64, date: &str, start: &str, end: &str, room: &str, note: &str) -> Result<MakeupInput, FormErrors>` | `date`, `start`, `end`. Used by reschedule (R9) and "Add makeup class" (R10); note via `optional_text` |
| `pub fn parse_assessment_form(course_id: Option<i64>, kind: i32, title: &str, due_date: &str, due_time: &str, notes: &str, remind: &str) -> Result<AssessmentInput, FormErrors>` | `course`, `kind` (`"Choose a kind"`), `due_date`, `due_time`, `remind`. `due_at` = date + time |
| `pub fn parse_holiday_form(start: &str, end: &str, label: &str) -> Result<Vec<Holiday>, FormErrors>` | `start`, `end` (optional; `end < start` → `"End date must be on or after the start date"`). Returns `expand_holiday_range` |
| `pub fn parse_settings_form(reminder_minutes: &str, window_days: &str, notifications: bool) -> Result<Settings, FormErrors>` | `reminder`, `window` via `parse_count` |

## Holidays (human decision §6: single date or inclusive range)

| Signature | Behavior |
| --- | --- |
| `pub fn expand_holiday_range(start: NaiveDate, end: Option<NaiveDate>, label: Option<String>) -> Result<Vec<Holiday>, String>` | `end = None` or `end == start` → one holiday. `2026-12-21..=2026-12-25` → 5 holidays, each with the same label, ascending. `end < start` → `Err("End date must be on or after the start date")` |
| `pub fn merge_holidays(existing: &[Holiday], added: &[Holiday]) -> Vec<Holiday>` | Exactly one holiday per date, sorted by date; for a date in both, the added one (range label) wins. Result is passed to `set_holidays` |
| `pub fn remove_holiday(existing: &[Holiday], date: NaiveDate) -> Vec<Holiday>` | Removes only that date; others unchanged and sorted |
| `pub struct HolidayRow { pub date: String, pub label: String }` + `pub fn holiday_row(h: &Holiday) -> HolidayRow` | `date` via `format_date` (`"December 25, 2026"`). Label `None` → `"—"` |

## Rows

| Signature | Behavior |
| --- | --- |
| `pub struct SessionRow { pub id: i64, pub date: String, pub weekday: String, pub time: String, pub course: String, pub room: String, pub status_label: String, pub chip: ChipKind, pub can_mark: bool, pub note: String }` + `pub fn session_row(view: &SessionView) -> SessionRow` | `course` = `"CSE101 · Intro to Programming"`; `room` via `or_dash`; `date` = `format_date` (`"October 26, 2026"`); `status_label` = `display_status_text(display_status)` (`"Rescheduled → January 15, 2026"`); `chip` from status; `can_mark` = R12; `note` = stored note or `""` |
| `pub struct SummaryView { pub attended: String, pub absent: String, pub cancelled: String, pub pending: String, pub pct: String, pub target: String, pub below_target: bool }` + `pub fn summary_view(s: &AttendanceSummary) -> SummaryView` | R13–R15: counts as decimal text; `pct` via `format_pct`; `target` via `format_target`; `below_target` copied |
| `pub fn banner_line(s: &AttendanceSummary) -> String` | R14: `"CSE101 — 66.7% (target 75%)"` (`"{code} — {pct} (target {target})"`) |
| `pub struct CourseRow { pub id: i64, pub code: String, pub name: String, pub instructor: String, pub room: String, pub target: String, pub archived: bool }` + `pub fn course_row(c: &Course) -> CourseRow` | Optional fields → `"—"`; target via `format_target` |
| `pub fn course_option_label(c: &Course) -> String` | `"CSE101 · Intro to Programming"` (combo boxes) |
| `pub struct SlotRow { pub id: i64, pub weekday: String, pub time: String, pub course: String, pub room: String, pub valid: String }` + `pub fn slot_row(slot: &Slot, courses: &[Course]) -> SlotRow` | `course` = matching code, `"—"` if not found; `valid` = `"From September 1, 2026 to December 20, 2026"` or `"From September 1, 2026 until term end"` |
| `pub struct AssessmentRow { pub id: i64, pub course_code: String, pub kind: String, pub title: String, pub due: String, pub completed: bool, pub notes: String }` + `pub fn assessment_row(a: &Assessment) -> AssessmentRow` | `due` via `format_datetime` (`"October 30, 2026, 23:59"`); `notes` or `""` |
| `pub fn assessment_kind_label(k: AssessmentKind) -> &'static str` | `"Quiz"`, `"Exam"`, `"Assignment"`, `"Other"` |
| `pub fn assessment_kind_from_index(i: i32) -> Option<AssessmentKind>` / `pub fn assessment_kind_index(k: AssessmentKind) -> i32` | 0 Quiz, 1 Exam, 2 Assignment, 3 Other |

## Assessment filters (R18, R19)

| Signature | Behavior |
| --- | --- |
| `pub enum FilterChoice { Upcoming, Overdue, Completed, All }` | Derive `Debug, Clone, Copy, PartialEq, Eq` |
| `pub fn filter_from_index(i: i32) -> FilterChoice` | 0 Upcoming, 1 Overdue, 2 Completed, 3 All; other → Upcoming |
| `pub fn assessment_filter(choice: FilterChoice, now: NaiveDateTime) -> AssessmentFilter` | Upcoming/Overdue carry `now` |
| `pub fn filter_title(choice: FilterChoice) -> &'static str` | "Upcoming", "Overdue", "Completed", "All assessments" (FR-005, T025) |
| `pub fn filter_empty_text(choice: FilterChoice) -> &'static str` | "No upcoming assessments.", "No overdue assessments.", "No completed assessments.", "No assessments yet." (FR-005, T025) |

## Week view

| Signature | Behavior |
| --- | --- |
| `pub fn week_start(date: NaiveDate) -> NaiveDate` | Monday on or before `date` (Monday → itself; Sunday 2026-11-01 → 2026-10-26) |
| `pub fn week_end(week_start: NaiveDate) -> NaiveDate` | `week_start + 6 days` (Sunday) |
| `pub fn shift_week(week_start: NaiveDate, weeks: i64) -> NaiveDate` | `±7 × weeks` days; works across month and year ends |
| `pub fn week_title(week_start: NaiveDate) -> String` | `"October 26 – November 1, 2026"`; year shown once at the end. Across years both years: `"December 28, 2026 – January 3, 2027"`. Within one month the short form: `"October 5 – 11, 2026"` (human decision 2026-10-29: short within a month, month named again when it changes) |
| `pub struct DayGroup { pub title: String, pub rows: Vec<SessionRow> }` + `pub fn group_week(views: &[SessionView]) -> Vec<DayGroup>` | One group per distinct date, in date order, keeping input order inside a day; title `"Monday, October 26, 2026"`; empty input → empty vec (page shows its empty state) |

## Dashboard (R14, R16–R20)

| Signature | Behavior |
| --- | --- |
| `pub struct DashboardView { pub today: Vec<SessionRow>, pub next_class: Option<SessionRow>, pub upcoming: Vec<AssessmentRow>, pub overdue: Vec<AssessmentRow>, pub awaiting: Vec<SessionRow>, pub banner: Vec<String> }` + `pub fn dashboard_view(d: &Dashboard) -> DashboardView` | Order kept from core; `banner` = `banner_line` per `below_target` entry (empty → banner hidden) |

## Errors, files, timer

| Signature | Behavior |
| --- | --- |
| `pub fn display_status_text(raw: &str) -> String` | Rewrites a trailing ISO date in core's `display_status`: `"Rescheduled → 2026-01-13"` → `"Rescheduled → January 13, 2026"`. Labels without a trailing ISO date (`"Pending"`, `"Attended"`) are returned unchanged |
| `pub fn error_text(e: &planner_core::Error) -> String` | `Validation("course code must not be empty")` → `"Course code must not be empty"`; other variants → their `Display` text (e.g. `SchemaTooNew` names both versions, R25) |
| `pub fn pre_restore_file_name(now: NaiveDateTime) -> String` | R22: `"pre-restore-20261029-1430.sqlite"` (seconds ignored) |
| `pub fn free_pre_restore_name(now: NaiveDateTime, existing: &[String]) -> String` | R22, human decision Q2 = A: the base name if not in `existing`; otherwise the first free of `"pre-restore-20261029-1430-2.sqlite"`, `"…-3.sqlite"`, …; unrelated names in `existing` are ignored |
| `pub fn backup_file_name(now: NaiveDateTime) -> String` | `"academic-planner-backup-20261029-1430.json"` |
| `pub fn date_rolled_over(last_sync: NaiveDate, now: NaiveDateTime) -> bool` | R6: `now.date() != last_sync` (23:59 same day → false; 00:00 next day → true) |
| `pub fn db_path_text(path: &Path) -> String` | R24: the path's display text |
| `pub fn restored_message(copy: &Path) -> String` | `"Backup restored. Previous data saved to {path}"` |

## Settings page messages (added by T019 / T022)

| Signature | Behavior |
| --- | --- |
| `pub fn backup_saved_message(path: &Path) -> String` | R21: `"Backup saved to {path}"` |
| `pub fn csv_exported_message(folder: &Path) -> String` | R23: `"Exported attendance.csv and assessments.csv to {folder}"` |
| `pub fn file_error_message(path: &Path, cause: &str) -> String` | `"Could not access {path}: {cause}"` |
| `pub fn pre_restore_failed_message(cause: &str) -> String` | R22: `"Restore not started: could not copy the current data ({cause})"` |
