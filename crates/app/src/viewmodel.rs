//! View model: pure formatting, parsing, and validation for the desktop UI (docs/spec.md §2).
//!
//! No Slint types, no clock, no database, no file access. Every function here is unit tested
//! in `viewmodel/tests.rs`. Contract: `specs/004-m4-desktop-ui/contracts/viewmodel.md`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use chrono::{Datelike, Days, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Weekday};
use planner_core::{
    Assessment, AssessmentFilter, AssessmentInput, AssessmentKind, Assumption, AttendanceBand,
    AttendanceSummary, ComponentKind, Course, CourseInput, CourseMarks, Dashboard, FinalOutlook,
    GpaStatus, GpaSummary, GradeRow, Holiday, MakeupInput, Needed, RuleComponent,
    RuleComponentInput, RuleSet, SessionView, Settings, Slot, SlotInput, Status, TermInput,
    ThemeMode,
};

use crate::updater::InstallKind;

#[cfg(test)]
mod tests;

pub mod import;

/// Shown for a missing value (R13: never `0%` or `NaN`).
const DASH: &str = "\u{2014}";
/// FR-007: the error states the expected date format.
const DATE_FORMAT_ERR: &str = "Use DD-MM-YYYY, e.g. 29-10-2026";
/// FR-007, R32: the error states the expected time format.
const TIME_FORMAT_ERR: &str =
    "Enter the time as digits, e.g. 930 (hour 1\u{2013}12, minutes 00\u{2013}59)";
const WHOLE_NUMBER_ERR: &str = "Enter a whole number";
const CHOOSE_COURSE_ERR: &str = "Choose a course";
/// Human decision (§6): the holiday range check belongs to the view model.
const HOLIDAY_RANGE_ERR: &str = "End date must be on or after the start date";

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

/// R11: chip kind of a status; a rescheduled original (status `Cancelled`) shows as `Cancelled`.
pub fn chip_kind(status: Status) -> ChipKind {
    match status {
        Status::Pending => ChipKind::Pending,
        Status::Attended => ChipKind::Attended,
        Status::Absent => ChipKind::Absent,
        Status::Cancelled => ChipKind::Cancelled,
    }
}

/// Fixed-point text with `decimals` places that never shows a minus sign for zero.
///
/// A negative zero, or a small negative that rounds to zero at this precision, prints
/// without a sign (`"0.0"`, not `"-0.0"`). Real negatives keep their sign.
fn fixed(value: f64, decimals: usize) -> String {
    let text = format!("{value:.decimals$}");
    match text.strip_prefix('-') {
        Some(unsigned) if unsigned.chars().all(|c| matches!(c, '0' | '.')) => unsigned.to_string(),
        _ => text,
    }
}

/// R13: percentage with one decimal; `None` (nothing attended or absent) shows an em dash.
pub fn format_pct(pct: Option<f64>) -> String {
    match pct {
        Some(p) if p.is_finite() => format!("{}%", fixed(p, 1)),
        _ => DASH.to_string(),
    }
}

/// R14: attendance target as `75%`, or an em dash when there is none.
pub fn format_target(target: Option<i64>) -> String {
    match target {
        Some(t) => format!("{t}%"),
        None => DASH.to_string(),
    }
}

/// Trimmed text, or an em dash when absent or blank.
pub fn or_dash(value: Option<&str>) -> String {
    match value.map(str::trim) {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => DASH.to_string(),
    }
}

/// R12: attendance can be marked only once the session has started.
pub fn can_mark_attendance(view: &SessionView) -> bool {
    view.is_past
}

// ---- Dates and times ----

/// Long display form, `October 29, 2026` (day not zero-padded; §6 date format change).
pub fn format_date(d: NaiveDate) -> String {
    d.format("%B %-d, %Y").to_string()
}

/// `26-10-2026` (zero-padded `DD-MM-YYYY`) for pre-filling date fields (§6 date format change).
pub fn format_date_input(d: NaiveDate) -> String {
    d.format("%d-%m-%Y").to_string()
}

/// Rewrites a trailing ISO date in core's `display_status`
/// (`Rescheduled → 2026-01-13` becomes `Rescheduled → January 13, 2026`); other labels unchanged.
/// T09: core keeps the ISO text; only the app shows the long form.
pub fn display_status_text(raw: &str) -> String {
    const ISO_LEN: usize = 10;
    if raw.len() >= ISO_LEN && raw.is_char_boundary(raw.len() - ISO_LEN) {
        let (head, tail) = raw.split_at(raw.len() - ISO_LEN);
        if let Ok(date) = NaiveDate::parse_from_str(tail, "%Y-%m-%d") {
            return format!("{head}{}", format_date(date));
        }
    }
    raw.to_string()
}

/// R31: `9:05 AM` (12-hour clock, no leading zero, seconds dropped; 00:xx is 12:xx AM and
/// 12:xx is 12:xx PM).
pub fn format_time(t: NaiveTime) -> String {
    let hour = t.hour();
    let hour12 = if hour.is_multiple_of(12) {
        12
    } else {
        hour % 12
    };
    let suffix = if hour < 12 { "AM" } else { "PM" };
    format!("{hour12}:{:02} {suffix}", t.minute())
}

/// R31: `9:00 AM–10:30 PM` (en dash, no spaces).
pub fn format_time_range(start: NaiveTime, end: NaiveTime) -> String {
    format!("{}\u{2013}{}", format_time(start), format_time(end))
}

/// R31: `October 30, 2026, 11:59 PM` (long date, comma, 12-hour time).
pub fn format_datetime(dt: NaiveDateTime) -> String {
    format!("{}, {}", format_date(dt.date()), format_time(dt.time()))
}

/// `Monday` … `Sunday`.
pub fn weekday_name(w: Weekday) -> &'static str {
    match w {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}

/// Combo index 0 = Monday … 6 = Sunday; anything else is `None`.
pub fn weekday_from_index(i: i32) -> Option<Weekday> {
    match i {
        0 => Some(Weekday::Mon),
        1 => Some(Weekday::Tue),
        2 => Some(Weekday::Wed),
        3 => Some(Weekday::Thu),
        4 => Some(Weekday::Fri),
        5 => Some(Weekday::Sat),
        6 => Some(Weekday::Sun),
        _ => None,
    }
}

/// Inverse of [`weekday_from_index`].
pub fn weekday_index(w: Weekday) -> i32 {
    // `num_days_from_monday` is 0..=6, so the cast cannot truncate.
    w.num_days_from_monday() as i32
}

/// Strict `DD-MM-YYYY` after trimming (FR-007, §6 date format change): `-` separator, 1 or 2
/// digit day and month, 4-digit year; the date must exist.
pub fn parse_date(text: &str) -> Result<NaiveDate, String> {
    let err = || DATE_FORMAT_ERR.to_string();
    let parts: Vec<&str> = text.trim().split('-').collect();
    let [day, month, year] = parts.as_slice() else {
        return Err(err());
    };
    let digits =
        |p: &str, lens: &[usize]| lens.contains(&p.len()) && p.bytes().all(|c| c.is_ascii_digit());
    if !(digits(day, &[1, 2]) && digits(month, &[1, 2]) && digits(year, &[4])) {
        return Err(err());
    }
    let day: u32 = day.parse().map_err(|_| err())?;
    let month: u32 = month.parse().map_err(|_| err())?;
    let year: i32 = year.parse().map_err(|_| err())?;
    NaiveDate::from_ymd_opt(year, month, day).ok_or_else(err)
}

/// Blank is `Ok(None)`; otherwise as [`parse_date`].
pub fn parse_optional_date(text: &str) -> Result<Option<NaiveDate>, String> {
    if text.trim().is_empty() {
        Ok(None)
    } else {
        parse_date(text).map(Some)
    }
}

/// R32: one time field as typed: the digits text and the AM/PM toggle (pm == false is AM).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeText<'a> {
    pub digits: &'a str,
    pub pm: bool,
}

/// R32: digits plus AM/PM toggle to a 24-hour time (FR-007).
///
/// Spaces and `:` are ignored; 1-2 digits are an hour, 3-4 digits are hour then two-digit
/// minutes. Hour must be 1-12 and minutes 0-59.
pub fn parse_time(digits: &str, pm: bool) -> Result<NaiveTime, String> {
    let err = || TIME_FORMAT_ERR.to_string();
    let cleaned: String = digits.trim().chars().filter(|c| *c != ':').collect();
    if cleaned.is_empty() || cleaned.len() > 4 || !cleaned.chars().all(|c| c.is_ascii_digit()) {
        return Err(err());
    }
    let (hour_text, minute_text) = if cleaned.len() <= 2 {
        (cleaned.as_str(), "0")
    } else {
        cleaned.split_at(cleaned.len() - 2)
    };
    let hour: u32 = hour_text.parse().map_err(|_| err())?;
    let minute: u32 = minute_text.parse().map_err(|_| err())?;
    if !(1..=12).contains(&hour) || minute > 59 {
        return Err(err());
    }
    let hour24 = if pm { hour % 12 + 12 } else { hour % 12 };
    NaiveTime::from_hms_opt(hour24, minute, 0).ok_or_else(err)
}

