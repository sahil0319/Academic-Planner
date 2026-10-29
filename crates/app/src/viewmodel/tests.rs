//! Unit tests for the view model (contracts/viewmodel.md). No database: core types are built
//! directly; each test checks one formatting or mapping function against its contract.

use std::path::PathBuf;

use super::*;
use planner_core::Error;

const DATE_ERR: &str = "Use DD-MM-YYYY, e.g. 29-10-2026";
const TIME_ERR: &str = "Use HH:MM, e.g. 09:30";
const HOLIDAY_RANGE_ERR: &str = "End date must be on or after the start date";

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

fn t(h: u32, m: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(h, m, 0).unwrap()
}

fn dt(y: i32, m: u32, day: u32, h: u32, min: u32) -> NaiveDateTime {
    d(y, m, day).and_time(t(h, min))
}

fn holiday(date: NaiveDate, label: Option<&str>) -> Holiday {
    Holiday {
        date,
        label: label.map(str::to_string),
    }
}

fn session_view(
    id: i64,
    date: NaiveDate,
    status: Status,
    display_status: &str,
    is_past: bool,
) -> SessionView {
    SessionView {
        session: planner_core::Session {
            id,
            course_id: 1,
            slot_id: Some(1),
            kind: planner_core::SessionKind::Regular,
            date,
            start_time: t(9, 0),
            end_time: t(10, 30),
            room: Some("B12".to_string()),
            status,
            replaces_session_id: None,
            note: None,
        },
        course_code: "CSE101".to_string(),
        course_name: "Intro to Programming".to_string(),
        display_status: display_status.to_string(),
        is_past,
    }
}

fn pending_view(id: i64, date: NaiveDate) -> SessionView {
    session_view(id, date, Status::Pending, "Pending", false)
}

fn course(id: i64, code: &str) -> Course {
    Course {
        id,
        term_id: 1,
        code: code.to_string(),
        name: "Intro to Programming".to_string(),
        instructor: Some("Dr. Rahman".to_string()),
        room: Some("B12".to_string()),
        notes: None,
        min_attendance_pct: Some(75),
        archived: false,
    }
}

fn summary(pct: Option<f64>, target: Option<i64>, below: bool) -> AttendanceSummary {
    AttendanceSummary {
        course_id: 1,
        course_code: "CSE101".to_string(),
        course_name: "Intro to Programming".to_string(),
        attended: 2,
        absent: 1,
        cancelled: 3,
        pending: 4,
        pct,
        target,
        below_target: below,
    }
}

fn assessment(id: i64, title: &str) -> Assessment {
    Assessment {
        id,
        course_id: 1,
        course_code: "CSE101".to_string(),
        kind: AssessmentKind::Exam,
        title: title.to_string(),
        due_at: dt(2026, 10, 30, 23, 59),
        notes: Some("Chapters 1-3".to_string()),
        completed: false,
        remind_minutes_before: None,
    }
}

fn slot(valid_to: Option<NaiveDate>, room: Option<&str>) -> Slot {
    Slot {
        id: 7,
        course_id: 1,
        weekday: Weekday::Tue,
        start_time: t(9, 0),
        end_time: t(10, 30),
        room: room.map(str::to_string),
        valid_from: d(2026, 9, 1),
        valid_to,
    }
}

// ---- Status and percentages ----

/// R11: chip kind is one-to-one with status.
#[test]
fn chip_kind_is_one_to_one_with_status() {
    assert_eq!(chip_kind(Status::Pending), ChipKind::Pending);
    assert_eq!(chip_kind(Status::Attended), ChipKind::Attended);
    assert_eq!(chip_kind(Status::Absent), ChipKind::Absent);
    assert_eq!(chip_kind(Status::Cancelled), ChipKind::Cancelled);
}

/// R14: the badge text is fixed.
#[test]
fn below_target_label_text() {
    assert_eq!(BELOW_TARGET_LABEL, "Below target");
}

/// R13: one decimal, percent sign.
#[test]
fn format_pct_one_decimal() {
    assert_eq!(format_pct(Some(50.0)), "50.0%");
    assert_eq!(format_pct(Some(100.0)), "100.0%");
    assert_eq!(format_pct(Some(0.0)), "0.0%");
}

/// R13: rounds to one decimal.
#[test]
fn format_pct_rounds_two_thirds() {
    assert_eq!(format_pct(Some(200.0 / 3.0)), "66.7%");
}

/// R13: no attended or absent sessions shows an em dash, never `0%` or `NaN`.
#[test]
fn format_pct_none_shows_em_dash() {
    assert_eq!(format_pct(None), "—");
}

/// R14: target text.
#[test]
fn format_target_some_and_none() {
    assert_eq!(format_target(Some(75)), "75%");
    assert_eq!(format_target(None), "—");
}

/// Missing or blank optional text shows an em dash; other text is trimmed.
#[test]
fn or_dash_handles_none_blank_and_text() {
    assert_eq!(or_dash(None), "—");
    assert_eq!(or_dash(Some("")), "—");
    assert_eq!(or_dash(Some("   ")), "—");
    assert_eq!(or_dash(Some("  B12 ")), "B12");
}

/// R12: attendance can be marked only for past sessions.
#[test]
fn can_mark_attendance_follows_is_past() {
    let past = session_view(1, d(2026, 10, 26), Status::Pending, "Pending", true);
    let future = session_view(2, d(2026, 10, 30), Status::Pending, "Pending", false);
    assert!(can_mark_attendance(&past));
    assert!(!can_mark_attendance(&future));
}

// ---- Dates and times ----

/// §6 date format change: dates display in the long form, day not zero-padded.
#[test]
fn format_date_long_form() {
    assert_eq!(format_date(d(2026, 10, 29)), "October 29, 2026");
    assert_eq!(format_date(d(2026, 1, 5)), "January 5, 2026");
}

/// §6 date format change: pre-filled date fields show zero-padded `DD-MM-YYYY`.
#[test]
fn format_date_input_is_zero_padded_dd_mm_yyyy() {
    assert_eq!(format_date_input(d(2026, 10, 26)), "26-10-2026");
    assert_eq!(format_date_input(d(2026, 1, 5)), "05-01-2026");
}

/// §6 date format change: the pre-fill text parses back to the same date.
#[test]
fn format_date_input_round_trips_through_parse_date() {
    for date in [d(2026, 1, 5), d(2026, 10, 29), d(2027, 2, 28)] {
        assert_eq!(parse_date(&format_date_input(date)), Ok(date));
    }
}

/// Times print as HH:MM, seconds dropped.
#[test]
fn format_time_drops_seconds() {
    assert_eq!(format_time(t(9, 5)), "09:05");
    let with_seconds = NaiveTime::from_hms_opt(9, 5, 42).unwrap();
    assert_eq!(format_time(with_seconds), "09:05");
}

/// Time ranges use an en dash and no spaces.
#[test]
fn format_time_range_uses_en_dash() {
    assert_eq!(format_time_range(t(9, 0), t(10, 30)), "09:00–10:30");
}

/// §6 date format change: date and time print as `Month D, YYYY, HH:MM` (24-hour).
#[test]
fn format_datetime_date_and_time() {
    assert_eq!(
        format_datetime(dt(2026, 10, 30, 23, 59)),
        "October 30, 2026, 23:59"
    );
}

/// Weekday names are full English names.
#[test]
fn weekday_name_all_days() {
    let names = [
        (Weekday::Mon, "Monday"),
        (Weekday::Tue, "Tuesday"),
        (Weekday::Wed, "Wednesday"),
        (Weekday::Thu, "Thursday"),
        (Weekday::Fri, "Friday"),
        (Weekday::Sat, "Saturday"),
        (Weekday::Sun, "Sunday"),
    ];
    for (w, name) in names {
        assert_eq!(weekday_name(w), name);
    }
}

/// Combo index 0 is Monday, 6 is Sunday; anything else is `None`.
#[test]
fn weekday_from_index_bounds() {
    assert_eq!(weekday_from_index(0), Some(Weekday::Mon));
    assert_eq!(weekday_from_index(6), Some(Weekday::Sun));
    assert_eq!(weekday_from_index(-1), None);
    assert_eq!(weekday_from_index(7), None);
}

