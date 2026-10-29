//! View model: pure formatting, parsing, and validation for the desktop UI (docs/spec.md §2).
//!
//! No Slint types, no clock, no database, no file access. Every function here is unit tested
//! in `viewmodel/tests.rs`. Contract: `specs/004-m4-desktop-ui/contracts/viewmodel.md`.

use std::collections::BTreeMap;
use std::path::Path;

use chrono::{Datelike, Days, Duration, NaiveDate, NaiveDateTime, NaiveTime, Weekday};
use planner_core::{
    Assessment, AssessmentFilter, AssessmentInput, AssessmentKind, AttendanceSummary, Course,
    CourseInput, Dashboard, Holiday, MakeupInput, SessionView, Settings, Slot, SlotInput, Status,
    TermInput,
};

#[cfg(test)]
mod tests;

/// Shown for a missing value (R13: never `0%` or `NaN`).
const DASH: &str = "\u{2014}";
/// FR-007: the error states the expected date format.
const DATE_FORMAT_ERR: &str = "Use DD-MM-YYYY, e.g. 29-10-2026";
/// FR-007: the error states the expected time format.
const TIME_FORMAT_ERR: &str = "Use HH:MM, e.g. 09:30";
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

/// R13: percentage with one decimal; `None` (nothing attended or absent) shows an em dash.
pub fn format_pct(pct: Option<f64>) -> String {
    match pct {
        Some(p) if p.is_finite() => format!("{p:.1}%"),
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

/// `09:05` (seconds dropped).
pub fn format_time(t: NaiveTime) -> String {
    t.format("%H:%M").to_string()
}

/// `09:00–10:30` (en dash, no spaces).
pub fn format_time_range(start: NaiveTime, end: NaiveTime) -> String {
    format!("{}\u{2013}{}", format_time(start), format_time(end))
}

/// `October 30, 2026, 23:59` (long date, comma, 24-hour time).
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

/// Strict `HH:MM` after trimming (FR-007).
pub fn parse_time(text: &str) -> Result<NaiveTime, String> {
    let s = text.trim();
    let b = s.as_bytes();
    let shape_ok = b.len() == 5
        && b.iter().enumerate().all(|(i, c)| match i {
            2 => *c == b':',
            _ => c.is_ascii_digit(),
        });
    if !shape_ok {
        return Err(TIME_FORMAT_ERR.to_string());
    }
    let hour: u32 = s[0..2].parse().map_err(|_| TIME_FORMAT_ERR.to_string())?;
    let minute: u32 = s[3..5].parse().map_err(|_| TIME_FORMAT_ERR.to_string())?;
    NaiveTime::from_hms_opt(hour, minute, 0).ok_or_else(|| TIME_FORMAT_ERR.to_string())
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

/// Checks `target`. R2 checks (empty code or name, range) belong to core.
pub fn parse_course_form(
    code: &str,
    name: &str,
    instructor: &str,
    room: &str,
    notes: &str,
    target: &str,
) -> Result<CourseInput, FormErrors> {
    let mut errors = FormErrors::default();
    match errors.check("target", parse_target(target)) {
        Some(min_attendance_pct) => Ok(CourseInput {
            code: code.trim().to_string(),
            name: name.trim().to_string(),
            instructor: optional_text(instructor),
            room: optional_text(room),
            notes: optional_text(notes),
            min_attendance_pct,
        }),
        None => Err(errors),
    }
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
    let mut errors = FormErrors::default();
    let course_id = errors.check(
        "course",
        course_id.ok_or_else(|| CHOOSE_COURSE_ERR.to_string()),
    );
    let weekday = errors.check(
        "weekday",
        weekday_from_index(weekday).ok_or_else(|| "Choose a weekday".to_string()),
    );
    let start_time = errors.check("start", parse_time(start));
    let end_time = errors.check("end", parse_time(end));
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
    start: &str,
    end: &str,
    room: &str,
    note: &str,
) -> Result<MakeupInput, FormErrors> {
    let mut errors = FormErrors::default();
    let date = errors.check("date", parse_date(date));
    let start_time = errors.check("start", parse_time(start));
    let end_time = errors.check("end", parse_time(end));
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
    let time = errors.check("due_time", parse_time(due_time));
    let remind = errors.check("remind", parse_optional_minutes(remind));
    match (course_id, kind, date, time, remind) {
        (Some(course_id), Some(kind), Some(date), Some(time), Some(remind_minutes_before)) => {
            Ok(AssessmentInput {
                course_id,
                kind,
                title: title.trim().to_string(),
                due_at: date.and_time(time),
                notes: optional_text(notes),
                remind_minutes_before,
            })
        }
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

/// Checks `reminder` and `window` via [`parse_count`].
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
    }
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
pub fn error_text(e: &planner_core::Error) -> String {
    match e {
        planner_core::Error::Validation(msg) => {
            let mut chars = msg.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
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

/// Message for a file that could not be read or written.
pub fn file_error_message(path: &Path, cause: &str) -> String {
    format!("Could not access {}: {cause}", path.display())
}

/// R22: message when the pre-restore copy fails; the restore is not attempted.
pub fn pre_restore_failed_message(cause: &str) -> String {
    format!("Restore not started: could not copy the current data ({cause})")
}