/// R32: pre-fill for editing: (`hmm` digits, pm).
pub fn format_time_input(t: NaiveTime) -> (String, bool) {
    let hour = t.hour();
    let hour12 = if hour.is_multiple_of(12) {
        12
    } else {
        hour % 12
    };
    (format!("{hour12}{:02}", t.minute()), hour >= 12)
}

/// R31: rewrites every standalone `HH:MM` token to 12-hour form (a valid time, no digit
/// directly before or after); other text is unchanged. Same idea as the T09
/// `display_status_text` precedent: stored text stays as is, only the display changes.
pub fn times_to_12h(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < bytes.len() {
        if i + 5 <= bytes.len() && text.is_char_boundary(i + 5) {
            let tok = &bytes[i..i + 5];
            let shape = tok[0].is_ascii_digit()
                && tok[1].is_ascii_digit()
                && tok[2] == b':'
                && tok[3].is_ascii_digit()
                && tok[4].is_ascii_digit();
            let digit_before = i > 0 && bytes[i - 1].is_ascii_digit();
            let digit_after = i + 5 < bytes.len() && bytes[i + 5].is_ascii_digit();
            if shape && !digit_before && !digit_after {
                if let Ok(t) = NaiveTime::parse_from_str(&text[i..i + 5], "%H:%M") {
                    out.push_str(&format_time(t));
                    i += 5;
                    continue;
                }
            }
        }
        let ch = text[i..].chars().next().unwrap_or(' ');
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

// ---- Text and numbers ----

/// Trimmed text; empty becomes `None` (optional fields are stored as absent).
pub fn optional_text(text: &str) -> Option<String> {
    let t = text.trim();
    (!t.is_empty()).then(|| t.to_string())
}

/// Optional whole-number target. The 0–100 range is checked by core (R2), not here.
pub fn parse_target(text: &str) -> Result<Option<i64>, String> {
    let t = text.trim();
    if t.is_empty() {
        return Ok(None);
    }
    t.parse::<i64>()
        .map(Some)
        .map_err(|_| WHOLE_NUMBER_ERR.to_string())
}

/// Optional non-negative whole number of minutes.
pub fn parse_optional_minutes(text: &str) -> Result<Option<u32>, String> {
    let t = text.trim();
    if t.is_empty() {
        return Ok(None);
    }
    t.parse::<u32>()
        .map(Some)
        .map_err(|_| "Enter a whole number of minutes".to_string())
}

/// Required non-negative whole number.
pub fn parse_count(text: &str) -> Result<u32, String> {
    let t = text.trim();
    if t.is_empty() {
        return Err("Required".to_string());
    }
    t.parse::<u32>().map_err(|_| WHOLE_NUMBER_ERR.to_string())
}

/// R33, R34, R35, R36, R39: a finite decimal number; the error says what is expected.
fn parse_number(text: &str) -> Result<f64, String> {
    let t = text.trim();
    if t.is_empty() {
        return Err("Required".to_string());
    }
    match t.parse::<f64>() {
        Ok(v) if v.is_finite() => Ok(v),
        _ => Err("Enter a number".to_string()),
    }
}

/// An optional decimal number: empty is `None`.
fn parse_optional_number(text: &str) -> Result<Option<f64>, String> {
    if text.trim().is_empty() {
        Ok(None)
    } else {
        parse_number(text).map(Some)
    }
}

/// An optional whole number: empty is `None`.
fn parse_optional_count(text: &str) -> Result<Option<u32>, String> {
    if text.trim().is_empty() {
        Ok(None)
    } else {
        parse_count(text).map(Some)
    }
}

/// R33: credits are empty (not graded) or a number above 0.
fn parse_credits(text: &str) -> Result<Option<f64>, String> {
    match parse_optional_number(text)? {
        Some(v) if v <= 0.0 => Err("Enter a number above 0".to_string()),
        other => Ok(other),
    }
}

/// R36: a score is empty (none) or a number of 0 or more. The upper bound is core's.
fn parse_score(text: &str) -> Result<Option<f64>, String> {
    match parse_optional_number(text)? {
        Some(v) if v < 0.0 => Err("Enter a number of 0 or more".to_string()),
        other => Ok(other),
    }
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
        self.errors
            .iter()
            .find(|(f, _)| f == field)
            .map(|(_, m)| m.clone())
            .unwrap_or_default()
    }

    /// True when no field has an error.
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    fn push(&mut self, field: &str, message: String) {
        self.errors.push((field.to_string(), message));
    }

    /// Records the error of `result` under `field`, so every field is checked (FR-007).
    fn check<T>(&mut self, field: &str, result: Result<T, String>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(message) => {
                self.push(field, message);
                None
            }
        }
    }
}

/// Checks `start` and `end`. R2 checks (end < start) belong to core.
pub fn parse_term_form(
    id: Option<i64>,
    name: &str,
    start: &str,
    end: &str,
) -> Result<TermInput, FormErrors> {
    let mut errors = FormErrors::default();
    let start_date = errors.check("start", parse_date(start));
    let end_date = errors.check("end", parse_date(end));
    match (start_date, end_date) {
        (Some(start_date), Some(end_date)) if errors.is_empty() => Ok(TermInput {
            id,
            name: name.trim().to_string(),
            start_date,
            end_date,
        }),
        _ => Err(errors),
    }
}

/// Checks `target` and `credits` (R33: empty = none, otherwise a number above 0). R2 checks
/// (empty code or name, range) belong to core.
#[allow(clippy::too_many_arguments)]
pub fn parse_course_form(
    code: &str,
    name: &str,
    instructor: &str,
    room: &str,
    notes: &str,
    target: &str,
    credits: &str,
    rule_set_id: Option<i64>,
) -> Result<CourseInput, FormErrors> {
    let mut errors = FormErrors::default();
    let min_attendance_pct = errors.check("target", parse_target(target));
    let credits = errors.check("credits", parse_credits(credits));
    match (min_attendance_pct, credits) {
        (Some(min_attendance_pct), Some(credits)) => Ok(CourseInput {
            code: code.trim().to_string(),
            name: name.trim().to_string(),
            instructor: optional_text(instructor),
            room: optional_text(room),
            notes: optional_text(notes),
            min_attendance_pct,
            credits,
            rule_set_id,
        }),
        _ => Err(errors),
    }
}

/// Checks `course`, `weekday`, `start`, `end`, `valid_from`, `valid_to`. R2: end <= start is core's.
pub fn parse_slot_form(
    course_id: Option<i64>,
    weekday: i32,
    start: TimeText<'_>,
    end: TimeText<'_>,
    room: &str,
    valid_from: &str,
    valid_to: &str,
) -> Result<SlotInput, FormErrors> {
    let mut errors = FormErrors::default();
    let course_id = errors.check(
        "course",
        course_id.ok_or_else(|| CHOOSE_COURSE_ERR.to_string()),
    );
    let weekday = errors.check(
        "weekday",
        weekday_from_index(weekday).ok_or_else(|| "Choose a weekday".to_string()),
    );
    let start_time = errors.check("start", parse_time(start.digits, start.pm));
    let end_time = errors.check("end", parse_time(end.digits, end.pm));
    let valid_from = errors.check("valid_from", parse_date(valid_from));
    let valid_to = errors.check("valid_to", parse_optional_date(valid_to));
    match (
        course_id, weekday, start_time, end_time, valid_from, valid_to,
    ) {
        (
            Some(course_id),
            Some(weekday),
            Some(start_time),
            Some(end_time),
            Some(valid_from),
            Some(valid_to),
        ) => Ok(SlotInput {
            course_id,
            weekday,
            start_time,
            end_time,
            room: optional_text(room),
            valid_from,
            valid_to,
        }),
        _ => Err(errors),
    }
}