/// `weekday_index` inverts `weekday_from_index` for every day.
#[test]
fn weekday_index_round_trips() {
    for i in 0..7 {
        let w = weekday_from_index(i).unwrap();
        assert_eq!(weekday_index(w), i);
    }
}

/// FR-007, §6 date format change: `DD-MM-YYYY` parses, with surrounding whitespace trimmed.
#[test]
fn parse_date_accepts_dd_mm_yyyy_and_trims() {
    assert_eq!(parse_date("29-10-2026"), Ok(d(2026, 10, 29)));
    assert_eq!(parse_date("  29-10-2026 "), Ok(d(2026, 10, 29)));
}

/// §6 date format change: day and month may have one digit or two.
#[test]
fn parse_date_accepts_one_digit_day_and_month() {
    assert_eq!(parse_date("5-1-2026"), Ok(d(2026, 1, 5)));
    assert_eq!(parse_date("05-01-2026"), Ok(d(2026, 1, 5)));
    assert_eq!(parse_date("5-01-2026"), Ok(d(2026, 1, 5)));
}

/// FR-007, §6 date format change: bad dates (including the old ISO form and a 2-digit year)
/// give one error text that states the format.
#[test]
fn parse_date_rejects_bad_input_with_format_hint() {
    for bad in [
        "",
        "   ",
        "2026-10-29",
        "29/10/2026",
        "30-02-2026",
        "29-10-26",
        "abc",
    ] {
        assert_eq!(parse_date(bad), Err(DATE_ERR.to_string()), "input {bad:?}");
    }
}

/// Optional date: blank is `Ok(None)`.
#[test]
fn parse_optional_date_blank_is_none() {
    assert_eq!(parse_optional_date(""), Ok(None));
    assert_eq!(parse_optional_date("   "), Ok(None));
}

/// Optional date: otherwise as `parse_date`.
#[test]
fn parse_optional_date_parses_or_errors() {
    assert_eq!(parse_optional_date("20-12-2026"), Ok(Some(d(2026, 12, 20))));
    assert_eq!(parse_optional_date("20/12/2026"), Err(DATE_ERR.to_string()));
}

/// FR-007: valid times parse.
#[test]
fn parse_time_accepts_hh_mm() {
    assert_eq!(parse_time("09:30"), Ok(t(9, 30)));
    assert_eq!(parse_time("00:00"), Ok(t(0, 0)));
    assert_eq!(parse_time("23:59"), Ok(t(23, 59)));
    assert_eq!(parse_time(" 09:30 "), Ok(t(9, 30)));
}

/// FR-007: bad times give one error text that states the format.
#[test]
fn parse_time_rejects_bad_input_with_format_hint() {
    for bad in ["", "9:30", "24:00", "12:60", "0930"] {
        assert_eq!(parse_time(bad), Err(TIME_ERR.to_string()), "input {bad:?}");
    }
}

// ---- Text and numbers ----

/// Optional fields are stored as absent when empty, trimmed otherwise.
#[test]
fn optional_text_trims_and_maps_empty_to_none() {
    assert_eq!(optional_text(""), None);
    assert_eq!(optional_text("   "), None);
    assert_eq!(optional_text("  B12 "), Some("B12".to_string()));
}

/// Target: empty is none, whole numbers pass, range is left to core (R2).
#[test]
fn parse_target_accepts_whole_numbers_without_range_check() {
    assert_eq!(parse_target(""), Ok(None));
    assert_eq!(parse_target("75"), Ok(Some(75)));
    assert_eq!(parse_target("-5"), Ok(Some(-5)));
    assert_eq!(parse_target("150"), Ok(Some(150)));
}

/// Target: non-integers are rejected.
#[test]
fn parse_target_rejects_non_integers() {
    assert_eq!(parse_target("7.5"), Err("Enter a whole number".to_string()));
    assert_eq!(parse_target("abc"), Err("Enter a whole number".to_string()));
}

/// Minutes: empty is none, non-negative integers pass.
#[test]
fn parse_optional_minutes_accepts_empty_and_non_negative() {
    assert_eq!(parse_optional_minutes(""), Ok(None));
    assert_eq!(parse_optional_minutes("30"), Ok(Some(30)));
    assert_eq!(parse_optional_minutes("0"), Ok(Some(0)));
}

/// Minutes: negatives and text are rejected.
#[test]
fn parse_optional_minutes_rejects_negative_and_text() {
    let msg = "Enter a whole number of minutes".to_string();
    assert_eq!(parse_optional_minutes("-1"), Err(msg.clone()));
    assert_eq!(parse_optional_minutes("x"), Err(msg));
}

/// Count: required non-negative whole number.
#[test]
fn parse_count_required_and_whole() {
    assert_eq!(parse_count("15"), Ok(15));
    assert_eq!(parse_count("0"), Ok(0));
    assert_eq!(parse_count(""), Err("Required".to_string()));
    assert_eq!(parse_count("-1"), Err("Enter a whole number".to_string()));
    assert_eq!(parse_count("x"), Err("Enter a whole number".to_string()));
}

// ---- Forms ----

/// `FormErrors::get` returns the message, or `""` for a field without an error.
#[test]
fn form_errors_get_and_is_empty() {
    let none = FormErrors::default();
    assert!(none.is_empty());
    assert_eq!(none.get("start"), "");

    let some = FormErrors {
        errors: vec![("start".to_string(), "bad".to_string())],
    };
    assert!(!some.is_empty());
    assert_eq!(some.get("start"), "bad");
    assert_eq!(some.get("end"), "");
}

/// Term form: valid input builds a `TermInput` with a trimmed name.
#[test]
fn parse_term_form_valid() {
    let input = parse_term_form(Some(3), "  Fall 2026 ", "01-09-2026", "20-12-2026").unwrap();
    assert_eq!(
        input,
        TermInput {
            id: Some(3),
            name: "Fall 2026".to_string(),
            start_date: d(2026, 9, 1),
            end_date: d(2026, 12, 20),
        }
    );
}

/// Term form: both date errors appear at once (FR-007).
#[test]
fn parse_term_form_reports_all_date_errors() {
    let err = parse_term_form(None, "Fall", "1/9/2026", "").unwrap_err();
    assert_eq!(err.get("start"), DATE_ERR);
    assert_eq!(err.get("end"), DATE_ERR);
}

/// R2: end before start is core's check, so the form passes it through.
#[test]
fn parse_term_form_passes_end_before_start_to_core() {
    let input = parse_term_form(None, "Fall", "20-12-2026", "01-09-2026").unwrap();
    assert_eq!(input.start_date, d(2026, 12, 20));
    assert_eq!(input.end_date, d(2026, 9, 1));
}

/// Course form: trims code and name, maps empty optional fields to `None`.
#[test]
fn parse_course_form_valid() {
    let input = parse_course_form(" CSE101 ", " Intro ", "Dr. Rahman", "  ", "", "75").unwrap();
    assert_eq!(
        input,
        CourseInput {
            code: "CSE101".to_string(),
            name: "Intro".to_string(),
            instructor: Some("Dr. Rahman".to_string()),
            room: None,
            notes: None,
            min_attendance_pct: Some(75),
        }
    );
}

/// Course form: an empty target means no target.
#[test]
fn parse_course_form_empty_target_is_none() {
    let input = parse_course_form("CSE101", "Intro", "", "", "", "").unwrap();
    assert_eq!(input.min_attendance_pct, None);
}

/// Course form: a non-integer target reports a field error.
#[test]
fn parse_course_form_bad_target_reports_field_error() {
    let err = parse_course_form("CSE101", "Intro", "", "", "", "7.5").unwrap_err();
    assert_eq!(err.get("target"), "Enter a whole number");
    assert_eq!(err.get("code"), "");
}