/// R9, R10: checks `date`, `start`, `end` for reschedule and "Add makeup class".
pub fn parse_makeup_form(
    course_id: i64,
    date: &str,
    start: TimeText<'_>,
    end: TimeText<'_>,
    room: &str,
    note: &str,
) -> Result<MakeupInput, FormErrors> {
    let mut errors = FormErrors::default();
    let date = errors.check("date", parse_date(date));
    let start_time = errors.check("start", parse_time(start.digits, start.pm));
    let end_time = errors.check("end", parse_time(end.digits, end.pm));
    match (date, start_time, end_time) {
        (Some(date), Some(start_time), Some(end_time)) => Ok(MakeupInput {
            course_id,
            date,
            start_time,
            end_time,
            room: optional_text(room),
            note: optional_text(note),
        }),
        _ => Err(errors),
    }
}

/// Checks `course`, `kind`, `due_date`, `due_time`, `remind`, `score` (R36: empty = none,
/// otherwise a number of 0 or more). `due_at` is date plus time.
#[allow(clippy::too_many_arguments)]
pub fn parse_assessment_form(
    course_id: Option<i64>,
    kind: i32,
    title: &str,
    due_date: &str,
    due_time: TimeText<'_>,
    notes: &str,
    remind: &str,
    component_id: Option<i64>,
    score: &str,
) -> Result<AssessmentInput, FormErrors> {
    let mut errors = FormErrors::default();
    let course_id = errors.check(
        "course",
        course_id.ok_or_else(|| CHOOSE_COURSE_ERR.to_string()),
    );
    let kind = errors.check(
        "kind",
        assessment_kind_from_index(kind).ok_or_else(|| "Choose a kind".to_string()),
    );
    let date = errors.check("due_date", parse_date(due_date));
    let time = errors.check("due_time", parse_time(due_time.digits, due_time.pm));
    let remind = errors.check("remind", parse_optional_minutes(remind));
    let score = errors.check("score", parse_score(score));
    match (course_id, kind, date, time, remind, score) {
        (
            Some(course_id),
            Some(kind),
            Some(date),
            Some(time),
            Some(remind_minutes_before),
            Some(score),
        ) => Ok(AssessmentInput {
            course_id,
            kind,
            title: title.trim().to_string(),
            due_at: date.and_time(time),
            notes: optional_text(notes),
            remind_minutes_before,
            component_id,
            score,
        }),
        _ => Err(errors),
    }
}

/// Checks `start` and `end`; `end < start` is an error. Returns the expanded range.
pub fn parse_holiday_form(start: &str, end: &str, label: &str) -> Result<Vec<Holiday>, FormErrors> {
    let mut errors = FormErrors::default();
    let start = errors.check("start", parse_date(start));
    let end = errors.check("end", parse_optional_date(end));
    let (Some(start), Some(end)) = (start, end) else {
        return Err(errors);
    };
    expand_holiday_range(start, end, optional_text(label)).map_err(|msg| {
        errors.push("end", msg);
        errors
    })
}

/// Checks `reminder` and `window` via [`parse_count`]. The form has no theme field: the result
/// carries the default `theme`, and the caller keeps the saved one (§5 Settings theme).
pub fn parse_settings_form(
    reminder_minutes: &str,
    window_days: &str,
    notifications: bool,
) -> Result<Settings, FormErrors> {
    let mut errors = FormErrors::default();
    let reminder = errors.check("reminder", parse_count(reminder_minutes));
    let window = errors.check("window", parse_count(window_days));
    match (reminder, window) {
        (Some(class_reminder_minutes), Some(upcoming_window_days)) => Ok(Settings {
            class_reminder_minutes,
            upcoming_window_days,
            notifications_enabled: notifications,
            theme: ThemeMode::System,
            check_updates_on_start: true,
        }),
        _ => Err(errors),
    }
}

// ---- Holidays (single date or inclusive range) ----

/// One holiday for a single day, or one per day (same label, ascending) for an inclusive range.
pub fn expand_holiday_range(
    start: NaiveDate,
    end: Option<NaiveDate>,
    label: Option<String>,
) -> Result<Vec<Holiday>, String> {
    let end = end.unwrap_or(start);
    if end < start {
        return Err(HOLIDAY_RANGE_ERR.to_string());
    }
    Ok(start
        .iter_days()
        .take_while(|d| *d <= end)
        .map(|date| Holiday {
            date,
            label: label.clone(),
        })
        .collect())
}

/// Exactly one holiday per date, sorted; for a date in both lists the added one wins.
pub fn merge_holidays(existing: &[Holiday], added: &[Holiday]) -> Vec<Holiday> {
    let mut by_date: BTreeMap<NaiveDate, Holiday> = BTreeMap::new();
    for h in existing.iter().chain(added) {
        by_date.insert(h.date, h.clone());
    }
    by_date.into_values().collect()
}

/// Removes only `date`; the others stay unchanged and sorted.
pub fn remove_holiday(existing: &[Holiday], date: NaiveDate) -> Vec<Holiday> {
    let mut rest: Vec<Holiday> = existing
        .iter()
        .filter(|h| h.date != date)
        .cloned()
        .collect();
    rest.sort_by_key(|h| h.date);
    rest
}

/// A holiday as display text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HolidayRow {
    /// Stored ISO date (`YYYY-MM-DD`), the row's identity for Remove; never shown.
    pub key: String,
    pub date: String,
    pub label: String,
}

/// Label `None` shows an em dash.
pub fn holiday_row(h: &Holiday) -> HolidayRow {
    HolidayRow {
        key: h.date.format("%Y-%m-%d").to_string(),
        date: format_date(h.date),
        label: or_dash(h.label.as_deref()),
    }
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

/// R11, R12: builds a session row; the status label is core's `display_status` with its date
/// part in the long form ([`display_status_text`]).
pub fn session_row(view: &SessionView) -> SessionRow {
    let s = &view.session;
    SessionRow {
        id: s.id,
        date: format_date(s.date),
        weekday: weekday_name(s.date.weekday()).to_string(),
        time: format_time_range(s.start_time, s.end_time),
        course: format!("{} \u{b7} {}", view.course_code, view.course_name),
        room: or_dash(s.room.as_deref()),
        status_label: display_status_text(&view.display_status),
        chip: chip_kind(s.status),
        can_mark: can_mark_attendance(view),
        note: s.note.clone().unwrap_or_default(),
    }
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
    SummaryView {
        attended: s.attended.to_string(),
        absent: s.absent.to_string(),
        cancelled: s.cancelled.to_string(),
        pending: s.pending.to_string(),
        pct: format_pct(s.pct),
        target: format_target(s.target),
        below_target: s.below_target,
    }
}

/// R14: banner text, `CSE101 — 66.7% (target 75%)`.
pub fn banner_line(s: &AttendanceSummary) -> String {
    format!(
        "{} {DASH} {} (target {})",
        s.course_code,
        format_pct(s.pct),
        format_target(s.target)
    )
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
    /// R52: ticked for deletion.
    pub selected: bool,
}

/// R3: builds a course row; missing optional fields show an em dash.
pub fn course_row(c: &Course) -> CourseRow {
    CourseRow {
        id: c.id,
        code: c.code.clone(),
        name: c.name.clone(),
        instructor: or_dash(c.instructor.as_deref()),
        room: or_dash(c.room.as_deref()),
        target: format_target(c.min_attendance_pct),
        archived: c.archived,
        selected: false,
    }
}

/// R52: the delete button label for `count` selected courses.
pub fn delete_selected_label(count: usize) -> String {
    format!("Delete selected ({count})")
}

/// R52: the confirm text naming the selected course codes, in the given order.
pub fn delete_confirm_text(codes: &[&str]) -> String {
    let noun = if codes.len() == 1 {
        "course"
    } else {
        "courses"
    };
    format!(
        "Delete {noun} {}? Their timetable, sessions, attendance, assessments, and marks are \
         deleted too. This can't be undone; back up in Settings first if you might need them.",
        codes.join(", ")
    )
}

/// R52: keeps only selected ids that are still listed.
pub fn retain_listed(selected: &mut BTreeSet<i64>, listed: &[i64]) {
    selected.retain(|id| listed.contains(id));
}

/// R52: "Select all" is ticked when the list is non-empty and every listed course is selected.
pub fn all_selected(selected: &BTreeSet<i64>, listed: &[i64]) -> bool {
    !listed.is_empty() && listed.iter().all(|id| selected.contains(id))
}

/// `CSE101 · Intro to Programming` for combo boxes.
pub fn course_option_label(c: &Course) -> String {
    format!("{} \u{b7} {}", c.code, c.name)
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
    let course = courses
        .iter()
        .find(|c| c.id == slot.course_id)
        .map_or_else(|| DASH.to_string(), |c| c.code.clone());
    let valid = match slot.valid_to {
        Some(to) => format!(
            "From {} to {}",
            format_date(slot.valid_from),
            format_date(to)
        ),
        None => format!("From {} until term end", format_date(slot.valid_from)),
    };
    SlotRow {
        id: slot.id,
        weekday: weekday_name(slot.weekday).to_string(),
        time: format_time_range(slot.start_time, slot.end_time),
        course,
        room: or_dash(slot.room.as_deref()),
        valid,
    }
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
    AssessmentRow {
        id: a.id,
        course_code: a.course_code.clone(),
        kind: assessment_kind_label(a.kind).to_string(),
        title: a.title.clone(),
        due: format_datetime(a.due_at),
        completed: a.completed,
        notes: a.notes.clone().unwrap_or_default(),
    }
}

/// `Quiz`, `Exam`, `Assignment`, `Other`.
pub fn assessment_kind_label(k: AssessmentKind) -> &'static str {
    match k {
        AssessmentKind::Quiz => "Quiz",
        AssessmentKind::Exam => "Exam",
        AssessmentKind::Assignment => "Assignment",
        AssessmentKind::Other => "Other",
    }
}

/// Combo index 0 Quiz, 1 Exam, 2 Assignment, 3 Other; anything else is `None`.
pub fn assessment_kind_from_index(i: i32) -> Option<AssessmentKind> {
    match i {
        0 => Some(AssessmentKind::Quiz),
        1 => Some(AssessmentKind::Exam),
        2 => Some(AssessmentKind::Assignment),
        3 => Some(AssessmentKind::Other),
        _ => None,
    }
}

/// §5 Settings theme, §6 "Appearance:": appearance switch segment for a theme mode: System 0,
/// Light 1, Dark 2 (also the value of `Theme.mode` in `ui/theme.slint`).
pub fn theme_index(mode: ThemeMode) -> i32 {
    match mode {
        ThemeMode::System => 0,
        ThemeMode::Light => 1,
        ThemeMode::Dark => 2,
    }
}

/// §5 Settings theme: theme mode for an appearance switch segment; `None` when out of range.
pub fn theme_from_index(i: i32) -> Option<ThemeMode> {
    match i {
        0 => Some(ThemeMode::System),
        1 => Some(ThemeMode::Light),
        2 => Some(ThemeMode::Dark),
        _ => None,
    }
}

/// Inverse of [`assessment_kind_from_index`].
pub fn assessment_kind_index(k: AssessmentKind) -> i32 {
    match k {
        AssessmentKind::Quiz => 0,
        AssessmentKind::Exam => 1,
        AssessmentKind::Assignment => 2,
        AssessmentKind::Other => 3,
    }
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
    match i {
        1 => FilterChoice::Overdue,
        2 => FilterChoice::Completed,
        3 => FilterChoice::All,
        _ => FilterChoice::Upcoming,
    }
}

/// R18, R19: Upcoming and Overdue carry `now`.
pub fn assessment_filter(choice: FilterChoice, now: NaiveDateTime) -> AssessmentFilter {
    match choice {
        FilterChoice::Upcoming => AssessmentFilter::Upcoming { now },
        FilterChoice::Overdue => AssessmentFilter::Overdue { now },
        FilterChoice::Completed => AssessmentFilter::Completed,
        FilterChoice::All => AssessmentFilter::All,
    }
}

/// FR-005: title of the assessments list for the filter choice.
pub fn filter_title(choice: FilterChoice) -> &'static str {
    match choice {
        FilterChoice::Upcoming => "Upcoming",
        FilterChoice::Overdue => "Overdue",
        FilterChoice::Completed => "Completed",
        FilterChoice::All => "All assessments",
    }
}

/// FR-005, FR-008: empty-state sentence of the assessments list for the filter choice. Says what
/// will appear and how to add it (DESIGN.md Layout, T028 critique).
pub fn filter_empty_text(choice: FilterChoice) -> &'static str {
    match choice {
        FilterChoice::Upcoming => {
            "No upcoming assessments. Add one in the New assessment form below."
        }
        FilterChoice::Overdue => {
            "No overdue assessments. Incomplete assessments past their due date appear here."
        }
        FilterChoice::Completed => {
            "No completed assessments. Tick an assessment's checkbox to mark it complete."
        }
        FilterChoice::All => "No assessments yet. Add one in the New assessment form below.",
    }
}

// ---- Week view ----

/// Monday on or before `date`.
pub fn week_start(date: NaiveDate) -> NaiveDate {
    date - Days::new(u64::from(date.weekday().num_days_from_monday()))
}

/// Sunday of the week starting at `week_start`.
pub fn week_end(week_start: NaiveDate) -> NaiveDate {
    week_start + Days::new(6)
}

/// Moves by whole weeks, across month and year ends.
pub fn shift_week(week_start: NaiveDate, weeks: i64) -> NaiveDate {
    week_start + Duration::weeks(weeks)
}

/// Week range with an en dash and spaces (human decision 2026-10-29): `October 5 – 11, 2026`
/// within one month, `October 26 – November 1, 2026` when the month changes, and
/// `December 28, 2026 – January 3, 2027` when the year changes.
pub fn week_title(week_start: NaiveDate) -> String {
    let end = week_end(week_start);
    let start_text = if week_start.year() != end.year() {
        format_date(week_start)
    } else {
        week_start.format("%B %-d").to_string()
    };
    let end_text = if week_start.year() == end.year() && week_start.month() == end.month() {
        end.format("%-d, %Y").to_string()
    } else {
        format_date(end)
    };
    format!("{start_text} \u{2013} {end_text}")
}

/// Sessions of one day.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayGroup {
    pub title: String,
    pub rows: Vec<SessionRow>,
}

/// One group per distinct date in date order, keeping input order inside a day; titles read
/// `Monday, October 26, 2026`.
pub fn group_week(views: &[SessionView]) -> Vec<DayGroup> {
    let mut by_date: BTreeMap<NaiveDate, Vec<SessionRow>> = BTreeMap::new();
    for view in views {
        by_date
            .entry(view.session.date)
            .or_default()
            .push(session_row(view));
    }
    by_date
        .into_iter()
        .map(|(date, rows)| DayGroup {
            title: format!("{}, {}", weekday_name(date.weekday()), format_date(date)),
            rows,
        })
        .collect()
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
    DashboardView {
        today: d.today.iter().map(session_row).collect(),
        next_class: d.next_class.as_ref().map(session_row),
        upcoming: d.upcoming.iter().map(assessment_row).collect(),
        overdue: d.overdue.iter().map(assessment_row).collect(),
        awaiting: d.awaiting.iter().map(session_row).collect(),
        banner: d.below_target.iter().map(banner_line).collect(),
    }
}

/// R16, R17 (T028 critique): false when the next class is already listed in Today's classes,
/// so the dashboard does not show the same session twice. With no next class the card stays
/// visible to show its empty state.
pub fn show_next_card(next: Option<&SessionRow>, today: &[SessionRow]) -> bool {
    match next {
        Some(row) => !today.iter().any(|t| t.id == row.id),
        None => true,
    }
}

// ---- Errors, files, timer ----

/// R2, R25: user-facing error text; `Validation` messages are capitalized.
/// R31: any `HH:MM` inside a `Validation` message is shown in 12-hour form.
pub fn error_text(e: &planner_core::Error) -> String {
    match e {
        planner_core::Error::Validation(msg) => {
            let mut chars = msg.chars();
            let capitalised: String = match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            };
            times_to_12h(&capitalised)
        }
        other => other.to_string(),
    }
}

/// R22: `pre-restore-20261029-1430.sqlite` (seconds ignored).
pub fn pre_restore_file_name(now: NaiveDateTime) -> String {
    format!("pre-restore-{}.sqlite", now.format("%Y%m%d-%H%M"))
}

/// R22: the base name if free, else the first free `-2`, `-3`, … suffix; unrelated names ignored.
pub fn free_pre_restore_name(now: NaiveDateTime, existing: &[String]) -> String {
    let taken = |name: &str| existing.iter().any(|e| e == name);
    let base = pre_restore_file_name(now);
    if !taken(&base) {
        return base;
    }
    let stem = format!("pre-restore-{}", now.format("%Y%m%d-%H%M"));
    (2u32..)
        .map(|n| format!("{stem}-{n}.sqlite"))
        .find(|name| !taken(name))
        .unwrap_or(base)
}

/// `academic-planner-backup-20261029-1430.json`.
pub fn backup_file_name(now: NaiveDateTime) -> String {
    format!("academic-planner-backup-{}.json", now.format("%Y%m%d-%H%M"))
}

/// R6: true once the calendar date moved past the last sync date.
pub fn date_rolled_over(last_sync: NaiveDate, now: NaiveDateTime) -> bool {
    now.date() != last_sync
}