/// R2: empty code and out-of-range target are core's checks; the form passes them through.
#[test]
fn parse_course_form_passes_r2_checks_to_core() {
    let input = parse_course_form("", "", "", "", "", "150").unwrap();
    assert_eq!(input.code, "");
    assert_eq!(input.name, "");
    assert_eq!(input.min_attendance_pct, Some(150));
}

/// Slot form: valid input builds a `SlotInput`.
#[test]
fn parse_slot_form_valid() {
    let input = parse_slot_form(
        Some(1),
        1,
        "09:00",
        "10:30",
        " B12 ",
        "01-09-2026",
        "20-12-2026",
    )
    .unwrap();
    assert_eq!(
        input,
        SlotInput {
            course_id: 1,
            weekday: Weekday::Tue,
            start_time: t(9, 0),
            end_time: t(10, 30),
            room: Some("B12".to_string()),
            valid_from: d(2026, 9, 1),
            valid_to: Some(d(2026, 12, 20)),
        }
    );
}

/// Slot form: empty valid-to means until term end.
#[test]
fn parse_slot_form_empty_valid_to_is_none() {
    let input = parse_slot_form(Some(1), 0, "09:00", "10:30", "", "01-09-2026", "").unwrap();
    assert_eq!(input.valid_to, None);
    assert_eq!(input.room, None);
}

/// Slot form: every bad field is reported at once (FR-007).
#[test]
fn parse_slot_form_reports_all_errors_at_once() {
    let err = parse_slot_form(None, 9, "9:00", "25:00", "", "x", "y").unwrap_err();
    assert_eq!(err.get("course"), "Choose a course");
    assert_eq!(err.get("weekday"), "Choose a weekday");
    assert_eq!(err.get("start"), TIME_ERR);
    assert_eq!(err.get("end"), TIME_ERR);
    assert_eq!(err.get("valid_from"), DATE_ERR);
    assert_eq!(err.get("valid_to"), DATE_ERR);
}

/// Slot form: valid-from is required.
#[test]
fn parse_slot_form_requires_valid_from() {
    let err = parse_slot_form(Some(1), 0, "09:00", "10:30", "", "", "").unwrap_err();
    assert_eq!(err.get("valid_from"), DATE_ERR);
    assert_eq!(err.get("valid_to"), "");
}

/// R2: slot end not after start is core's check; the form passes it through.
#[test]
fn parse_slot_form_passes_end_before_start_to_core() {
    let input = parse_slot_form(Some(1), 0, "10:30", "09:00", "", "01-09-2026", "").unwrap();
    assert_eq!(input.start_time, t(10, 30));
    assert_eq!(input.end_time, t(9, 0));
}

/// R9, R10: makeup form builds a `MakeupInput`; the note is optional.
#[test]
fn parse_makeup_form_valid() {
    let input = parse_makeup_form(4, "15-01-2026", "14:00", "15:30", "C3", " moved ").unwrap();
    assert_eq!(
        input,
        MakeupInput {
            course_id: 4,
            date: d(2026, 1, 15),
            start_time: t(14, 0),
            end_time: t(15, 30),
            room: Some("C3".to_string()),
            note: Some("moved".to_string()),
        }
    );
}

/// R9, R10: empty room and note become `None`.
#[test]
fn parse_makeup_form_empty_optional_fields_are_none() {
    let input = parse_makeup_form(4, "15-01-2026", "14:00", "15:30", "", "").unwrap();
    assert_eq!(input.room, None);
    assert_eq!(input.note, None);
}

/// R9, R10: all three required fields report errors at once.
#[test]
fn parse_makeup_form_reports_all_errors_at_once() {
    let err = parse_makeup_form(4, "", "2pm", "", "", "").unwrap_err();
    assert_eq!(err.get("date"), DATE_ERR);
    assert_eq!(err.get("start"), TIME_ERR);
    assert_eq!(err.get("end"), TIME_ERR);
}

/// Assessment form: `due_at` is date plus time; remind minutes parse.
#[test]
fn parse_assessment_form_valid() {
    let input =
        parse_assessment_form(Some(1), 1, " Midterm ", "30-10-2026", "23:59", "", "60").unwrap();
    assert_eq!(
        input,
        AssessmentInput {
            course_id: 1,
            kind: AssessmentKind::Exam,
            title: "Midterm".to_string(),
            due_at: dt(2026, 10, 30, 23, 59),
            notes: None,
            remind_minutes_before: Some(60),
        }
    );
}

/// Assessment form: empty remind means no reminder.
#[test]
fn parse_assessment_form_empty_remind_is_none() {
    let input = parse_assessment_form(Some(1), 0, "Quiz 1", "30-10-2026", "09:00", "", "").unwrap();
    assert_eq!(input.remind_minutes_before, None);
    assert_eq!(input.kind, AssessmentKind::Quiz);
}

/// Assessment form: every bad field is reported at once (FR-007).
#[test]
fn parse_assessment_form_reports_all_errors_at_once() {
    let err = parse_assessment_form(None, 9, "", "30/10", "noon", "", "-5").unwrap_err();
    assert_eq!(err.get("course"), "Choose a course");
    assert_eq!(err.get("kind"), "Choose a kind");
    assert_eq!(err.get("due_date"), DATE_ERR);
    assert_eq!(err.get("due_time"), TIME_ERR);
    assert_eq!(err.get("remind"), "Enter a whole number of minutes");
}

/// R2: an empty title is core's check; the form passes it through.
#[test]
fn parse_assessment_form_passes_empty_title_to_core() {
    let input = parse_assessment_form(Some(1), 3, "", "30-10-2026", "09:00", "", "").unwrap();
    assert_eq!(input.title, "");
}

/// Holiday form: a blank end date gives a single holiday with an optional label.
#[test]
fn parse_holiday_form_single_day() {
    let got = parse_holiday_form("25-12-2026", "", " Christmas ").unwrap();
    assert_eq!(got, vec![holiday(d(2026, 12, 25), Some("Christmas"))]);
}

/// Holiday form: a range expands to one holiday per day.
#[test]
fn parse_holiday_form_range_expands() {
    let got = parse_holiday_form("21-12-2026", "25-12-2026", "Break").unwrap();
    assert_eq!(got.len(), 5);
    assert_eq!(got[0].date, d(2026, 12, 21));
    assert_eq!(got[4].date, d(2026, 12, 25));
}

/// Holiday form: bad dates report both fields at once.
#[test]
fn parse_holiday_form_reports_date_errors() {
    let err = parse_holiday_form("x", "y", "").unwrap_err();
    assert_eq!(err.get("start"), DATE_ERR);
    assert_eq!(err.get("end"), DATE_ERR);
}

/// Holiday form: `end < start` is the view model's own check (human decision, §6). The error is
/// attached to the `end` field.
#[test]
fn parse_holiday_form_end_before_start_errors_on_end() {
    let err = parse_holiday_form("25-12-2026", "21-12-2026", "").unwrap_err();
    assert_eq!(err.get("end"), HOLIDAY_RANGE_ERR);
    assert_eq!(err.get("start"), "");
}

/// Settings form: valid numbers build `Settings`.
#[test]
fn parse_settings_form_valid() {
    let got = parse_settings_form("15", "7", false).unwrap();
    assert_eq!(
        got,
        Settings {
            class_reminder_minutes: 15,
            upcoming_window_days: 7,
            notifications_enabled: false,
        }
    );
}

/// Settings form: both counts are checked and reported at once.
#[test]
fn parse_settings_form_reports_both_errors() {
    let err = parse_settings_form("", "x", true).unwrap_err();
    assert_eq!(err.get("reminder"), "Required");
    assert_eq!(err.get("window"), "Enter a whole number");
}

// ---- Holidays ----

/// §6: no end date gives one holiday.
#[test]
fn expand_holiday_range_single_day() {
    let got = expand_holiday_range(d(2026, 12, 25), None, Some("Christmas".to_string())).unwrap();
    assert_eq!(got, vec![holiday(d(2026, 12, 25), Some("Christmas"))]);
}