/// R24: display text of the database path.
pub fn db_path_text(path: &Path) -> String {
    path.display().to_string()
}

/// Message shown after a restore (R22).
pub fn restored_message(copy: &Path) -> String {
    format!("Backup restored. Previous data saved to {}", copy.display())
}

// ---- Course form (T017) ----

/// R2: a stored target as edit-form text (`75`, or empty when there is none), so saving the form
/// unchanged parses back to the same value.
pub fn target_input(target: Option<i64>) -> String {
    target.map(|t| t.to_string()).unwrap_or_default()
}

// ---- Settings page messages ----

/// R21: message shown after a backup file is written.
pub fn backup_saved_message(path: &Path) -> String {
    format!("Backup saved to {}", path.display())
}

/// R23: message shown after both CSV files are written to `folder`.
pub fn csv_exported_message(folder: &Path) -> String {
    format!(
        "Exported attendance.csv and assessments.csv to {}",
        folder.display()
    )
}

/// R89: toast text: Term saved.
pub fn term_saved_text() -> String {
    "Term saved.".to_string()
}

/// R89: toast text: Holidays saved.
pub fn holidays_saved_text() -> String {
    "Holidays saved.".to_string()
}

/// R89: toast text: Holiday removed.
pub fn holiday_removed_text() -> String {
    "Holiday removed.".to_string()
}

/// R89: toast text: Settings saved.
pub fn settings_saved_text() -> String {
    "Settings saved.".to_string()
}

/// R89: toast text: Template copied
pub fn template_copied_text() -> String {
    "Template copied".to_string()
}

/// R89: toast text: AI prompt copied
pub fn prompt_copied_text() -> String {
    "AI prompt copied".to_string()
}

/// R90: toast text: Rule set saved.
pub fn rule_set_saved_text() -> String {
    "Rule set saved.".to_string()
}

/// R90: toast text: Grade scale saved.
pub fn grade_scale_saved_text() -> String {
    "Grade scale saved.".to_string()
}

/// R91: the hint above the Save button when a row fails to parse
pub fn fix_rows_hint_text() -> String {
    "Fix the rows marked in red before saving.".to_string()
}

/// Message for a file that could not be read or written.
pub fn file_error_message(path: &Path, cause: &str) -> String {
    format!("Could not access {}: {cause}", path.display())
}

/// R22: message when the pre-restore copy fails; the restore is not attempted.
pub fn pre_restore_failed_message(cause: &str) -> String {
    format!("Restore not started: could not copy the current data ({cause})")
}

// ---- Mini calendar (§6 mini calendar change) ----

/// R16: empty sentence of the day panel when today is selected (unchanged Today's classes text).
const TODAY_EMPTY_TEXT: &str =
    "No classes today. Add courses in Courses and weekly slots in Timetable \u{2192} Rules.";
/// §6 mini calendar change: empty sentence of the day panel for any other day.
const OTHER_DAY_EMPTY_TEXT: &str = "Nothing on this day.";
/// §6 mini calendar change: six weeks of seven days.
const GRID_CELLS: u64 = 42;

/// First day of `d`'s month.
pub fn month_start(d: NaiveDate) -> NaiveDate {
    d.with_day(1).unwrap_or(d)
}

/// First day of the month `months` away, across year ends.
pub fn shift_month(month_start: NaiveDate, months: i32) -> NaiveDate {
    let index = month_start.year() * 12 + month_start.month0() as i32 + months;
    let year = index.div_euclid(12);
    let month0 = index.rem_euclid(12) as u32;
    NaiveDate::from_ymd_opt(year, month0 + 1, 1).unwrap_or(month_start)
}

/// `October 2026`.
pub fn month_title(month_start: NaiveDate) -> String {
    month_start.format("%B %Y").to_string()
}

/// R3, R12: one entry per non-cancelled session (pending, attended, absent), duplicates kept;
/// cancelled sessions and rescheduled originals (status `Cancelled`) are excluded.
pub fn class_days(views: &[SessionView]) -> Vec<NaiveDate> {
    views
        .iter()
        .filter(|v| v.session.status != Status::Cancelled)
        .map(|v| v.session.date)
        .collect()
}

/// R16: one entry per incomplete assessment (`due_at.date()`), duplicates kept.
pub fn due_days(assessments: &[Assessment]) -> Vec<NaiveDate> {
    assessments
        .iter()
        .filter(|a| !a.completed)
        .map(|a| a.due_at.date())
        .collect()
}

/// One cell of the mini calendar grid.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalendarDay {
    pub key: String,
    pub day: String,
    pub in_month: bool,
    pub is_today: bool,
    pub is_selected: bool,
    pub has_class: bool,
    pub has_due: bool,
    /// Saturday or Sunday (§6 mini calendar change).
    pub is_weekend: bool,
    /// Holiday label for the date: `"Holiday"` when unlabeled, `""` when not a holiday.
    pub holiday: String,
    pub label: String,
}

/// First day of the grid for `month_start`: the Monday on or before the 1st.
pub fn grid_start(month_start: NaiveDate) -> NaiveDate {
    week_start(month_start)
}

/// Last day of the grid for `month_start` (41 days after [`grid_start`]).
pub fn grid_end(month_start: NaiveDate) -> NaiveDate {
    grid_start(month_start) + Days::new(GRID_CELLS - 1)
}

/// `1 class` / `2 classes`, and so on.
fn count_text(n: usize, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{n} {many}")
    }
}

/// R14: holiday text for `date`: its label, `Holiday` when unlabeled, `""` when not a holiday
/// (§6 mini calendar change).
pub fn holiday_text(holidays: &[Holiday], date: NaiveDate) -> String {
    holidays
        .iter()
        .find(|h| h.date == date)
        .map(|h| h.label.clone().unwrap_or_else(|| "Holiday".to_string()))
        .unwrap_or_default()
}

/// Always 42 cells starting at the Monday on or before `month_start`. The label is the long
/// weekday date, then `, today`, then the holiday text, then the class and due counts when
/// non-zero (R3, R14, R16). A holiday without a label reads `Holiday` (§6 mini calendar change).
pub fn month_grid(
    month_start: NaiveDate,
    today: NaiveDate,
    selected: NaiveDate,
    class_days: &[NaiveDate],
    due_days: &[NaiveDate],
    holidays: &[Holiday],
) -> Vec<CalendarDay> {
    let first = grid_start(month_start);
    (0..GRID_CELLS)
        .map(|offset| {
            let date = first + Days::new(offset);
            let classes = class_days.iter().filter(|d| **d == date).count();
            let dues = due_days.iter().filter(|d| **d == date).count();
            let is_today = date == today;
            let mut label = format!("{}, {}", weekday_name(date.weekday()), format_date(date));
            if is_today {
                label.push_str(", today");
            }
            let holiday = holiday_text(holidays, date);
            if !holiday.is_empty() {
                label.push_str(", ");
                label.push_str(&holiday);
            }
            if classes > 0 {
                label.push_str(", ");
                label.push_str(&count_text(classes, "class", "classes"));
            }
            if dues > 0 {
                label.push_str(", ");
                label.push_str(&count_text(dues, "assessment due", "assessments due"));
            }
            CalendarDay {
                key: date.format("%Y-%m-%d").to_string(),
                day: date.day().to_string(),
                in_month: date.year() == month_start.year() && date.month() == month_start.month(),
                is_today,
                is_selected: date == selected,
                has_class: classes > 0,
                has_due: dues > 0,
                is_weekend: matches!(date.weekday(), Weekday::Sat | Weekday::Sun),
                holiday,
                label,
            }
        })
        .collect()
}

/// `M T W T F S S`.
pub fn weekday_initials() -> [&'static str; 7] {
    ["M", "T", "W", "T", "F", "S", "S"]
}

/// R16: `Today's classes` for today, else the long weekday date.
pub fn day_panel_title(selected: NaiveDate, today: NaiveDate) -> String {
    if selected == today {
        "Today's classes".to_string()
    } else {
        format!(
            "{}, {}",
            weekday_name(selected.weekday()),
            format_date(selected)
        )
    }
}

/// Holiday line of the day panel: empty for no holiday, else `Holiday: <text>` (§6 mini calendar
/// change).
pub fn day_panel_holiday_text(holiday: &str) -> String {
    if holiday.is_empty() {
        String::new()
    } else {
        format!("Holiday: {holiday}")
    }
}

/// R16: empty text of the day panel; today keeps the Today's classes sentence.
pub fn day_panel_empty_text(selected: NaiveDate, today: NaiveDate) -> String {
    if selected == today {
        TODAY_EMPTY_TEXT.to_string()
    } else {
        OTHER_DAY_EMPTY_TEXT.to_string()
    }
}

/// Parses a `CalendarDay.key` (`2026-10-26`); anything else is `None`.
pub fn parse_date_key(key: &str) -> Option<NaiveDate> {
    if key.len() != 10 {
        return None;
    }
    NaiveDate::parse_from_str(key, "%Y-%m-%d").ok()
}

// ---- About page and updates (§13.1 UI, R27–R29) ----
// Contract: `specs/007-m7-updater/contracts/ui-about.md`.

/// R27, R28: where the update flow is, as shown on the About page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdatePhase {
    Idle,
    Checking,
    UpToDate,
    Available { version: String },
    Downloading { received: u64, total: Option<u64> },
    Installing,
    CheckFailed,
    InstallFailed,
}

/// R27, R28: the About page's update status line, action button, and progress.
#[derive(Debug, Clone, PartialEq)]
pub struct UpdateView {
    pub status: String,
    pub action: String,
    pub action_enabled: bool,
    pub progress: f32,
    pub progress_text: String,
    pub is_error: bool,
}

/// R27, R28: status text, action label, and progress for `phase`. The action is disabled
/// while a check, download, or install is running. Progress is 0 and its text empty outside
/// Downloading.
pub fn update_view(phase: &UpdatePhase) -> UpdateView {
    const CHECK: &str = "Check for updates";
    const INSTALL: &str = "Install";
    const RETRY: &str = "Retry";
    let (status, action, action_enabled, is_error) = match phase {
        UpdatePhase::Idle => (String::new(), CHECK, true, false),
        UpdatePhase::Checking => ("Checking for updates…".to_string(), CHECK, false, false),
        UpdatePhase::UpToDate => ("You're up to date.".to_string(), CHECK, true, false),
        UpdatePhase::Available { version } => (banner_text(version), INSTALL, true, false),
        UpdatePhase::Downloading { .. } => ("Downloading…".to_string(), INSTALL, false, false),
        UpdatePhase::Installing => ("Installing…".to_string(), INSTALL, false, false),
        UpdatePhase::CheckFailed => ("Couldn't check for updates".to_string(), RETRY, true, true),
        UpdatePhase::InstallFailed => (
            "The update couldn't be installed. Nothing was changed.".to_string(),
            RETRY,
            true,
            true,
        ),
    };
    let (progress, progress_text) = match phase {
        UpdatePhase::Downloading { received, total } => (
            progress_fraction(*received, *total),
            download_text(*received, *total),
        ),
        _ => (0.0, String::new()),
    };
    UpdateView {
        status,
        action: action.to_string(),
        action_enabled,
        progress,
        progress_text,
        is_error,
    }
}

/// R28: the total download size when it is known and non-zero; `Some(0)` counts as unknown.
fn known_total(total: Option<u64>) -> Option<u64> {
    total.filter(|&t| t > 0)
}

/// R28: the whole download percentage, rounded down so 100% shows only when done.
pub fn download_percent(received: u64, total: Option<u64>) -> Option<u64> {
    known_total(total).map(|t| (u128::from(received.min(t)) * 100 / u128::from(t)) as u64)
}

/// R28: "Downloading… 42%", or "Downloading…" when the total size is unknown.
fn download_text(received: u64, total: Option<u64>) -> String {
    match download_percent(received, total) {
        Some(percent) => format!("Downloading… {percent}%"),
        None => "Downloading…".to_string(),
    }
}

/// §13.1 UI: "Version 0.1.1".
pub fn version_text(version: &str) -> String {
    format!("Version {version}")
}

/// §13.1 UI: the one-line license statement.
pub fn license_text() -> String {
    "Free software under the GNU GPL v3 or later.".to_string()
}

/// R53: the repo path beside the GitHub button, e.g. `sahil0319/Academic-Planner`.
pub fn repo_label(url: &str) -> String {
    url.trim_end_matches('/')
        .trim_start_matches("https://github.com/")
        .to_string()
}

/// R29: the update banner, e.g. "Version 0.2.0 is available.".
pub fn banner_text(version: &str) -> String {
    format!("Version {version} is available.")
}

/// R28: the confirm panel title, e.g. "Install version 0.2.0?".
pub fn confirm_title(version: &str) -> String {
    format!("Install version {version}?")
}

/// R28: the confirm button label for this kind of install.
pub fn confirm_label(kind: InstallKind) -> String {
    match kind {
        InstallKind::SelfInstall => "Install and restart",
        InstallKind::OpenReleasePage => "Open release page",
    }
    .to_string()
}

/// R28: download progress clamped to 0.0..=1.0, or -1.0 when the total size is unknown
/// (missing or zero).
pub fn progress_fraction(received: u64, total: Option<u64>) -> f32 {
    match known_total(total) {
        Some(t) => (received as f64 / t as f64).clamp(0.0, 1.0) as f32,
        None => -1.0,
    }
}

// ---- M9: credits, rule sets, marks, GPA (contracts/ui-grading.md) ----

/// R38: one row of the needed-in-final table as display text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NeededRowView {
    pub letter: String,
    pub text: String,
}

/// R37: a mark such as `45` or `37.5`, without a trailing `.0`. Two decimals at most.
pub fn format_mark(value: f64) -> String {
    let rounded = (value * 100.0).round() / 100.0;
    if rounded == 0.0 {
        return "0".to_string();
    }
    format!("{rounded}")
}

/// R35/R36: `"39.0 of 45"`, earned with one decimal, possible as a plain mark.
pub fn format_marks(earned: f64, possible: f64) -> String {
    format!("{} of {}", fixed(earned, 1), format_mark(possible))
}

/// R37: `"80.0% · A+"`; `None` shows an em dash.
pub fn format_projected(pct: Option<f64>, letter: Option<&str>) -> String {
    match (pct, letter) {
        (Some(pct), Some(letter)) => format!("{}% \u{b7} {letter}", fixed(pct, 1)),
        (Some(pct), None) => format!("{}%", fixed(pct, 1)),
        _ => DASH.to_string(),
    }
}

/// R38: `"111.0"`, `"Secured"`, or `"Not reachable"`.
pub fn needed_text(needed: &Needed) -> String {
    match needed {
        Needed::Score(v) => fixed(*v, 1),
        Needed::Secured => "Secured".to_string(),
        Needed::NotReachable => "Not reachable".to_string(),
    }
}

/// R38: one row per grade letter in scale order; empty for `FinalOutlook::Actual`.
pub fn needed_rows(outlook: &FinalOutlook) -> Vec<NeededRowView> {
    match outlook {
        FinalOutlook::Needed { rows, .. } => rows
            .iter()
            .map(|r| NeededRowView {
                letter: r.letter.clone(),
                text: needed_text(&r.needed),
            })
            .collect(),
        FinalOutlook::Actual { .. } => Vec::new(),
    }
}

/// R38: `"Quizzes: assumes 0 for 1 item not entered"`.
pub fn assumption_text(assumption: &Assumption) -> String {
    format!(
        "{}: assumes 0 for {} not entered",
        assumption.component_name,
        count_text(assumption.missing_items as usize, "item", "items")
    )
}

/// R38: `"Final grade: A"`; `None` gives `"Final grade: —"`.
pub fn actual_grade_text(letter: Option<&str>) -> String {
    format!("Final grade: {}", letter.unwrap_or(DASH))
}

/// R40: `"Term GPA 3.67 (projected)"`, with the left-out count appended when above 0.
pub fn gpa_header_text(summary: &GpaSummary) -> String {
    let mut text = match summary.gpa {
        Some(gpa) => {
            let status = match summary.status {
                GpaStatus::Projected => "projected",
                GpaStatus::Final => "final",
            };
            format!("Term GPA {} ({status})", fixed(gpa, 2))
        }
        None => format!("Term GPA {DASH}"),
    };
    if summary.left_out > 0 {
        text.push_str(&format!(
            " \u{b7} {} without marks",
            count_text(summary.left_out as usize, "graded course", "graded courses")
        ));
    }
    text
}

/// R33: the Marks card line shown for a course without credits or a rule set.
pub const NOT_GRADED_LINE: &str =
    "Not graded. Add credits and a rule set to the course to see marks.";

/// R35-R38: display text of the course detail's Marks card and Needed in final table.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MarksView {
    /// R33: false shows only [`NOT_GRADED_LINE`].
    pub graded: bool,
    /// R35/R36: `(component name, "39.0 of 45")` in rule set order.
    pub rows: Vec<(String, String)>,
    /// R37: `"129.0 of 300"`, secured marks of the course total.
    pub secured: String,
    /// R37: `"86.0% · A+"` or an em dash.
    pub projected: String,
    /// R38: `"Score needed in the final, out of 150"`; empty when the final is scored.
    pub needed_caption: String,
    /// R38: one row per grade letter; empty when the final is scored.
    pub needed: Vec<NeededRowView>,
    /// R38: assumption lines under the table.
    pub assumptions: Vec<String>,
    /// R38: `"Final grade: A"` when the final is scored, else empty.
    pub actual: String,
}