/// §6: end equal to start gives one holiday.
#[test]
fn expand_holiday_range_end_equals_start_is_single_day() {
    let got = expand_holiday_range(d(2026, 12, 25), Some(d(2026, 12, 25)), None).unwrap();
    assert_eq!(got, vec![holiday(d(2026, 12, 25), None)]);
}

/// §6: a 5-day inclusive range gives five holidays, ascending, one label.
#[test]
fn expand_holiday_range_five_days_inclusive_same_label() {
    let got = expand_holiday_range(
        d(2026, 12, 21),
        Some(d(2026, 12, 25)),
        Some("Winter break".to_string()),
    )
    .unwrap();
    let dates: Vec<NaiveDate> = got.iter().map(|h| h.date).collect();
    assert_eq!(
        dates,
        vec![
            d(2026, 12, 21),
            d(2026, 12, 22),
            d(2026, 12, 23),
            d(2026, 12, 24),
            d(2026, 12, 25)
        ]
    );
    assert!(got
        .iter()
        .all(|h| h.label.as_deref() == Some("Winter break")));
}

/// §6: end before start is an error.
#[test]
fn expand_holiday_range_end_before_start_errors() {
    let got = expand_holiday_range(d(2026, 12, 25), Some(d(2026, 12, 21)), None);
    assert_eq!(got, Err(HOLIDAY_RANGE_ERR.to_string()));
}

/// §6: overlapping dates keep one holiday per date; the added label wins; result is sorted.
#[test]
fn merge_holidays_overlap_added_label_wins_and_sorted() {
    let existing = vec![
        holiday(d(2026, 12, 24), Some("Old")),
        holiday(d(2026, 11, 2), Some("Other")),
    ];
    let added = vec![
        holiday(d(2026, 12, 25), Some("Break")),
        holiday(d(2026, 12, 24), Some("Break")),
    ];
    let got = merge_holidays(&existing, &added);
    assert_eq!(
        got,
        vec![
            holiday(d(2026, 11, 2), Some("Other")),
            holiday(d(2026, 12, 24), Some("Break")),
            holiday(d(2026, 12, 25), Some("Break")),
        ]
    );
}

/// §6: merging nothing keeps the existing list, sorted.
#[test]
fn merge_holidays_with_nothing_added_keeps_existing_sorted() {
    let existing = vec![
        holiday(d(2026, 12, 25), None),
        holiday(d(2026, 11, 2), None),
    ];
    let got = merge_holidays(&existing, &[]);
    assert_eq!(
        got,
        vec![
            holiday(d(2026, 11, 2), None),
            holiday(d(2026, 12, 25), None)
        ]
    );
}

/// §6: removing a date leaves the other dates unchanged.
#[test]
fn remove_holiday_removes_only_that_date() {
    let existing = vec![
        holiday(d(2026, 12, 23), Some("A")),
        holiday(d(2026, 12, 24), Some("A")),
        holiday(d(2026, 12, 25), Some("A")),
    ];
    let got = remove_holiday(&existing, d(2026, 12, 24));
    assert_eq!(
        got,
        vec![
            holiday(d(2026, 12, 23), Some("A")),
            holiday(d(2026, 12, 25), Some("A"))
        ]
    );
}

/// §6: removing an absent date changes nothing.
#[test]
fn remove_holiday_absent_date_is_noop() {
    let existing = vec![holiday(d(2026, 12, 25), None)];
    assert_eq!(remove_holiday(&existing, d(2026, 1, 1)), existing);
}

/// Holiday rows show an em dash for a missing label.
#[test]
fn holiday_row_label_and_dash() {
    assert_eq!(
        holiday_row(&holiday(d(2026, 12, 25), Some("Christmas"))),
        HolidayRow {
            key: "2026-12-25".to_string(),
            date: "December 25, 2026".to_string(),
            label: "Christmas".to_string()
        }
    );
    assert_eq!(holiday_row(&holiday(d(2026, 12, 25), None)).label, "—");
}

// ---- Rows ----

/// §6 date format change: the trailing ISO date of a rescheduled label becomes a long date.
#[test]
fn display_status_text_rewrites_trailing_iso_date() {
    assert_eq!(
        display_status_text("Rescheduled → 2026-01-13"),
        "Rescheduled → January 13, 2026"
    );
}

/// §6 date format change: labels without a date are unchanged.
#[test]
fn display_status_text_leaves_other_labels_unchanged() {
    for label in ["Pending", "Attended", "Absent", "Cancelled", ""] {
        assert_eq!(display_status_text(label), label);
    }
}

/// R11, R12: a regular session row.
#[test]
fn session_row_for_past_pending_session() {
    let mut view = session_view(5, d(2026, 10, 26), Status::Pending, "Pending", true);
    view.session.note = Some("bring laptop".to_string());
    let row = session_row(&view);
    assert_eq!(
        row,
        SessionRow {
            id: 5,
            date: "October 26, 2026".to_string(),
            weekday: "Monday".to_string(),
            time: "09:00–10:30".to_string(),
            course: "CSE101 · Intro to Programming".to_string(),
            room: "B12".to_string(),
            status_label: "Pending".to_string(),
            chip: ChipKind::Pending,
            can_mark: true,
            note: "bring laptop".to_string(),
        }
    );
}

/// R12: a future session cannot be marked; a missing room and note show as dash and empty.
#[test]
fn session_row_future_session_missing_room_and_note() {
    let mut view = pending_view(6, d(2026, 10, 30));
    view.session.room = None;
    let row = session_row(&view);
    assert!(!row.can_mark);
    assert_eq!(row.room, "—");
    assert_eq!(row.note, "");
}

/// R9, §6 date format change: a rescheduled original shows core's text with a long date and
/// the Cancelled chip.
#[test]
fn session_row_rescheduled_original_keeps_display_status() {
    let view = session_view(
        7,
        d(2026, 1, 12),
        Status::Cancelled,
        "Rescheduled → 2026-01-15",
        true,
    );
    let row = session_row(&view);
    assert_eq!(row.status_label, "Rescheduled → January 15, 2026");
    assert_eq!(row.chip, ChipKind::Cancelled);
}

/// R13–R15: counts as text, percentage and target formatted.
#[test]
fn summary_view_formats_counts_pct_and_target() {
    let got = summary_view(&summary(Some(200.0 / 3.0), Some(75), true));
    assert_eq!(
        got,
        SummaryView {
            attended: "2".to_string(),
            absent: "1".to_string(),
            cancelled: "3".to_string(),
            pending: "4".to_string(),
            pct: "66.7%".to_string(),
            target: "75%".to_string(),
            below_target: true,
        }
    );
}

/// R13, R14: no percentage and no target show dashes.
#[test]
fn summary_view_without_pct_or_target_shows_dashes() {
    let got = summary_view(&summary(None, None, false));
    assert_eq!(got.pct, "—");
    assert_eq!(got.target, "—");
    assert!(!got.below_target);
}

/// R14: banner line format.
#[test]
fn banner_line_format() {
    let got = banner_line(&summary(Some(200.0 / 3.0), Some(75), true));
    assert_eq!(got, "CSE101 — 66.7% (target 75%)");
}

/// R3: course row shows dashes for missing optional fields.
#[test]
fn course_row_dashes_for_missing_fields() {
    let mut c = course(3, "CSE101");
    c.instructor = None;
    c.room = None;
    c.min_attendance_pct = None;
    c.archived = true;
    assert_eq!(
        course_row(&c),
        CourseRow {
            id: 3,
            code: "CSE101".to_string(),
            name: "Intro to Programming".to_string(),
            instructor: "—".to_string(),
            room: "—".to_string(),
            target: "—".to_string(),
            archived: true,
        }
    );
}

/// Course row with every optional field set.
#[test]
fn course_row_with_all_fields() {
    let row = course_row(&course(3, "CSE101"));
    assert_eq!(row.instructor, "Dr. Rahman");
    assert_eq!(row.room, "B12");
    assert_eq!(row.target, "75%");
    assert!(!row.archived);
}

/// Combo label is code, middle dot, name.
#[test]
fn course_option_label_format() {
    assert_eq!(
        course_option_label(&course(1, "CSE101")),
        "CSE101 · Intro to Programming"
    );
}

/// Slot row: valid range with an end date.
#[test]
fn slot_row_with_valid_to() {
    let courses = vec![course(2, "MAT200"), course(1, "CSE101")];
    let row = slot_row(&slot(Some(d(2026, 12, 20)), Some("C3")), &courses);
    assert_eq!(
        row,
        SlotRow {
            id: 7,
            weekday: "Tuesday".to_string(),
            time: "09:00–10:30".to_string(),
            course: "CSE101".to_string(),
            room: "C3".to_string(),
            valid: "From September 1, 2026 to December 20, 2026".to_string(),
        }
    );
}

/// Slot row: open-ended validity, missing room, and unknown course show as dashes.
#[test]
fn slot_row_until_term_end_and_unknown_course() {
    let row = slot_row(&slot(None, None), &[course(9, "PHY101")]);
    assert_eq!(row.valid, "From September 1, 2026 until term end");
    assert_eq!(row.room, "—");
    assert_eq!(row.course, "—");
}

/// Assessment row formats the due date and notes.
#[test]
fn assessment_row_formats_fields() {
    let row = assessment_row(&assessment(8, "Midterm"));
    assert_eq!(
        row,
        AssessmentRow {
            id: 8,
            course_code: "CSE101".to_string(),
            kind: "Exam".to_string(),
            title: "Midterm".to_string(),
            due: "October 30, 2026, 23:59".to_string(),
            completed: false,
            notes: "Chapters 1-3".to_string(),
        }
    );
}

/// Assessment row: missing notes become empty text.
#[test]
fn assessment_row_missing_notes_is_empty() {
    let mut a = assessment(8, "Midterm");
    a.notes = None;
    a.completed = true;
    let row = assessment_row(&a);
    assert_eq!(row.notes, "");
    assert!(row.completed);
}

/// Assessment kind labels.
#[test]
fn assessment_kind_label_all_kinds() {
    assert_eq!(assessment_kind_label(AssessmentKind::Quiz), "Quiz");
    assert_eq!(assessment_kind_label(AssessmentKind::Exam), "Exam");
    assert_eq!(
        assessment_kind_label(AssessmentKind::Assignment),
        "Assignment"
    );
    assert_eq!(assessment_kind_label(AssessmentKind::Other), "Other");
}

/// Assessment kind combo indices: 0 Quiz, 1 Exam, 2 Assignment, 3 Other.
#[test]
fn assessment_kind_index_mapping_and_round_trip() {
    assert_eq!(assessment_kind_from_index(0), Some(AssessmentKind::Quiz));
    assert_eq!(assessment_kind_from_index(1), Some(AssessmentKind::Exam));
    assert_eq!(
        assessment_kind_from_index(2),
        Some(AssessmentKind::Assignment)
    );
    assert_eq!(assessment_kind_from_index(3), Some(AssessmentKind::Other));
    assert_eq!(assessment_kind_from_index(4), None);
    assert_eq!(assessment_kind_from_index(-1), None);
    for i in 0..4 {
        assert_eq!(
            assessment_kind_index(assessment_kind_from_index(i).unwrap()),
            i
        );
    }
}

// ---- Assessment filters ----

/// Filter combo index mapping; out of range falls back to Upcoming.
#[test]
fn filter_from_index_mapping_and_fallback() {
    assert_eq!(filter_from_index(0), FilterChoice::Upcoming);
    assert_eq!(filter_from_index(1), FilterChoice::Overdue);
    assert_eq!(filter_from_index(2), FilterChoice::Completed);
    assert_eq!(filter_from_index(3), FilterChoice::All);
    assert_eq!(filter_from_index(4), FilterChoice::Upcoming);
    assert_eq!(filter_from_index(-1), FilterChoice::Upcoming);
}

/// R18, R19: Upcoming and Overdue carry `now`; Completed and All do not.
#[test]
fn assessment_filter_per_choice() {
    let now = dt(2026, 10, 29, 14, 30);
    assert_eq!(
        assessment_filter(FilterChoice::Upcoming, now),
        AssessmentFilter::Upcoming { now }
    );
    assert_eq!(
        assessment_filter(FilterChoice::Overdue, now),
        AssessmentFilter::Overdue { now }
    );
    assert_eq!(
        assessment_filter(FilterChoice::Completed, now),
        AssessmentFilter::Completed
    );
    assert_eq!(
        assessment_filter(FilterChoice::All, now),
        AssessmentFilter::All
    );
}

/// FR-005: the list title per filter choice is built in Rust, not in assessments.slint.
#[test]
fn filter_title_per_choice() {
    assert_eq!(filter_title(FilterChoice::Upcoming), "Upcoming");
    assert_eq!(filter_title(FilterChoice::Overdue), "Overdue");
    assert_eq!(filter_title(FilterChoice::Completed), "Completed");
    assert_eq!(filter_title(FilterChoice::All), "All assessments");
}

/// FR-005, T028 (DESIGN.md Layout): the empty-state sentence per filter choice says what will
/// appear and how to add it.
#[test]
fn filter_empty_text_per_choice() {
    assert_eq!(
        filter_empty_text(FilterChoice::Upcoming),
        "No upcoming assessments. Add one in the New assessment form below."
    );
    assert_eq!(
        filter_empty_text(FilterChoice::Overdue),
        "No overdue assessments. Incomplete assessments past their due date appear here."
    );
    assert_eq!(
        filter_empty_text(FilterChoice::Completed),
        "No completed assessments. Tick an assessment's checkbox to mark it complete."
    );
    assert_eq!(
        filter_empty_text(FilterChoice::All),
        "No assessments yet. Add one in the New assessment form below."
    );
}

// ---- Week view ----

/// Monday maps to itself.
#[test]
fn week_start_monday_is_itself() {
    assert_eq!(week_start(d(2026, 10, 26)), d(2026, 10, 26));
}

/// A mid-week day maps back to its Monday.
#[test]
fn week_start_mid_week() {
    assert_eq!(week_start(d(2026, 10, 29)), d(2026, 10, 26));
}

/// Sunday belongs to the week that started the Monday before.
#[test]
fn week_start_sunday_belongs_to_previous_monday() {
    assert_eq!(week_start(d(2026, 11, 1)), d(2026, 10, 26));
}

/// Week end is Sunday, six days after the start.
#[test]
fn week_end_is_sunday() {
    assert_eq!(week_end(d(2026, 10, 26)), d(2026, 11, 1));
}

/// Shifting works across a month end and a year end, forward and back.
#[test]
fn shift_week_across_month_and_year_end() {
    assert_eq!(shift_week(d(2026, 10, 26), 1), d(2026, 11, 2));
    assert_eq!(shift_week(d(2026, 10, 26), -1), d(2026, 10, 19));
    assert_eq!(shift_week(d(2026, 12, 28), 1), d(2027, 1, 4));
    assert_eq!(shift_week(d(2027, 1, 4), -1), d(2026, 12, 28));
    assert_eq!(shift_week(d(2026, 10, 26), 0), d(2026, 10, 26));
    assert_eq!(shift_week(d(2026, 10, 26), 2), d(2026, 11, 9));
}

/// §6 date format change: week title is a long-form range with the year once at the end.
#[test]
fn week_title_range() {
    assert_eq!(week_title(d(2026, 10, 26)), "October 26 – November 1, 2026");
}

/// §6 date format change: a week inside one month names the month once ("October 5 – 11, 2026").
#[test]
fn week_title_within_one_month_is_short() {
    assert_eq!(week_title(d(2026, 10, 5)), "October 5 – 11, 2026");
}

/// §6 date format change: a week spanning two years shows both years.
#[test]
fn week_title_across_years_shows_both_years() {
    assert_eq!(
        week_title(d(2026, 12, 28)),
        "December 28, 2026 – January 3, 2027"
    );
}

/// Empty input gives no groups (the page shows its empty state).
#[test]
fn group_week_empty_input_is_empty() {
    assert!(group_week(&[]).is_empty());
}