/// R33, R35-R38: view of a course's marks.
pub fn marks_view(marks: &CourseMarks) -> MarksView {
    if !marks.graded {
        return MarksView::default();
    }
    let mut view = MarksView {
        graded: true,
        rows: marks
            .components
            .iter()
            .map(|c| (c.name.clone(), format_marks(c.earned, c.marks)))
            .collect(),
        secured: format_marks(marks.secured, marks.total),
        projected: format_projected(marks.projected_pct, marks.letter.as_deref()),
        ..MarksView::default()
    };
    match &marks.outlook {
        Some(
            outlook @ FinalOutlook::Needed {
                assumptions,
                final_item_max,
                ..
            },
        ) => {
            view.needed_caption = format!(
                "Score needed in the final, out of {}",
                format_mark(*final_item_max)
            );
            view.needed = needed_rows(outlook);
            view.assumptions = assumptions.iter().map(assumption_text).collect();
        }
        Some(FinalOutlook::Actual { letter }) => {
            view.actual = actual_grade_text(letter.as_deref());
        }
        None => {}
    }
    view
}

/// R34: `"Items · 15% · best 3 of 4, out of 15"`.
pub fn component_summary(component: &RuleComponent) -> String {
    let weight = format_mark(component.weight_pct);
    match component.kind {
        ComponentKind::Attendance => format!("Attendance \u{b7} {weight}%"),
        ComponentKind::Items => {
            let mut text = format!("Items \u{b7} {weight}%");
            if let (Some(count), Some(best)) = (component.item_count, component.best_count) {
                let prefix = if best < count { "best " } else { "" };
                text.push_str(&format!(" \u{b7} {prefix}{best} of {count}"));
                if let Some(max) = component.item_max {
                    text.push_str(&format!(", out of {}", format_mark(max)));
                }
            }
            if component.is_final {
                text.push_str(" \u{b7} final");
            }
            text
        }
    }
}

/// R87: the lines of the "Saved rules" list: each component once as `"name: summary"`, each band
/// of an attendance component indented by two spaces under it.
pub fn rule_set_lines(set: &RuleSet) -> Vec<String> {
    set.components
        .iter()
        .flat_map(|c| {
            std::iter::once(format!("{}: {}", c.name, component_summary(c)))
                .chain(c.bands.iter().map(|b| format!("  {}", band_text(b))))
        })
        .collect()
}

/// R35: `"≥ 90% → 100%"`.
pub fn band_text(band: &AttendanceBand) -> String {
    format!(
        "\u{2265} {}% \u{2192} {}%",
        format_mark(band.min_pct),
        format_mark(band.credit_pct)
    )
}

/// R36: `"/ 15"` for `Some(15.0)`; `None` gives `""`.
pub fn score_max_label(item_max: Option<f64>) -> String {
    item_max
        .map(|m| format!("/ {}", format_mark(m)))
        .unwrap_or_default()
}

/// R34: parses the number fields of one component row; range checks stay in core.
///
/// An attendance row has no item fields (they are ignored); an items row has no bands.
#[allow(clippy::too_many_arguments)]
pub fn parse_component_row(
    id: Option<i64>,
    name: &str,
    kind: ComponentKind,
    weight: &str,
    item_count: &str,
    best_count: &str,
    item_max: &str,
    is_final: bool,
    bands: Vec<AttendanceBand>,
) -> Result<RuleComponentInput, FormErrors> {
    let mut errors = FormErrors::default();
    let weight_pct = errors.check("weight", parse_number(weight));
    let items = match kind {
        ComponentKind::Attendance => Some((None, None, None)),
        ComponentKind::Items => {
            let item_count = errors.check("item_count", parse_optional_count(item_count));
            let best_count = errors.check("best_count", parse_optional_count(best_count));
            let item_max = errors.check("item_max", parse_optional_number(item_max));
            match (item_count, best_count, item_max) {
                (Some(c), Some(b), Some(m)) => Some((c, b, m)),
                _ => None,
            }
        }
    };
    match (weight_pct, items) {
        (Some(weight_pct), Some((item_count, best_count, item_max))) => Ok(RuleComponentInput {
            id,
            name: name.trim().to_string(),
            kind,
            weight_pct,
            item_count,
            best_count,
            item_max,
            is_final,
            bands: match kind {
                ComponentKind::Attendance => bands,
                ComponentKind::Items => Vec::new(),
            },
        }),
        _ => Err(errors),
    }
}

/// R35: parses one attendance band row; range checks stay in core.
pub fn parse_band_row(min: &str, credit: &str) -> Result<AttendanceBand, FormErrors> {
    let mut errors = FormErrors::default();
    let min_pct = errors.check("min", parse_number(min));
    let credit_pct = errors.check("credit", parse_number(credit));
    match (min_pct, credit_pct) {
        (Some(min_pct), Some(credit_pct)) => Ok(AttendanceBand {
            min_pct,
            credit_pct,
        }),
        _ => Err(errors),
    }
}

/// R39: parses one grade scale row; range checks stay in core. The letter is trimmed.
pub fn parse_grade_row(letter: &str, min: &str, points: &str) -> Result<GradeRow, FormErrors> {
    let mut errors = FormErrors::default();
    let letter = letter.trim();
    if letter.is_empty() {
        errors.push("letter", "Required".to_string());
    }
    let min_pct = errors.check("min", parse_number(min));
    let points = errors.check("points", parse_number(points));
    match (min_pct, points) {
        (Some(min_pct), Some(points)) if errors.is_empty() => Ok(GradeRow {
            letter: letter.to_string(),
            min_pct,
            points,
        }),
        _ => Err(errors),
    }
}

// ---- M9: rules page rows and choice lists (contracts/ui-grading.md) ----

/// R33: first label of the course form's rule set choice.
pub const NOT_GRADED_LABEL: &str = "Not graded";
/// R36: first label of the assessment form's "Counts toward" choice.
pub const NOT_COUNTED_LABEL: &str = "Not counted";

/// R33, R36: a stored optional number as input text: empty for `None`, else `format_mark`.
pub fn number_input(value: Option<f64>) -> String {
    value.map(format_mark).unwrap_or_default()
}

/// R33, R36: choice labels with `none_label` at index 0, then `names` in order.
pub fn choice_labels(none_label: &str, names: &[String]) -> Vec<String> {
    std::iter::once(none_label.to_string())
        .chain(names.iter().cloned())
        .collect()
}

/// R33, R36: the id behind choice `index` of a list built by `choice_labels` (index 0 = none, so
/// `ids[index - 1]`). Out of range is `None`.
pub fn choice_id(index: i32, ids: &[i64]) -> Option<i64> {
    let i = usize::try_from(index).ok()?.checked_sub(1)?;
    ids.get(i).copied()
}

/// R33, R36: the choice index of `id` in a list built by `choice_labels`; 0 (none) when `id` is
/// `None` or not listed.
pub fn choice_index(id: Option<i64>, ids: &[i64]) -> i32 {
    id.and_then(|id| ids.iter().position(|&x| x == id))
        .and_then(|i| i32::try_from(i + 1).ok())
        .unwrap_or(0)
}

/// R34: a rule set in the Rules page list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSetRow {
    pub id: i64,
    pub name: String,
    pub summary: String,
}

/// R34: `"Attendance 10%, Quizzes 15%, Mid 25%"`, in component order.
pub fn rule_set_row(set: &RuleSet) -> RuleSetRow {
    let summary = set
        .components
        .iter()
        .map(|c| format!("{} {}%", c.name, format_mark(c.weight_pct)))
        .collect::<Vec<_>>()
        .join(", ");
    RuleSetRow {
        id: set.id,
        name: set.name.clone(),
        summary,
    }
}

/// R34: one editable component row as input text. `id` is `None` for a row not yet saved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ComponentFormRow {
    pub id: Option<i64>,
    pub name: String,
    pub is_items: bool,
    pub weight: String,
    pub item_count: String,
    pub best_count: String,
    pub item_max: String,
    pub is_final: bool,
}

/// R34: a stored component as editable input text; attendance has empty item fields.
pub fn component_form_row(c: &RuleComponent) -> ComponentFormRow {
    ComponentFormRow {
        id: Some(c.id),
        name: c.name.clone(),
        is_items: c.kind == ComponentKind::Items,
        weight: format_mark(c.weight_pct),
        item_count: c.item_count.map(|n| n.to_string()).unwrap_or_default(),
        best_count: c.best_count.map(|n| n.to_string()).unwrap_or_default(),
        item_max: number_input(c.item_max),
        is_final: c.is_final,
    }
}

/// R35: a stored band as `(min %, credit %)` input text.
pub fn band_form_row(b: &AttendanceBand) -> (String, String) {
    (format_mark(b.min_pct), format_mark(b.credit_pct))
}

/// R39: a stored grade row as `(letter, min %, points)` input text.
pub fn grade_form_row(g: &GradeRow) -> (String, String, String) {
    (
        g.letter.clone(),
        format_mark(g.min_pct),
        format_mark(g.points),
    )
}

/// R34, R35, R39: the field errors of one Rules page row as one line, e.g.
/// `"Weight: Enter a number; Out of: Enter a number"`.
pub fn field_errors_text(errors: &FormErrors) -> String {
    errors
        .errors
        .iter()
        .map(|(field, message)| {
            let label = match field.as_str() {
                "weight" => "Weight",
                "item_count" => "Items held",
                "best_count" => "Best counted",
                "item_max" => "Out of",
                "min" => "Min %",
                "credit" => "Credit %",
                "letter" => "Letter",
                "points" => "Points",
                other => other,
            };
            format!("{label}: {message}")
        })
        .collect::<Vec<_>>()
        .join("; ")
}

// ---- M13: Progress page ----

/// R65: card status; declaration order is the R61 card order.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ProgressStatus {
    AtRisk,
    Watch,
    OnTrack,
    NotEnoughData,
}

/// R65: "At risk", "Watch", "On track", "Not enough data".
#[allow(dead_code, unused_variables)]
pub fn progress_status_label(status: ProgressStatus) -> &'static str {
    todo!("M13: R65")
}

/// R65: On track = Attended, Watch = Pending, At risk = Absent, Not enough data = Cancelled.
#[allow(dead_code, unused_variables)]
pub fn progress_chip(status: ProgressStatus) -> ChipKind {
    todo!("M13: R65")
}

/// R65: `points` is the grade points of the projected letter (None when no projected letter).
#[allow(dead_code, unused_variables)]
pub fn progress_status(
    below_target: bool,
    pct: Option<f64>,
    target: Option<i64>,
    projected_pct: Option<f64>,
    points: Option<f64>,
) -> ProgressStatus {
    todo!("M13: R65")
}

/// R65: points of `letter` in the grade scale; None when the letter is None or not in the scale.
#[allow(dead_code, unused_variables)]
pub fn letter_points(letter: Option<&str>, scale: &[GradeRow]) -> Option<f64> {
    todo!("M13: R65")
}

/// Noun for a class count: "class" when `n` is 1, else "classes" (D6).
fn class_noun(n: u32) -> &'static str {
    if n == 1 {
        "class"
    } else {
        "classes"
    }
}

/// R63: k = floor((100(a + r) - t(a + b + r)) / 100) in i64 (`div_euclid`); "" = no line.
///
/// Checks run in the D5 order: no target gives no line; no remaining classes gives "No classes
/// left to mark"; then `k >= r` (miss all), `0 < k < r` (miss k), `k = 0` (attend all), and
/// `k < 0` (out of reach). A negative numerator floors, so -25 / 100 is -1 (D4).
#[allow(dead_code)]
pub fn miss_line(attended: u32, absent: u32, remaining: u32, target: Option<i64>) -> String {
    let Some(t) = target else {
        return String::new();
    };
    if remaining == 0 {
        return "No classes left to mark".to_string();
    }
    let (a, b, r) = (i64::from(attended), i64::from(absent), i64::from(remaining));
    let k = (100 * (a + r) - t * (a + b + r)).div_euclid(100);
    let noun = class_noun(remaining);
    if k >= r {
        format!("You can miss all {remaining} remaining {noun} and stay at or above {t}%")
    } else if k > 0 {
        format!("You can miss {k} of the {remaining} remaining {noun} and stay at or above {t}%")
    } else if k == 0 {
        format!("Attend all {remaining} remaining {noun} to stay at or above {t}%")
    } else {
        format!("{t}% is out of reach, even attending all {remaining} remaining {noun}")
    }
}

/// R62: "87.5% · 14 of 16 classes", or "No classes marked yet" when attended + absent = 0.
#[allow(dead_code)]
pub fn attendance_line(attended: u32, absent: u32, pct: Option<f64>) -> String {
    let marked = attended + absent;
    if marked == 0 {
        return "No classes marked yet".to_string();
    }
    format!(
        "{} \u{b7} {attended} of {marked} {}",
        format_pct(pct),
        class_noun(marked)
    )
}

/// R62: "Target 75%", or "No target" (D7).
#[allow(dead_code)]
pub fn progress_target_text(target: Option<i64>) -> String {
    match target {
        Some(t) => format!("Target {t}%"),
        None => "No target".to_string(),
    }
}

/// R62: bar fill 0.0..=1.0 = pct / 100, clamped; 0.0 when pct is None (empty bar).
#[allow(dead_code)]
pub fn bar_fraction(pct: Option<f64>) -> f32 {
    match pct {
        Some(p) if p.is_finite() => (p / 100.0).clamp(0.0, 1.0) as f32,
        _ => 0.0,
    }
}

/// R62: marker position 0.0..=1.0 = target / 100, clamped; None = no marker.
#[allow(dead_code)]
pub fn target_fraction(target: Option<i64>) -> Option<f32> {
    target.map(|t| (t as f64 / 100.0).clamp(0.0, 1.0) as f32)
}

/// R64: "Not graded" | "No marks entered yet" | "Secured 129.0 of 150 assessed marks".
#[allow(dead_code, unused_variables)]
pub fn progress_marks_line(marks: &CourseMarks) -> String {
    todo!("M13: R64")
}

/// R64, R38: "A needs 96.0 of 150 in the final" | "A+ is already secured" | "Final grade: A";
/// "" when there is no outlook (not graded) or every row is Not reachable.
#[allow(dead_code, unused_variables)]
pub fn best_grade_line(outlook: Option<&FinalOutlook>) -> String {
    todo!("M13: R64")
}

/// R61: "1 on track · 1 to watch · 1 at risk" (Not enough data is not counted).
#[allow(dead_code, unused_variables)]
pub fn progress_counts_text(statuses: &[ProgressStatus]) -> String {
    todo!("M13: R61")
}

/// R61: empty-state text; `has_term` false gives the no-term text, else the no-courses text.
#[allow(dead_code, unused_variables)]
pub fn progress_empty_text(has_term: bool) -> &'static str {
    todo!("M13: R61")
}

/// R61–R65: one card.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressCardView {
    /// Sort key and title.
    pub code: String,
    pub name: String,
    pub status: ProgressStatus,
    /// `progress_status_label`.
    pub status_label: String,
    /// `progress_chip`.
    pub chip: ChipKind,
    /// `format_pct(pct)`: "87.5%" or "—".
    pub attendance_figure: String,
    /// `attendance_line`.
    pub attendance_line: String,
    /// `progress_target_text`.
    pub target_text: String,
    /// `bar_fraction`.
    pub bar_fill: f32,
    /// `target_fraction(..).is_some()`.
    pub has_marker: bool,
    /// `target_fraction(..).unwrap_or(0.0)`.
    pub marker_at: f32,
    /// `miss_line(attended, absent, pending, target)`.
    pub miss_line: String,
    /// `format_projected(projected_pct, letter)` when graded, else "—".
    pub marks_figure: String,
    /// `progress_marks_line`.
    pub marks_line: String,
    /// `best_grade_line(marks.outlook.as_ref())`.
    pub best_grade_line: String,
}

/// R61–R65: build one card from the attendance summary, the course marks, and the grade scale.
#[allow(dead_code, unused_variables)]
pub fn progress_card(
    summary: &AttendanceSummary,
    marks: &CourseMarks,
    scale: &[GradeRow],
) -> ProgressCardView {
    todo!("M13: R61")
}

/// R61: header texts and cards sorted by (status, code).
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressView {
    pub gpa_text: String,
    pub counts_text: String,
    pub cards: Vec<ProgressCardView>,
}

/// R61: gpa_text = gpa_header_text(gpa) (R40, reused); counts from the cards.
#[allow(dead_code, unused_variables)]
pub fn progress_view(gpa: &GpaSummary, cards: Vec<ProgressCardView>) -> ProgressView {
    todo!("M13: R61")
}