/// Groups follow date order with titles; rows inside a day keep input order.
#[test]
fn group_week_orders_days_and_keeps_row_order() {
    let views = vec![
        pending_view(1, d(2026, 10, 26)),
        pending_view(2, d(2026, 10, 26)),
        pending_view(3, d(2026, 10, 28)),
    ];
    let groups = group_week(&views);
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].title, "Monday, October 26, 2026");
    assert_eq!(
        groups[0].rows.iter().map(|r| r.id).collect::<Vec<_>>(),
        vec![1, 2]
    );
    assert_eq!(groups[1].title, "Wednesday, October 28, 2026");
    assert_eq!(groups[1].rows.len(), 1);
}

// ---- Dashboard ----

/// R14, R16–R20: keeps core's order and builds one banner line per below-target course.
#[test]
fn dashboard_view_keeps_order_and_builds_banner() {
    let dashboard = Dashboard {
        today: vec![
            pending_view(2, d(2026, 10, 29)),
            pending_view(1, d(2026, 10, 29)),
        ],
        next_class: Some(pending_view(3, d(2026, 10, 30))),
        upcoming: vec![assessment(20, "B"), assessment(10, "A")],
        overdue: vec![assessment(30, "C")],
        awaiting: vec![session_view(
            4,
            d(2026, 10, 28),
            Status::Pending,
            "Pending",
            true,
        )],
        below_target: vec![summary(Some(200.0 / 3.0), Some(75), true)],
    };
    let got = dashboard_view(&dashboard);
    assert_eq!(
        got.today.iter().map(|r| r.id).collect::<Vec<_>>(),
        vec![2, 1]
    );
    assert_eq!(got.next_class.map(|r| r.id), Some(3));
    assert_eq!(
        got.upcoming.iter().map(|r| r.id).collect::<Vec<_>>(),
        vec![20, 10]
    );
    assert_eq!(got.overdue.len(), 1);
    assert_eq!(got.awaiting.len(), 1);
    assert_eq!(got.banner, vec!["CSE101 — 66.7% (target 75%)".to_string()]);
}

/// R14: an empty dashboard has no banner and no next class.
#[test]
fn dashboard_view_empty_has_no_banner() {
    let dashboard = Dashboard {
        today: vec![],
        next_class: None,
        upcoming: vec![],
        overdue: vec![],
        awaiting: vec![],
        below_target: vec![],
    };
    let got = dashboard_view(&dashboard);
    assert!(got.banner.is_empty());
    assert!(got.next_class.is_none());
    assert!(got.today.is_empty());
}

/// R16, R17 (T028): the Next class card is hidden when that session is already in Today's
/// classes.
#[test]
fn show_next_card_hides_session_already_in_today() {
    let today = vec![
        session_row(&pending_view(1, d(2026, 10, 29))),
        session_row(&pending_view(2, d(2026, 10, 29))),
    ];
    let next = session_row(&pending_view(2, d(2026, 10, 29)));
    assert!(!show_next_card(Some(&next), &today));
}

/// R17 (T028): a next class on a later day keeps the card.
#[test]
fn show_next_card_keeps_session_not_in_today() {
    let today = vec![session_row(&pending_view(1, d(2026, 10, 29)))];
    let next = session_row(&pending_view(3, d(2026, 10, 30)));
    assert!(show_next_card(Some(&next), &today));
    assert!(show_next_card(Some(&next), &[]));
}

/// R17 (T028): with no next class the card stays to show its empty state.
#[test]
fn show_next_card_without_next_class_shows_empty_state() {
    let today = vec![session_row(&pending_view(1, d(2026, 10, 29)))];
    assert!(show_next_card(None, &today));
    assert!(show_next_card(None, &[]));
}

// ---- Errors, files, timer ----

/// R2: validation messages are capitalized.
#[test]
fn error_text_capitalizes_validation_message() {
    let e = Error::Validation("course code must not be empty".to_string());
    assert_eq!(error_text(&e), "Course code must not be empty");
}

/// R25: the schema-too-new text names both versions.
#[test]
fn error_text_schema_too_new_names_both_versions() {
    let e = Error::SchemaTooNew {
        found: 42,
        supported: 7,
    };
    let text = error_text(&e);
    assert_eq!(text, e.to_string());
    assert!(text.contains("42"));
    assert!(text.contains('7'));
}

/// Other error variants show their `Display` text.
#[test]
fn error_text_other_variants_use_display() {
    let e = Error::NotFound {
        entity: "course",
        id: 9,
    };
    assert_eq!(error_text(&e), e.to_string());
    assert_eq!(
        error_text(&Error::NoActiveTerm),
        Error::NoActiveTerm.to_string()
    );
}

/// R22: pre-restore name, seconds ignored.
#[test]
fn pre_restore_file_name_format() {
    let now = d(2026, 10, 29).and_hms_opt(14, 30, 59).unwrap();
    assert_eq!(
        pre_restore_file_name(now),
        "pre-restore-20261029-1430.sqlite"
    );
}

/// R22, Q2: no clash returns the base name.
#[test]
fn free_pre_restore_name_no_clash() {
    let now = dt(2026, 10, 29, 14, 30);
    assert_eq!(
        free_pre_restore_name(now, &[]),
        "pre-restore-20261029-1430.sqlite"
    );
}

/// R22, Q2: one clash gets suffix `-2`.
#[test]
fn free_pre_restore_name_one_clash_adds_2() {
    let now = dt(2026, 10, 29, 14, 30);
    let existing = vec!["pre-restore-20261029-1430.sqlite".to_string()];
    assert_eq!(
        free_pre_restore_name(now, &existing),
        "pre-restore-20261029-1430-2.sqlite"
    );
}

/// R22, Q2: two clashes get suffix `-3`.
#[test]
fn free_pre_restore_name_two_clashes_adds_3() {
    let now = dt(2026, 10, 29, 14, 30);
    let existing = vec![
        "pre-restore-20261029-1430.sqlite".to_string(),
        "pre-restore-20261029-1430-2.sqlite".to_string(),
    ];
    assert_eq!(
        free_pre_restore_name(now, &existing),
        "pre-restore-20261029-1430-3.sqlite"
    );
}

/// R22, Q2: unrelated names do not count as clashes.
#[test]
fn free_pre_restore_name_ignores_unrelated_names() {
    let now = dt(2026, 10, 29, 14, 30);
    let existing = vec![
        "pre-restore-20261029-1431.sqlite".to_string(),
        "notes.txt".to_string(),
        "academic-planner-backup-20261029-1430.json".to_string(),
    ];
    assert_eq!(
        free_pre_restore_name(now, &existing),
        "pre-restore-20261029-1430.sqlite"
    );
}

/// R21: backup file name.
#[test]
fn backup_file_name_format() {
    let now = dt(2026, 10, 29, 14, 30);
    assert_eq!(
        backup_file_name(now),
        "academic-planner-backup-20261029-1430.json"
    );
}

/// R6: still the same day at 23:59.
#[test]
fn date_rolled_over_same_day_is_false() {
    assert!(!date_rolled_over(d(2026, 10, 29), dt(2026, 10, 29, 23, 59)));
}

/// R6: midnight of the next day is a rollover.
#[test]
fn date_rolled_over_next_midnight_is_true() {
    assert!(date_rolled_over(d(2026, 10, 29), dt(2026, 10, 30, 0, 0)));
}

/// R24: the path is shown as its display text.
#[test]
fn db_path_text_is_display_text() {
    let path = PathBuf::from("/home/u/academic-planner.sqlite");
    assert_eq!(db_path_text(&path), "/home/u/academic-planner.sqlite");
}

/// R22: restore message names the saved copy.
#[test]
fn restored_message_names_copy_path() {
    let copy = PathBuf::from("/data/backups/pre-restore-20261029-1430.sqlite");
    assert_eq!(
        restored_message(&copy),
        "Backup restored. Previous data saved to /data/backups/pre-restore-20261029-1430.sqlite"
    );
}

// ---- Course form (T017) ----

/// R2: the course edit form shows a stored target as plain digits, so saving it unchanged
/// round-trips through `parse_target`.
#[test]
fn target_input_shows_digits_without_percent() {
    assert_eq!(target_input(Some(75)), "75");
    assert_eq!(parse_target(&target_input(Some(75))), Ok(Some(75)));
}

/// R2: no target leaves the edit field empty (not an em dash), which parses back to `None`.
#[test]
fn target_input_none_is_empty() {
    assert_eq!(target_input(None), "");
    assert_eq!(parse_target(&target_input(None)), Ok(None));
}

/// R21: the backup success message names the written file.
#[test]
fn backup_saved_message_names_file() {
    let path = PathBuf::from("/home/u/academic-planner-backup-20261029-1430.json");
    assert_eq!(
        backup_saved_message(&path),
        "Backup saved to /home/u/academic-planner-backup-20261029-1430.json"
    );
}

/// R23: the export success message names both files and the folder.
#[test]
fn csv_exported_message_names_folder() {
    let folder = PathBuf::from("/home/u/exports");
    assert_eq!(
        csv_exported_message(&folder),
        "Exported attendance.csv and assessments.csv to /home/u/exports"
    );
}

/// A failed file read or write names the path and the cause.
#[test]
fn file_error_message_names_path_and_cause() {
    let path = PathBuf::from("/home/u/backup.json");
    assert_eq!(
        file_error_message(&path, "permission denied"),
        "Could not access /home/u/backup.json: permission denied"
    );
}

/// R22: a failed pre-restore copy says the restore was not started.
#[test]
fn pre_restore_failed_message_says_restore_not_started() {
    assert_eq!(
        pre_restore_failed_message("disk full"),
        "Restore not started: could not copy the current data (disk full)"
    );
}

// ---- Mini calendar (§6 mini calendar change) ----

const TODAY_EMPTY: &str =
    "No classes today. Add courses in Courses and weekly slots in Timetable \u{2192} Rules.";

fn assessment_on(id: i64, due: NaiveDateTime, completed: bool) -> Assessment {
    Assessment {
        due_at: due,
        completed,
        ..assessment(id, "Quiz")
    }
}

fn grid_oct() -> Vec<CalendarDay> {
    month_grid(
        d(2026, 10, 1),
        d(2026, 10, 29),
        d(2026, 10, 29),
        &[],
        &[],
        &[],
    )
}

fn cell<'a>(grid: &'a [CalendarDay], key: &str) -> &'a CalendarDay {
    grid.iter().find(|c| c.key == key).unwrap()
}

/// §6 mini calendar change: month_start returns the 1st of the month.
#[test]
fn month_start_is_first_of_month() {
    assert_eq!(month_start(d(2026, 10, 29)), d(2026, 10, 1));
    assert_eq!(month_start(d(2026, 10, 1)), d(2026, 10, 1));
    assert_eq!(month_start(d(2026, 2, 28)), d(2026, 2, 1));
}

/// §6 mini calendar change: shift_month crosses year ends in both directions.
#[test]
fn shift_month_across_year_ends() {
    assert_eq!(shift_month(d(2026, 12, 1), 1), d(2027, 1, 1));
    assert_eq!(shift_month(d(2026, 1, 1), -1), d(2025, 12, 1));
    assert_eq!(shift_month(d(2026, 10, 1), 0), d(2026, 10, 1));
    assert_eq!(shift_month(d(2026, 10, 1), 3), d(2027, 1, 1));
    assert_eq!(shift_month(d(2026, 10, 1), -10), d(2025, 12, 1));
}

/// §6 mini calendar change: month title is the full month name and year.
#[test]
fn month_title_text() {
    assert_eq!(month_title(d(2026, 10, 1)), "October 2026");
    assert_eq!(month_title(d(2027, 1, 1)), "January 2027");
}

/// §6 mini calendar change, R3: class_days counts pending, attended, and absent sessions and
/// keeps duplicates.
#[test]
fn class_days_keeps_non_cancelled_with_duplicates() {
    let views = vec![
        session_view(1, d(2026, 10, 26), Status::Pending, "Pending", false),
        session_view(2, d(2026, 10, 26), Status::Attended, "Attended", true),
        session_view(3, d(2026, 10, 27), Status::Absent, "Absent", true),
    ];
    assert_eq!(
        class_days(&views),
        vec![d(2026, 10, 26), d(2026, 10, 26), d(2026, 10, 27)]
    );
    assert!(class_days(&[]).is_empty());
}

/// §6 mini calendar change, R12: cancelled sessions and rescheduled originals do not count.
#[test]
fn class_days_excludes_cancelled_and_rescheduled_original() {
    let views = vec![
        session_view(1, d(2026, 10, 26), Status::Cancelled, "Cancelled", true),
        session_view(
            2,
            d(2026, 10, 27),
            Status::Cancelled,
            "Rescheduled → 2026-10-29",
            true,
        ),
        pending_view(3, d(2026, 10, 29)),
    ];
    assert_eq!(class_days(&views), vec![d(2026, 10, 29)]);
}

/// §6 mini calendar change, R16: due_days lists incomplete assessments by due date, with
/// duplicates; completed ones are excluded.
#[test]
fn due_days_excludes_completed_keeps_duplicates() {
    let list = vec![
        assessment_on(1, dt(2026, 10, 30, 23, 59), false),
        assessment_on(2, dt(2026, 10, 30, 8, 0), false),
        assessment_on(3, dt(2026, 10, 31, 12, 0), true),
        assessment_on(4, dt(2026, 11, 2, 9, 0), false),
    ];
    assert_eq!(
        due_days(&list),
        vec![d(2026, 10, 30), d(2026, 10, 30), d(2026, 11, 2)]
    );
    assert!(due_days(&[]).is_empty());
}

/// §6 mini calendar change: the grid has 42 cells; October 2026 runs 2026-09-28 to 2026-11-08.
#[test]
fn month_grid_has_42_cells_from_monday() {
    let grid = grid_oct();
    assert_eq!(grid.len(), 42);
    assert_eq!(grid[0].key, "2026-09-28");
    assert_eq!(grid[41].key, "2026-11-08");
    // A month starting on a Monday begins its grid on the 1st.
    let jun = month_grid(
        d(2026, 6, 1),
        d(2026, 10, 29),
        d(2026, 10, 29),
        &[],
        &[],
        &[],
    );
    assert_eq!(jun.len(), 42);
    assert_eq!(jun[0].key, "2026-06-01");
}

/// §6 mini calendar change: in_month marks only days of the shown month; day text has no padding.
#[test]
fn month_grid_in_month_and_day_text() {
    let grid = grid_oct();
    assert!(!cell(&grid, "2026-09-30").in_month);
    assert!(cell(&grid, "2026-10-01").in_month);
    assert!(cell(&grid, "2026-10-31").in_month);
    assert!(!cell(&grid, "2026-11-01").in_month);
    assert_eq!(grid.iter().filter(|c| c.in_month).count(), 31);
    assert_eq!(cell(&grid, "2026-10-01").day, "1");
    assert_eq!(cell(&grid, "2026-10-09").day, "9");
    assert_eq!(cell(&grid, "2026-10-26").day, "26");
    assert_eq!(cell(&grid, "2026-09-28").day, "28");
}

/// §6 mini calendar change: exactly one cell is today and one is selected, and they can differ.
#[test]
fn month_grid_today_and_selected_flags() {
    let grid = month_grid(
        d(2026, 10, 1),
        d(2026, 10, 29),
        d(2026, 10, 26),
        &[],
        &[],
        &[],
    );
    assert_eq!(grid.iter().filter(|c| c.is_today).count(), 1);
    assert_eq!(grid.iter().filter(|c| c.is_selected).count(), 1);
    assert!(cell(&grid, "2026-10-29").is_today);
    assert!(!cell(&grid, "2026-10-29").is_selected);
    assert!(cell(&grid, "2026-10-26").is_selected);
    assert!(!cell(&grid, "2026-10-26").is_today);
}

/// §6 mini calendar change: today outside the shown grid marks no cell as today.
#[test]
fn month_grid_today_outside_grid_marks_nothing() {
    let grid = month_grid(d(2026, 3, 1), d(2026, 10, 29), d(2026, 3, 5), &[], &[], &[]);
    assert_eq!(grid.len(), 42);
    assert!(grid.iter().all(|c| !c.is_today));
    assert!(cell(&grid, "2026-03-05").is_selected);
}

/// §6 mini calendar change, R3, R16: has_class and has_due follow the given day lists, even for
/// days outside the month.
#[test]
fn month_grid_has_class_and_has_due_flags() {
    let classes = [d(2026, 10, 26), d(2026, 10, 26), d(2026, 9, 30)];
    let dues = [d(2026, 10, 30), d(2026, 11, 2)];
    let grid = month_grid(
        d(2026, 10, 1),
        d(2026, 10, 29),
        d(2026, 10, 29),
        &classes,
        &dues,
        &[],
    );
    assert!(cell(&grid, "2026-10-26").has_class);
    assert!(!cell(&grid, "2026-10-26").has_due);
    assert!(cell(&grid, "2026-09-30").has_class);
    assert!(cell(&grid, "2026-10-30").has_due);
    assert!(!cell(&grid, "2026-10-30").has_class);
    assert!(cell(&grid, "2026-11-02").has_due);
    assert!(!cell(&grid, "2026-10-27").has_class);
    assert!(!cell(&grid, "2026-10-27").has_due);
}

/// §6 mini calendar change: the accessible label is the long date, then ", today", then counts.
#[test]
fn month_grid_label_plain_day() {
    let grid = grid_oct();
    assert_eq!(cell(&grid, "2026-10-26").label, "Monday, October 26, 2026");
    assert_eq!(cell(&grid, "2026-10-01").label, "Thursday, October 1, 2026");
}

/// §6 mini calendar change: label uses singular and plural counts and the ", today" suffix.
#[test]
fn month_grid_label_counts_singular_plural_and_today() {
    let classes = [d(2026, 10, 26), d(2026, 10, 26), d(2026, 10, 29)];
    let dues = [
        d(2026, 10, 26),
        d(2026, 10, 27),
        d(2026, 10, 27),
        d(2026, 10, 29),
    ];
    let grid = month_grid(
        d(2026, 10, 1),
        d(2026, 10, 29),
        d(2026, 10, 29),
        &classes,
        &dues,
        &[],
    );
    assert_eq!(
        cell(&grid, "2026-10-26").label,
        "Monday, October 26, 2026, 2 classes, 1 assessment due"
    );
    assert_eq!(
        cell(&grid, "2026-10-27").label,
        "Tuesday, October 27, 2026, 2 assessments due"
    );
    assert_eq!(
        cell(&grid, "2026-10-29").label,
        "Thursday, October 29, 2026, today, 1 class, 1 assessment due"
    );
}

/// §6 mini calendar change: the key is the ISO date and round-trips through parse_date_key.
#[test]
fn month_grid_keys_are_iso_and_parse_back() {
    let grid = grid_oct();
    assert_eq!(cell(&grid, "2026-10-09").key, "2026-10-09");
    for c in &grid {
        assert!(parse_date_key(&c.key).is_some());
    }
    assert_eq!(parse_date_key(&grid[0].key), Some(d(2026, 9, 28)));
}

/// §6 mini calendar change: weekday initials start on Monday.
#[test]
fn weekday_initials_monday_first() {
    assert_eq!(weekday_initials(), ["M", "T", "W", "T", "F", "S", "S"]);
}

/// §6 mini calendar change, R16: the panel title is "Today's classes" for today and the long
/// weekday date for any other day.
#[test]
fn day_panel_title_today_vs_other() {
    let today = d(2026, 10, 29);
    assert_eq!(day_panel_title(today, today), "Today's classes");
    assert_eq!(
        day_panel_title(d(2026, 10, 26), today),
        "Monday, October 26, 2026"
    );
}

/// §6 mini calendar change, R16: today keeps the existing empty sentence; other days say
/// "Nothing on this day."
#[test]
fn day_panel_empty_text_today_vs_other() {
    let today = d(2026, 10, 29);
    assert_eq!(day_panel_empty_text(today, today), TODAY_EMPTY);
    assert_eq!(
        day_panel_empty_text(d(2026, 10, 26), today),
        "Nothing on this day."
    );
}

/// §6 mini calendar change: parse_date_key accepts only an ISO date key.
#[test]
fn parse_date_key_accepts_iso_only() {
    assert_eq!(parse_date_key("2026-10-26"), Some(d(2026, 10, 26)));
    assert_eq!(parse_date_key("26-10-2026"), None);
    assert_eq!(parse_date_key(""), None);
    assert_eq!(parse_date_key("2026-13-01"), None);
    assert_eq!(parse_date_key("nonsense"), None);
}

/// §6 mini calendar change: is_weekend is true for Saturday and Sunday, including days outside
/// the shown month.
#[test]
fn month_grid_is_weekend_flags() {
    let grid = grid_oct();
    assert!(cell(&grid, "2026-10-03").is_weekend);
    assert!(cell(&grid, "2026-10-04").is_weekend);
    assert!(!cell(&grid, "2026-10-05").is_weekend);
    assert!(!cell(&grid, "2026-10-02").is_weekend);
    assert!(cell(&grid, "2026-11-07").is_weekend);
    assert_eq!(grid.iter().filter(|c| c.is_weekend).count(), 12);
}

/// §6 mini calendar change, R14: a holiday cell shows its label; an unlabeled holiday shows
/// "Holiday"; other days show "".
#[test]
fn month_grid_holiday_text_from_label() {
    let holidays = [
        holiday(d(2026, 10, 27), Some("Midterm week")),
        holiday(d(2026, 10, 28), None),
    ];
    let grid = month_grid(
        d(2026, 10, 1),
        d(2026, 10, 29),
        d(2026, 10, 29),
        &[],
        &[],
        &holidays,
    );
    assert_eq!(cell(&grid, "2026-10-27").holiday, "Midterm week");
    assert_eq!(cell(&grid, "2026-10-28").holiday, "Holiday");
    assert_eq!(cell(&grid, "2026-10-26").holiday, "");
    assert_eq!(cell(&grid, "2026-10-29").holiday, "");
}

/// §6 mini calendar change, R14: a holiday on a weekend keeps both flags.
#[test]
fn month_grid_weekend_holiday_has_both() {
    let holidays = [holiday(d(2026, 10, 3), Some("Founders Day"))];
    let grid = month_grid(
        d(2026, 10, 1),
        d(2026, 10, 29),
        d(2026, 10, 29),
        &[],
        &[],
        &holidays,
    );
    let c = cell(&grid, "2026-10-03");
    assert!(c.is_weekend);
    assert_eq!(c.holiday, "Founders Day");
}

/// §6 mini calendar change, R14: the label adds ", <holiday>" after ", today" and before the
/// class and due counts.
#[test]
fn month_grid_label_includes_holiday_between_today_and_counts() {
    let holidays = [
        holiday(d(2026, 11, 2), Some("Midterm week")),
        holiday(d(2026, 10, 29), None),
    ];
    let dues = [d(2026, 11, 2), d(2026, 10, 29)];
    let classes = [d(2026, 10, 29)];
    let grid = month_grid(
        d(2026, 10, 1),
        d(2026, 10, 29),
        d(2026, 10, 29),
        &classes,
        &dues,
        &holidays,
    );
    assert_eq!(
        cell(&grid, "2026-11-02").label,
        "Monday, November 2, 2026, Midterm week, 1 assessment due"
    );
    assert_eq!(
        cell(&grid, "2026-10-29").label,
        "Thursday, October 29, 2026, today, Holiday, 1 class, 1 assessment due"
    );
}

/// §6 mini calendar change: the day panel holiday line is empty without a holiday, else
/// "Holiday: <text>".
#[test]
fn day_panel_holiday_text_empty_and_labeled() {
    assert_eq!(day_panel_holiday_text(""), "");
    assert_eq!(
        day_panel_holiday_text("Midterm week"),
        "Holiday: Midterm week"
    );
}
