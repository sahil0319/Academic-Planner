//! Unit tests for the view model (contracts/viewmodel.md). No database: core types are built
//! directly; each test checks one formatting or mapping function against its contract.

use std::path::PathBuf;

use super::*;
use planner_core::{Error, GpaStatus, NeededRow};

const DATE_ERR: &str = "Use DD-MM-YYYY, e.g. 29-10-2026";
const TIME_ERR: &str = "Enter the time as digits, e.g. 930 (hour 1–12, minutes 00–59)";
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
        biweekly: false,
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
        credits: None,
        rule_set_id: None,
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
        component_id: None,
        score: None,
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
        week_parity: None,
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

/// R31: times print in 12-hour form (`9:05 AM`), seconds dropped.
#[test]
fn format_time_drops_seconds() {
    assert_eq!(format_time(t(9, 5)), "9:05 AM");
    let with_seconds = NaiveTime::from_hms_opt(9, 5, 42).unwrap();
    assert_eq!(format_time(with_seconds), "9:05 AM");
}

/// T35, R31: midnight is 12:00 AM, noon is 12:00 PM, afternoon hours wrap to 1-11 PM.
#[test]
fn t35_times_display_12_hour() {
    assert_eq!(format_time(t(0, 0)), "12:00 AM");
    assert_eq!(format_time(t(9, 5)), "9:05 AM");
    assert_eq!(format_time(t(12, 0)), "12:00 PM");
    assert_eq!(format_time(t(13, 30)), "1:30 PM");
}

/// R31: time ranges use an en dash and no spaces around it.
#[test]
fn format_time_range_uses_en_dash() {
    assert_eq!(format_time_range(t(9, 0), t(10, 30)), "9:00 AM–10:30 AM");
}

/// R31: date and time print as `Month D, YYYY, H:MM AM/PM`.
#[test]
fn format_datetime_date_and_time() {
    assert_eq!(
        format_datetime(dt(2026, 10, 30, 23, 59)),
        "October 30, 2026, 11:59 PM"
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

/// T34, R32: AM inputs parse; 12 AM is hour 0; a colon is optional.
#[test]
fn t34_am_inputs_parse() {
    assert_eq!(parse_time("9", false), Ok(t(9, 0)));
    assert_eq!(parse_time("930", false), Ok(t(9, 30)));
    assert_eq!(parse_time("0930", false), Ok(t(9, 30)));
    assert_eq!(parse_time("1215", false), Ok(t(0, 15)));
    assert_eq!(parse_time("12:15", false), Ok(t(0, 15)));
}

/// T34, R32: PM inputs parse; 12 PM stays hour 12, other hours add 12.
#[test]
fn t34_pm_inputs_parse() {
    assert_eq!(parse_time("1215", true), Ok(t(12, 15)));
    assert_eq!(parse_time("130", true), Ok(t(13, 30)));
}

/// T34, R32, FR-007: bad inputs give the one format-hint error, with AM and with PM.
#[test]
fn t34_bad_inputs_are_errors() {
    for pm in [false, true] {
        for bad in ["1260", "1300", "0", ""] {
            assert_eq!(
                parse_time(bad, pm),
                Err(TIME_ERR.to_string()),
                "input {bad:?}, pm {pm}"
            );
        }
    }
}

/// R32: surrounding spaces are trimmed and a colon is allowed.
#[test]
fn parse_time_trims_and_accepts_colon() {
    assert_eq!(parse_time(" 930 ", false), Ok(t(9, 30)));
    assert_eq!(parse_time("9:30", false), Ok(t(9, 30)));
    assert_eq!(parse_time("9:30", true), Ok(t(21, 30)));
}

/// R32: hour 12 alone is midnight with AM and noon with PM.
#[test]
fn parse_time_twelve_alone_follows_toggle() {
    assert_eq!(parse_time("12", false), Ok(t(0, 0)));
    assert_eq!(parse_time("12", true), Ok(t(12, 0)));
}

/// R32, FR-007: out-of-range hours, a leading-zero 3-digit hour, 5+ digits, letters,
/// and a dot all give the format-hint error.
#[test]
fn parse_time_rejects_edge_inputs() {
    for pm in [false, true] {
        for bad in ["13", "030", "12345", "9a", "9.30"] {
            assert_eq!(
                parse_time(bad, pm),
                Err(TIME_ERR.to_string()),
                "input {bad:?}, pm {pm}"
            );
        }
    }
}

/// R32: pre-fill gives `hmm` digits and the PM flag (no leading zero on the hour).
#[test]
fn format_time_input_examples() {
    assert_eq!(format_time_input(t(9, 30)), ("930".to_string(), false));
    assert_eq!(format_time_input(t(13, 30)), ("130".to_string(), true));
    assert_eq!(format_time_input(t(0, 15)), ("1215".to_string(), false));
    assert_eq!(format_time_input(t(12, 5)), ("1205".to_string(), true));
    assert_eq!(format_time_input(t(0, 0)), ("1200".to_string(), false));
    assert_eq!(format_time_input(t(12, 0)), ("1200".to_string(), true));
}

/// R32, SC-005: pre-fill then parse returns the same time for every minute of the day.
#[test]
fn format_time_input_round_trips_all_minutes() {
    for minute_of_day in 0..1440_u32 {
        let time = t(minute_of_day / 60, minute_of_day % 60);
        let (digits, pm) = format_time_input(time);
        assert_eq!(parse_time(&digits, pm), Ok(time), "minute {minute_of_day}");
    }
}

/// R31: core's slot message shows its times in 12-hour form.
#[test]
fn times_to_12h_rewrites_slot_message() {
    assert_eq!(
        times_to_12h("slot end time 09:00 is not after its start time 10:30"),
        "slot end time 9:00 AM is not after its start time 10:30 AM"
    );
}

/// R31: a time inside a date-and-time text is rewritten, the date stays.
#[test]
fn times_to_12h_rewrites_time_after_date() {
    assert_eq!(
        times_to_12h("due October 30, 2026, 23:59"),
        "due October 30, 2026, 11:59 PM"
    );
}

/// R31: text with no time, and an ISO date, stay unchanged.
#[test]
fn times_to_12h_leaves_text_without_time_unchanged() {
    assert_eq!(
        times_to_12h("course code must not be empty"),
        "course code must not be empty"
    );
    assert_eq!(times_to_12h("starts on 2026-10-30"), "starts on 2026-10-30");
}

/// R31: tokens that are not a valid standalone `HH:MM` stay unchanged.
#[test]
fn times_to_12h_leaves_non_time_tokens_unchanged() {
    assert_eq!(times_to_12h("at 24:00"), "at 24:00");
    assert_eq!(times_to_12h("ratio 123:45"), "ratio 123:45");
}

/// R31: `error_text` shows core validation times in 12-hour form after capitalising.
#[test]
fn error_text_validation_shows_12_hour_times() {
    let e = Error::Validation("slot end time 09:00 is not after its start time 10:30".to_string());
    assert_eq!(
        error_text(&e),
        "Slot end time 9:00 AM is not after its start time 10:30 AM"
    );
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
    let input = parse_course_form(
        " CSE101 ",
        " Intro ",
        "Dr. Rahman",
        "  ",
        "",
        "75",
        "",
        None,
    )
    .unwrap();
    assert_eq!(
        input,
        CourseInput {
            code: "CSE101".to_string(),
            name: "Intro".to_string(),
            instructor: Some("Dr. Rahman".to_string()),
            room: None,
            notes: None,
            min_attendance_pct: Some(75),
            credits: None,
            rule_set_id: None,
        }
    );
}

/// Course form: an empty target means no target.
#[test]
fn parse_course_form_empty_target_is_none() {
    let input = parse_course_form("CSE101", "Intro", "", "", "", "", "", None).unwrap();
    assert_eq!(input.min_attendance_pct, None);
}

/// Course form: a non-integer target reports a field error.
#[test]
fn parse_course_form_bad_target_reports_field_error() {
    let err = parse_course_form("CSE101", "Intro", "", "", "", "7.5", "", None).unwrap_err();
    assert_eq!(err.get("target"), "Enter a whole number");
    assert_eq!(err.get("code"), "");
}

/// R2: empty code and out-of-range target are core's checks; the form passes them through.
#[test]
fn parse_course_form_passes_r2_checks_to_core() {
    let input = parse_course_form("", "", "", "", "", "150", "", None).unwrap();
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
        TimeText {
            digits: "900",
            pm: false,
        },
        TimeText {
            digits: "1030",
            pm: false,
        },
        " B12 ",
        "01-09-2026",
        "20-12-2026",
        None,
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
            week_parity: None,
        }
    );
}

/// Slot form: empty valid-to means until term end.
#[test]
fn parse_slot_form_empty_valid_to_is_none() {
    let input = parse_slot_form(
        Some(1),
        0,
        TimeText {
            digits: "900",
            pm: false,
        },
        TimeText {
            digits: "1030",
            pm: false,
        },
        "",
        "01-09-2026",
        "",
        None,
    )
    .unwrap();
    assert_eq!(input.valid_to, None);
    assert_eq!(input.room, None);
}

/// Slot form: every bad field is reported at once (FR-007).
#[test]
fn parse_slot_form_reports_all_errors_at_once() {
    let err = parse_slot_form(
        None,
        9,
        TimeText {
            digits: "9a",
            pm: false,
        },
        TimeText {
            digits: "1300",
            pm: false,
        },
        "",
        "x",
        "y",
        None,
    )
    .unwrap_err();
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
    let err = parse_slot_form(
        Some(1),
        0,
        TimeText {
            digits: "900",
            pm: false,
        },
        TimeText {
            digits: "1030",
            pm: false,
        },
        "",
        "",
        "",
        None,
    )
    .unwrap_err();
    assert_eq!(err.get("valid_from"), DATE_ERR);
    assert_eq!(err.get("valid_to"), "");
}

/// R2: slot end not after start is core's check; the form passes it through.
#[test]
fn parse_slot_form_passes_end_before_start_to_core() {
    let input = parse_slot_form(
        Some(1),
        0,
        TimeText {
            digits: "1030",
            pm: false,
        },
        TimeText {
            digits: "900",
            pm: false,
        },
        "",
        "01-09-2026",
        "",
        None,
    )
    .unwrap();
    assert_eq!(input.start_time, t(10, 30));
    assert_eq!(input.end_time, t(9, 0));
}

/// R9, R10: makeup form builds a `MakeupInput`; the note is optional.
#[test]
fn parse_makeup_form_valid() {
    let input = parse_makeup_form(
        4,
        "15-01-2026",
        TimeText {
            digits: "200",
            pm: true,
        },
        TimeText {
            digits: "330",
            pm: true,
        },
        "C3",
        " moved ",
    )
    .unwrap();
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
    let input = parse_makeup_form(
        4,
        "15-01-2026",
        TimeText {
            digits: "200",
            pm: true,
        },
        TimeText {
            digits: "330",
            pm: true,
        },
        "",
        "",
    )
    .unwrap();
    assert_eq!(input.room, None);
    assert_eq!(input.note, None);
}

/// R9, R10: all three required fields report errors at once.
#[test]
fn parse_makeup_form_reports_all_errors_at_once() {
    let err = parse_makeup_form(
        4,
        "",
        TimeText {
            digits: "2pm",
            pm: false,
        },
        TimeText {
            digits: "",
            pm: false,
        },
        "",
        "",
    )
    .unwrap_err();
    assert_eq!(err.get("date"), DATE_ERR);
    assert_eq!(err.get("start"), TIME_ERR);
    assert_eq!(err.get("end"), TIME_ERR);
}

/// Assessment form: `due_at` is date plus time; remind minutes parse.
#[test]
fn parse_assessment_form_valid() {
    let input = parse_assessment_form(
        Some(1),
        1,
        " Midterm ",
        "30-10-2026",
        TimeText {
            digits: "1159",
            pm: true,
        },
        "",
        "60",
        None,
        "",
    )
    .unwrap();
    assert_eq!(
        input,
        AssessmentInput {
            course_id: 1,
            kind: AssessmentKind::Exam,
            title: "Midterm".to_string(),
            due_at: dt(2026, 10, 30, 23, 59),
            notes: None,
            remind_minutes_before: Some(60),
            component_id: None,
            score: None,
        }
    );
}

/// Assessment form: empty remind means no reminder.
#[test]
fn parse_assessment_form_empty_remind_is_none() {
    let input = parse_assessment_form(
        Some(1),
        0,
        "Quiz 1",
        "30-10-2026",
        TimeText {
            digits: "900",
            pm: false,
        },
        "",
        "",
        None,
        "",
    )
    .unwrap();
    assert_eq!(input.remind_minutes_before, None);
    assert_eq!(input.kind, AssessmentKind::Quiz);
}

/// Assessment form: every bad field is reported at once (FR-007).
#[test]
fn parse_assessment_form_reports_all_errors_at_once() {
    let err = parse_assessment_form(
        None,
        9,
        "",
        "30/10",
        TimeText {
            digits: "noon",
            pm: false,
        },
        "",
        "-5",
        None,
        "",
    )
    .unwrap_err();
    assert_eq!(err.get("course"), "Choose a course");
    assert_eq!(err.get("kind"), "Choose a kind");
    assert_eq!(err.get("due_date"), DATE_ERR);
    assert_eq!(err.get("due_time"), TIME_ERR);
    assert_eq!(err.get("remind"), "Enter a whole number of minutes");
}

/// R2: an empty title is core's check; the form passes it through.
#[test]
fn parse_assessment_form_passes_empty_title_to_core() {
    let input = parse_assessment_form(
        Some(1),
        3,
        "",
        "30-10-2026",
        TimeText {
            digits: "900",
            pm: false,
        },
        "",
        "",
        None,
        "",
    )
    .unwrap();
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
            theme: ThemeMode::System,
            check_updates_on_start: true,
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
            time: "9:00 AM–10:30 AM".to_string(),
            course: "CSE101 · Intro to Programming".to_string(),
            room: "B12".to_string(),
            status_label: "Pending".to_string(),
            chip: ChipKind::Pending,
            can_mark: true,
            note: "bring laptop".to_string(),
            biweekly: false,
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
            selected: false,
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

/// R52: the delete button counts the selection; the confirm text names the codes (T54).
#[test]
fn t54_delete_label_and_confirm_text() {
    assert_eq!(delete_selected_label(1), "Delete selected (1)");
    assert_eq!(delete_selected_label(3), "Delete selected (3)");
    let text = delete_confirm_text(&["CSE101", "MAT102"]);
    assert!(
        text.starts_with("Delete courses CSE101, MAT102? "),
        "{text}"
    );
    assert!(text.contains("can't be undone"), "{text}");
    assert!(delete_confirm_text(&["CSE101"]).starts_with("Delete course CSE101? "));
}

/// R52: the selection drops courses no longer listed; "Select all" needs every listed course
/// selected and at least one course (T54).
#[test]
fn t54_selection_follows_the_list() {
    let mut selected: BTreeSet<i64> = [1, 2, 3].into_iter().collect();
    retain_listed(&mut selected, &[1, 3, 4]);
    assert_eq!(selected.into_iter().collect::<Vec<_>>(), vec![1, 3]);

    let none = BTreeSet::new();
    assert!(!all_selected(&none, &[]));
    assert!(!all_selected(&none, &[1]));
    let some: BTreeSet<i64> = [1, 2].into_iter().collect();
    assert!(all_selected(&some, &[1, 2]));
    assert!(!all_selected(&some, &[1, 2, 5]));
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
            time: "9:00 AM–10:30 AM".to_string(),
            course: "CSE101".to_string(),
            room: "C3".to_string(),
            valid: "From September 1, 2026 to December 20, 2026".to_string(),
            biweekly: false,
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
            due: "October 30, 2026, 11:59 PM".to_string(),
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

/// Dark mode (human decision 2026-10-29): theme mode maps to switch segments 0, 1, 2.
#[test]
fn theme_index_maps_modes() {
    assert_eq!(theme_index(ThemeMode::System), 0);
    assert_eq!(theme_index(ThemeMode::Light), 1);
    assert_eq!(theme_index(ThemeMode::Dark), 2);
}

/// Dark mode (human decision 2026-10-29): segments 0, 1, 2 map back; anything else is `None`.
#[test]
fn theme_from_index_maps_segments() {
    assert_eq!(theme_from_index(0), Some(ThemeMode::System));
    assert_eq!(theme_from_index(1), Some(ThemeMode::Light));
    assert_eq!(theme_from_index(2), Some(ThemeMode::Dark));
    assert_eq!(theme_from_index(-1), None);
    assert_eq!(theme_from_index(3), None);
}

// ---- M7: updater view model (contracts/ui-about.md; R27, R28, R29) ----

fn view(phase: UpdatePhase) -> UpdateView {
    update_view(&phase)
}

/// R25, R26: the startup error screen names the pre-migration copy through `error_text`.
#[test]
fn error_text_names_pre_migrate_backup_path() {
    let path = "/x/backups/pre-migrate-v1-to-v2.sqlite";
    let e = Error::MigrationFailed {
        version: 2,
        at: 1,
        backup: Some(PathBuf::from(path)),
        source: rusqlite::Error::QueryReturnedNoRows,
    };
    let text = error_text(&e);
    assert!(text.contains(path), "text does not name the copy: {text}");
}

/// R27: Idle shows no status and offers "Check for updates".
#[test]
fn update_view_idle() {
    let u = view(UpdatePhase::Idle);
    assert_eq!(u.status, "");
    assert_eq!(u.action, "Check for updates");
    assert!(u.action_enabled);
    assert!(!u.is_error);
}

/// R27: Checking shows its status and disables the button.
#[test]
fn update_view_checking() {
    let u = view(UpdatePhase::Checking);
    assert_eq!(u.status, "Checking for updates…");
    assert_eq!(u.action, "Check for updates");
    assert!(!u.action_enabled);
    assert!(!u.is_error);
}

/// R27: UpToDate says so and offers another check.
#[test]
fn update_view_up_to_date() {
    let u = view(UpdatePhase::UpToDate);
    assert_eq!(u.status, "You're up to date.");
    assert_eq!(u.action, "Check for updates");
    assert!(u.action_enabled);
    assert!(!u.is_error);
}

/// R27, R28: Available names the version and offers "Install".
#[test]
fn update_view_available() {
    let u = view(UpdatePhase::Available {
        version: "0.2.0".into(),
    });
    assert_eq!(u.status, "Version 0.2.0 is available.");
    assert_eq!(u.action, "Install");
    assert!(u.action_enabled);
    assert!(!u.is_error);
}

/// R28: Downloading starts its status with "Downloading…", keeps "Install" disabled, and
/// carries the progress fraction and text.
#[test]
fn update_view_downloading() {
    let u = view(UpdatePhase::Downloading {
        received: 42,
        total: Some(100),
    });
    assert!(
        u.status.starts_with("Downloading…"),
        "status was {:?}",
        u.status
    );
    assert_eq!(u.action, "Install");
    assert!(!u.action_enabled);
    assert!(!u.is_error);
    assert!((u.progress - 0.42).abs() < 1e-6, "progress {}", u.progress);
    assert_eq!(u.progress_text, "Downloading… 42%");
}

/// R28: with an unknown total the progress is -1.0 and the text has no percentage.
#[test]
fn update_view_downloading_unknown_total() {
    let u = view(UpdatePhase::Downloading {
        received: 42,
        total: None,
    });
    assert_eq!(u.progress, -1.0);
    assert_eq!(u.progress_text, "Downloading…");
    assert!(!u.action_enabled);
}

/// R28: Installing shows its status and disables "Install".
#[test]
fn update_view_installing() {
    let u = view(UpdatePhase::Installing);
    assert_eq!(u.status, "Installing…");
    assert_eq!(u.action, "Install");
    assert!(!u.action_enabled);
    assert!(!u.is_error);
}

/// R27: a failed check says "Couldn't check for updates", offers "Retry", and is an error.
#[test]
fn update_view_check_failed() {
    let u = view(UpdatePhase::CheckFailed);
    assert_eq!(u.status, "Couldn't check for updates");
    assert_eq!(u.action, "Retry");
    assert!(u.action_enabled);
    assert!(u.is_error);
}

/// R28: a failed install says nothing was changed, offers "Retry", and is an error.
#[test]
fn update_view_install_failed() {
    let u = view(UpdatePhase::InstallFailed);
    assert_eq!(
        u.status,
        "The update couldn't be installed. Nothing was changed."
    );
    assert_eq!(u.action, "Retry");
    assert!(u.action_enabled);
    assert!(u.is_error);
}

/// R28: `is_error` is true only for CheckFailed and InstallFailed.
#[test]
fn update_view_is_error_only_for_failures() {
    let phases = [
        (UpdatePhase::Idle, false),
        (UpdatePhase::Checking, false),
        (UpdatePhase::UpToDate, false),
        (
            UpdatePhase::Available {
                version: "0.2.0".into(),
            },
            false,
        ),
        (
            UpdatePhase::Downloading {
                received: 1,
                total: Some(2),
            },
            false,
        ),
        (UpdatePhase::Installing, false),
        (UpdatePhase::CheckFailed, true),
        (UpdatePhase::InstallFailed, true),
    ];
    for (phase, expected) in phases {
        assert_eq!(update_view(&phase).is_error, expected, "{phase:?}");
    }
}

/// R28: the action is disabled while Checking, Downloading, and Installing, and only then.
#[test]
fn update_view_action_disabled_only_while_busy() {
    let phases = [
        (UpdatePhase::Idle, true),
        (UpdatePhase::Checking, false),
        (UpdatePhase::UpToDate, true),
        (
            UpdatePhase::Available {
                version: "0.2.0".into(),
            },
            true,
        ),
        (
            UpdatePhase::Downloading {
                received: 1,
                total: None,
            },
            false,
        ),
        (UpdatePhase::Installing, false),
        (UpdatePhase::CheckFailed, true),
        (UpdatePhase::InstallFailed, true),
    ];
    for (phase, expected) in phases {
        assert_eq!(update_view(&phase).action_enabled, expected, "{phase:?}");
    }
}

/// R28: progress is the received share of the total, clamped to 0.0..=1.0.
#[test]
fn progress_fraction_known_total() {
    assert_eq!(progress_fraction(0, Some(100)), 0.0);
    assert!((progress_fraction(42, Some(100)) - 0.42).abs() < 1e-6);
    assert_eq!(progress_fraction(150, Some(100)), 1.0);
}

/// R28: progress is -1.0 when the total size is unknown.
#[test]
fn progress_fraction_unknown_total() {
    assert_eq!(progress_fraction(42, None), -1.0);
}

/// §13.1 UI: the version line reads "Version 0.1.1".
#[test]
fn version_text_prefixes_version() {
    assert_eq!(version_text("0.1.1"), "Version 0.1.1");
}

/// §13.1 UI: the license line.
/// R53: the GitHub button shows the owner/repo path of the repo URL.
#[test]
fn repo_label_is_owner_and_repo() {
    assert_eq!(
        repo_label("https://github.com/sahil0319/Academic-Planner"),
        "sahil0319/Academic-Planner"
    );
    assert_eq!(repo_label("https://github.com/a/b/"), "a/b");
}

#[test]
fn license_text_states_gpl() {
    assert_eq!(
        license_text(),
        "Free software under the GNU GPL v3 or later."
    );
}

/// R29: the update banner text.
#[test]
fn banner_text_names_version() {
    assert_eq!(banner_text("0.2.0"), "Version 0.2.0 is available.");
}

/// R28: the confirm panel title.
#[test]
fn confirm_title_names_version() {
    assert_eq!(confirm_title("0.2.0"), "Install version 0.2.0?");
}

/// R28: the confirm button says what happens for each kind of install.
#[test]
fn confirm_label_by_install_kind() {
    assert_eq!(
        confirm_label(InstallKind::SelfInstall),
        "Install and restart"
    );
    assert_eq!(
        confirm_label(InstallKind::OpenReleasePage),
        "Open release page"
    );
}

// Router decisions on M7 tester ambiguities (R28): the percentage rounds down, a zero total
// counts as unknown, and phases other than Downloading carry no progress.

/// R28: the download percentage rounds down, so 100% shows only once everything arrived.
#[test]
fn update_view_download_percent_rounds_down() {
    let u = view(UpdatePhase::Downloading {
        received: 999,
        total: Some(1000),
    });
    assert_eq!(u.progress_text, "Downloading… 99%");
    let done = view(UpdatePhase::Downloading {
        received: 1000,
        total: Some(1000),
    });
    assert_eq!(done.progress_text, "Downloading… 100%");
}

/// R28: a total size of zero is treated as unknown (indeterminate progress).
#[test]
fn update_view_zero_total_is_unknown() {
    assert_eq!(progress_fraction(5, Some(0)), -1.0);
    let u = view(UpdatePhase::Downloading {
        received: 5,
        total: Some(0),
    });
    assert_eq!(u.progress, -1.0);
    assert_eq!(u.progress_text, "Downloading…");
}

/// R28: outside Downloading the progress is 0 and its text is empty.
#[test]
fn update_view_no_progress_outside_downloading() {
    for phase in [
        UpdatePhase::Idle,
        UpdatePhase::Checking,
        UpdatePhase::UpToDate,
        UpdatePhase::Available {
            version: "0.2.0".into(),
        },
        UpdatePhase::Installing,
        UpdatePhase::CheckFailed,
        UpdatePhase::InstallFailed,
    ] {
        let u = update_view(&phase);
        assert_eq!(u.progress, 0.0, "{phase:?}");
        assert_eq!(u.progress_text, "", "{phase:?}");
    }
}

// ---- M9: grading formatting (contracts/ui-grading.md) ----

fn rule_component(
    name: &str,
    kind: ComponentKind,
    weight_pct: f64,
    counts: Option<(u32, u32)>,
    item_max: Option<f64>,
    is_final: bool,
) -> RuleComponent {
    RuleComponent {
        id: 1,
        rule_set_id: 1,
        position: 0,
        name: name.to_string(),
        kind,
        weight_pct,
        item_count: counts.map(|(count, _)| count),
        best_count: counts.map(|(_, best)| best),
        item_max,
        is_final,
        bands: Vec::new(),
    }
}

/// R37: a mark drops a trailing `.0` and keeps one real decimal.
#[test]
fn t37_format_mark_drops_trailing_zero() {
    assert_eq!(format_mark(45.0), "45");
    assert_eq!(format_mark(37.5), "37.5");
    assert_eq!(format_mark(1.5), "1.5");
}

/// R37, T37: Quizzes of the seeded Theory rule set show `39.0 of 45`.
#[test]
fn t37_format_marks_quizzes() {
    assert_eq!(format_marks(39.0, 45.0), "39.0 of 45");
}

/// R35, T38: Attendance of the seeded Theory rule set shows `27.0 of 30`.
#[test]
fn t38_format_marks_attendance() {
    assert_eq!(format_marks(27.0, 30.0), "27.0 of 30");
}

/// R37, T40: projected percentage and letter in one string; no data shows an em dash.
#[test]
fn t37_format_projected_with_and_without_data() {
    assert_eq!(format_projected(Some(80.0), Some("A+")), "80.0% · A+");
    assert_eq!(format_projected(None, None), "—");
}

/// R38, T39: a needed score shows with one decimal.
#[test]
fn t39_needed_text_score() {
    assert_eq!(needed_text(&Needed::Score(111.0)), "111.0");
}

/// R38, T39: secured and not reachable have their own wording.
#[test]
fn t39_needed_text_secured_and_not_reachable() {
    assert_eq!(needed_text(&Needed::Secured), "Secured");
    assert_eq!(needed_text(&Needed::NotReachable), "Not reachable");
}

/// R38, T39: one view row per grade letter, in scale order.
#[test]
fn t39_needed_rows_follow_scale_order() {
    let outlook = FinalOutlook::Needed {
        rows: vec![
            NeededRow {
                letter: "A+".to_string(),
                min_pct: 80.0,
                needed: Needed::Score(111.0),
            },
            NeededRow {
                letter: "A".to_string(),
                min_pct: 75.0,
                needed: Needed::Secured,
            },
            NeededRow {
                letter: "B".to_string(),
                min_pct: 70.0,
                needed: Needed::NotReachable,
            },
        ],
        final_item_max: 150.0,
        assumptions: Vec::new(),
    };
    let rows = needed_rows(&outlook);
    let pairs: Vec<(&str, &str)> = rows
        .iter()
        .map(|r| (r.letter.as_str(), r.text.as_str()))
        .collect();
    assert_eq!(
        pairs,
        vec![("A+", "111.0"), ("A", "Secured"), ("B", "Not reachable")]
    );
}

/// R38: once the final has a score there is no needed table.
#[test]
fn t39_needed_rows_empty_for_actual() {
    let outlook = FinalOutlook::Actual {
        letter: Some("A".to_string()),
    };
    assert!(needed_rows(&outlook).is_empty());
}

/// R38, T39: a missing quiz is listed as assumed 0; one item and several items.
#[test]
fn t39_assumption_text_one_and_two_items() {
    let one = Assumption {
        component_name: "Quizzes".to_string(),
        missing_items: 1,
    };
    let two = Assumption {
        component_name: "Quizzes".to_string(),
        missing_items: 2,
    };
    assert_eq!(
        assumption_text(&one),
        "Quizzes: assumes 0 for 1 item not entered"
    );
    assert_eq!(
        assumption_text(&two),
        "Quizzes: assumes 0 for 2 items not entered"
    );
}

/// R38: the actual grade line, with and without a letter.
#[test]
fn t39_actual_grade_text() {
    assert_eq!(actual_grade_text(Some("A")), "Final grade: A");
    assert_eq!(actual_grade_text(None), "Final grade: —");
}

/// R40, T41: header for a projected GPA, a final GPA, and no GPA.
#[test]
fn t41_gpa_header_projected_final_and_none() {
    let summary = |gpa, status| GpaSummary {
        gpa,
        status,
        included: 2,
        left_out: 0,
    };
    assert_eq!(
        gpa_header_text(&summary(Some(3.67), GpaStatus::Projected)),
        "Term GPA 3.67 (projected)"
    );
    assert_eq!(
        gpa_header_text(&summary(Some(3.67), GpaStatus::Final)),
        "Term GPA 3.67 (final)"
    );
    assert_eq!(
        gpa_header_text(&summary(None, GpaStatus::Projected)),
        "Term GPA —"
    );
}

/// R40: a graded course left out for lack of marks is counted in the header.
#[test]
fn t41_gpa_header_mentions_left_out_course() {
    let summary = GpaSummary {
        gpa: Some(3.67),
        status: GpaStatus::Projected,
        included: 2,
        left_out: 1,
    };
    assert_eq!(
        gpa_header_text(&summary),
        "Term GPA 3.67 (projected) · 1 graded course without marks"
    );
}

/// R34: component summaries for the seeded Attendance, Quizzes, and Final components.
#[test]
fn t37_component_summary_seeded_components() {
    let attendance = rule_component(
        "Attendance",
        ComponentKind::Attendance,
        10.0,
        None,
        None,
        false,
    );
    let quizzes = rule_component(
        "Quizzes",
        ComponentKind::Items,
        15.0,
        Some((4, 3)),
        Some(15.0),
        false,
    );
    let final_exam = rule_component(
        "Final",
        ComponentKind::Items,
        50.0,
        Some((1, 1)),
        None,
        true,
    );
    assert_eq!(component_summary(&attendance), "Attendance · 10%");
    assert_eq!(
        component_summary(&quizzes),
        "Items · 15% · best 3 of 4, out of 15"
    );
    assert_eq!(
        component_summary(&final_exam),
        "Items · 50% · 1 of 1 · final"
    );
}

/// R35: an attendance band reads `min → credit`.
#[test]
fn t38_band_text() {
    let band = AttendanceBand {
        min_pct: 90.0,
        credit_pct: 100.0,
    };
    assert_eq!(band_text(&band), "≥ 90% → 100%");
}

/// R36: the score field shows the item max after a slash; no max shows nothing.
#[test]
fn t37_score_max_label() {
    assert_eq!(score_max_label(Some(15.0)), "/ 15");
    assert_eq!(score_max_label(None), "");
}

// ---- M9: grading forms ----

fn course_form_with(credits: &str, rule_set_id: Option<i64>) -> Result<CourseInput, FormErrors> {
    parse_course_form("CSE101", "Intro", "", "", "", "", credits, rule_set_id)
}

fn assessment_form_with(
    component_id: Option<i64>,
    score: &str,
) -> Result<AssessmentInput, FormErrors> {
    parse_assessment_form(
        Some(1),
        0,
        "Quiz 1",
        "30-10-2026",
        TimeText {
            digits: "900",
            pm: false,
        },
        "",
        "",
        component_id,
        score,
    )
}

/// R33: empty credits mean not graded; numbers (whole or fractional) parse.
#[test]
fn t33_course_form_credits_parse() {
    assert_eq!(course_form_with("", None).unwrap().credits, None);
    assert_eq!(course_form_with("3", None).unwrap().credits, Some(3.0));
    assert_eq!(course_form_with("1.5", None).unwrap().credits, Some(1.5));
}

/// R33: credits that are not a number or not above 0 give an inline error on `credits`.
#[test]
fn t33_course_form_bad_credits_inline_error() {
    for bad in ["abc", "0", "-1"] {
        let err = course_form_with(bad, None).unwrap_err();
        assert!(!err.get("credits").is_empty(), "credits {bad:?}");
    }
}

/// R33: the rule set choice passes through unchanged, `None` included.
#[test]
fn t33_course_form_rule_set_passes_through() {
    assert_eq!(course_form_with("3", None).unwrap().rule_set_id, None);
    assert_eq!(course_form_with("3", Some(5)).unwrap().rule_set_id, Some(5));
}

/// R36: an empty score means no score; whole and fractional scores parse.
#[test]
fn t36_assessment_form_score_parse() {
    assert_eq!(assessment_form_with(None, "").unwrap().score, None);
    assert_eq!(assessment_form_with(None, "12").unwrap().score, Some(12.0));
    assert_eq!(
        assessment_form_with(None, "12.5").unwrap().score,
        Some(12.5)
    );
}

/// R36: a score that is not a number or is below 0 gives an inline error on `score`.
#[test]
fn t36_assessment_form_bad_score_inline_error() {
    for bad in ["x", "-1"] {
        let err = assessment_form_with(None, bad).unwrap_err();
        assert!(!err.get("score").is_empty(), "score {bad:?}");
    }
}

/// R36: the component choice passes through unchanged, `None` included.
#[test]
fn t36_assessment_form_component_passes_through() {
    assert_eq!(assessment_form_with(None, "").unwrap().component_id, None);
    assert_eq!(
        assessment_form_with(Some(7), "").unwrap().component_id,
        Some(7)
    );
}

/// R34: number fields parse; an empty item max is `None` (the component's marks).
#[test]
fn t34_component_row_parses_numbers() {
    let quizzes = parse_component_row(
        Some(3),
        "Quizzes",
        ComponentKind::Items,
        "15",
        "4",
        "3",
        "15",
        false,
        Vec::new(),
    )
    .unwrap();
    assert_eq!(
        quizzes,
        RuleComponentInput {
            id: Some(3),
            name: "Quizzes".to_string(),
            kind: ComponentKind::Items,
            weight_pct: 15.0,
            item_count: Some(4),
            best_count: Some(3),
            item_max: Some(15.0),
            is_final: false,
            bands: Vec::new(),
        }
    );
    let final_exam = parse_component_row(
        None,
        "Final",
        ComponentKind::Items,
        "50",
        "1",
        "1",
        "",
        true,
        Vec::new(),
    )
    .unwrap();
    assert_eq!(final_exam.item_max, None);
    assert!(final_exam.is_final);
    assert_eq!(final_exam.id, None);
}

/// R34: an attendance row keeps its bands and has no item counts.
#[test]
fn t34_component_row_attendance_keeps_bands() {
    let bands = vec![
        AttendanceBand {
            min_pct: 90.0,
            credit_pct: 100.0,
        },
        AttendanceBand {
            min_pct: 80.0,
            credit_pct: 90.0,
        },
    ];
    let row = parse_component_row(
        None,
        "Attendance",
        ComponentKind::Attendance,
        "10",
        "",
        "",
        "",
        false,
        bands.clone(),
    )
    .unwrap();
    assert_eq!(row.kind, ComponentKind::Attendance);
    assert_eq!(row.weight_pct, 10.0);
    assert_eq!(row.item_count, None);
    assert_eq!(row.best_count, None);
    assert_eq!(row.item_max, None);
    assert_eq!(row.bands, bands);
}

/// R34: every field that is not a number gets its own inline error, all reported at once.
#[test]
fn t34_component_row_bad_numbers_inline_errors() {
    let err = parse_component_row(
        None,
        "Quizzes",
        ComponentKind::Items,
        "abc",
        "x",
        "y",
        "z",
        false,
        Vec::new(),
    )
    .unwrap_err();
    for field in ["weight", "item_count", "best_count", "item_max"] {
        assert!(!err.get(field).is_empty(), "{field}");
    }
}

/// R35: a band row parses both numbers.
#[test]
fn t35_band_row_parses() {
    assert_eq!(
        parse_band_row("90", "100").unwrap(),
        AttendanceBand {
            min_pct: 90.0,
            credit_pct: 100.0,
        }
    );
}

/// R35: a band number that is not a number gives an inline error on its field.
#[test]
fn t35_band_row_bad_numbers_inline_errors() {
    let err = parse_band_row("abc", "100").unwrap_err();
    assert!(!err.get("min").is_empty());
    assert!(err.get("credit").is_empty());
    let err = parse_band_row("90", "").unwrap_err();
    assert!(!err.get("credit").is_empty());
    assert!(err.get("min").is_empty());
}

/// R39: a grade row parses its numbers.
#[test]
fn t39_grade_row_parses() {
    assert_eq!(
        parse_grade_row("A+", "80", "4").unwrap(),
        GradeRow {
            letter: "A+".to_string(),
            min_pct: 80.0,
            points: 4.0,
        }
    );
}

/// R39: a grade number that is not a number gives an inline error on its field.
#[test]
fn t39_grade_row_bad_numbers_inline_errors() {
    let err = parse_grade_row("A+", "x", "y").unwrap_err();
    assert!(!err.get("min").is_empty());
    assert!(!err.get("points").is_empty());
}

// ---- M9: rules page rows and choice lists ----

/// R34: a seeded-style component becomes editable text; ids and flags carry over.
#[test]
fn r34_component_form_row_texts() {
    let quizzes = RuleComponent {
        id: 2,
        rule_set_id: 1,
        position: 1,
        name: "Quizzes".to_string(),
        kind: ComponentKind::Items,
        weight_pct: 15.0,
        item_count: Some(4),
        best_count: Some(3),
        item_max: Some(15.0),
        is_final: false,
        bands: Vec::new(),
    };
    assert_eq!(
        component_form_row(&quizzes),
        ComponentFormRow {
            id: Some(2),
            name: "Quizzes".to_string(),
            is_items: true,
            weight: "15".to_string(),
            item_count: "4".to_string(),
            best_count: "3".to_string(),
            item_max: "15".to_string(),
            is_final: false,
        }
    );
    let attendance = RuleComponent {
        kind: ComponentKind::Attendance,
        item_count: None,
        best_count: None,
        item_max: None,
        ..quizzes
    };
    let row = component_form_row(&attendance);
    assert!(!row.is_items);
    assert_eq!(row.item_count, "");
    assert_eq!(row.item_max, "");
}

/// R34, R35, R39: band and grade rows as input text.
#[test]
fn r39_band_and_grade_form_rows() {
    let band = AttendanceBand {
        min_pct: 85.0,
        credit_pct: 90.0,
    };
    assert_eq!(band_form_row(&band), ("85".to_string(), "90".to_string()));
    let grade = GradeRow {
        letter: "B+".to_string(),
        min_pct: 65.0,
        points: 3.25,
    };
    assert_eq!(
        grade_form_row(&grade),
        ("B+".to_string(), "65".to_string(), "3.25".to_string())
    );
}

/// R34: the rule set list shows each component with its weight.
#[test]
fn r34_rule_set_row_summary() {
    let component = |id: i64, name: &str, weight: f64| RuleComponent {
        id,
        rule_set_id: 7,
        position: id,
        name: name.to_string(),
        kind: ComponentKind::Items,
        weight_pct: weight,
        item_count: Some(1),
        best_count: Some(1),
        item_max: None,
        is_final: false,
        bands: Vec::new(),
    };
    let set = RuleSet {
        id: 7,
        name: "Seminar".to_string(),
        components: vec![component(1, "Paper", 37.5), component(2, "Talk", 62.5)],
    };
    assert_eq!(
        rule_set_row(&set),
        RuleSetRow {
            id: 7,
            name: "Seminar".to_string(),
            summary: "Paper 37.5%, Talk 62.5%".to_string(),
        }
    );
}

/// R33, R36: choice lists put the "none" label first and map indexes to ids.
#[test]
fn r33_choice_labels_and_ids() {
    let names = vec!["Theory".to_string(), "Lab".to_string()];
    assert_eq!(
        choice_labels(NOT_GRADED_LABEL, &names),
        vec!["Not graded", "Theory", "Lab"]
    );
    let ids = [10, 20];
    assert_eq!(choice_id(0, &ids), None);
    assert_eq!(choice_id(1, &ids), Some(10));
    assert_eq!(choice_id(2, &ids), Some(20));
    assert_eq!(choice_id(3, &ids), None);
    assert_eq!(choice_id(-1, &ids), None);
    assert_eq!(choice_index(None, &ids), 0);
    assert_eq!(choice_index(Some(20), &ids), 2);
    assert_eq!(choice_index(Some(99), &ids), 0);
}

/// R33, R36: stored credits and scores come back as input text.
#[test]
fn r33_number_input_text() {
    assert_eq!(number_input(None), "");
    assert_eq!(number_input(Some(3.0)), "3");
    assert_eq!(number_input(Some(1.5)), "1.5");
    assert_eq!(number_input(Some(12.0)), "12");
}

/// R34: row errors read as one line with field labels.
#[test]
fn r34_field_errors_text_one_line() {
    let err = parse_component_row(
        None,
        "Quizzes",
        ComponentKind::Items,
        "x",
        "4",
        "3",
        "y",
        false,
        Vec::new(),
    )
    .unwrap_err();
    assert_eq!(
        field_errors_text(&err),
        "Weight: Enter a number; Out of: Enter a number"
    );
    assert_eq!(field_errors_text(&FormErrors::default()), "");
}

fn component_marks(name: &str, marks: f64, earned: f64) -> planner_core::ComponentMarks {
    planner_core::ComponentMarks {
        component_id: 1,
        name: name.to_string(),
        kind: ComponentKind::Items,
        is_final: false,
        marks,
        earned,
        assessed_fraction: 1.0,
        item_max: None,
        scores_entered: 0,
        missing_items: 0,
    }
}

fn course_marks(outlook: Option<FinalOutlook>) -> CourseMarks {
    CourseMarks {
        course_id: 1,
        graded: true,
        total: 300.0,
        components: vec![
            component_marks("Quizzes", 45.0, 39.0),
            component_marks("Attendance", 30.0, 27.0),
        ],
        secured: 129.0,
        assessed: 150.0,
        projected_pct: Some(86.0),
        letter: Some("A+".to_string()),
        fully_assessed: false,
        outlook,
    }
}

/// T37, T38, T39: the marks card shows each component, the secured total, the projection, and
/// the needed-in-final table with its assumption lines.
#[test]
fn t39_marks_view_needed_table_with_assumptions() {
    let outlook = FinalOutlook::Needed {
        rows: vec![
            NeededRow {
                letter: "A+".to_string(),
                min_pct: 80.0,
                needed: Needed::Score(111.0),
            },
            NeededRow {
                letter: "F".to_string(),
                min_pct: 0.0,
                needed: Needed::Secured,
            },
        ],
        final_item_max: 150.0,
        assumptions: vec![Assumption {
            component_name: "Quizzes".to_string(),
            missing_items: 1,
        }],
    };
    let view = marks_view(&course_marks(Some(outlook)));
    assert!(view.graded);
    assert_eq!(
        view.rows,
        vec![
            ("Quizzes".to_string(), "39.0 of 45".to_string()),
            ("Attendance".to_string(), "27.0 of 30".to_string()),
        ]
    );
    assert_eq!(view.secured, "129.0 of 300");
    assert_eq!(view.projected, "86.0% \u{b7} A+");
    assert_eq!(view.needed_caption, "Score needed in the final, out of 150");
    assert_eq!(view.needed.len(), 2);
    assert_eq!(view.needed[0].letter, "A+");
    assert_eq!(view.needed[0].text, "111.0");
    assert_eq!(view.needed[1].text, "Secured");
    assert_eq!(
        view.assumptions,
        vec!["Quizzes: assumes 0 for 1 item not entered".to_string()]
    );
    assert_eq!(view.actual, "");
}

/// R38: a scored final replaces the table with the actual grade.
#[test]
fn r38_marks_view_actual_grade_replaces_table() {
    let view = marks_view(&course_marks(Some(FinalOutlook::Actual {
        letter: Some("A".to_string()),
    })));
    assert!(view.needed.is_empty());
    assert_eq!(view.needed_caption, "");
    assert_eq!(view.actual, "Final grade: A");
}

/// R33: an ungraded course has no rows and the card shows only the ungraded line.
#[test]
fn r33_marks_view_ungraded_is_empty() {
    let mut marks = course_marks(None);
    marks.graded = false;
    let view = marks_view(&marks);
    assert!(!view.graded);
    assert!(view.rows.is_empty());
    assert!(view.needed.is_empty());
}

/// T37, T39 end to end (92% attendance is in the 100% band): the worked example of the spec (3-credit Theory, attendance 92%,
/// quizzes 12/11/13/14, mid 60 of 75) reads through core into the card and table text.
#[test]
fn t39_worked_example_end_to_end() {
    use planner_core::Planner;

    let at = |h: u32, m: u32| NaiveTime::from_hms_opt(h, m, 0).unwrap();
    let mut p = Planner::open_in_memory().unwrap();
    p.save_term(TermInput {
        id: None,
        name: "Winter 2026".to_string(),
        start_date: d(2026, 1, 5),
        end_date: d(2026, 1, 25),
    })
    .unwrap();
    let theory = p.list_rule_sets().unwrap().remove(0);
    assert_eq!(theory.name, "Theory");
    let course = p
        .create_course(CourseInput {
            code: "CSE101".to_string(),
            name: "Intro".to_string(),
            instructor: None,
            room: None,
            notes: None,
            min_attendance_pct: None,
            credits: Some(3.0),
            rule_set_id: Some(theory.id),
        })
        .unwrap();
    // 23 attended and 2 absent makeups give 92.0%.
    for i in 0..25i64 {
        let start = at(6, 0) + Duration::hours(i / 21);
        let makeup = p
            .add_makeup(planner_core::MakeupInput {
                course_id: course.id,
                date: d(2026, 1, 5) + Days::new((i % 21) as u64),
                start_time: start,
                end_time: start + Duration::minutes(30),
                room: None,
                note: None,
            })
            .unwrap();
        let status = if i < 2 {
            Status::Absent
        } else {
            Status::Attended
        };
        p.set_status(makeup.id, status, dt(2026, 2, 1, 0, 0))
            .unwrap();
    }
    let component = |name: &str| {
        theory
            .components
            .iter()
            .find(|c| c.name == name)
            .unwrap()
            .id
    };
    let mut score = |name: &str, title: &str, score: f64| {
        p.create_assessment(AssessmentInput {
            course_id: course.id,
            kind: AssessmentKind::Other,
            title: title.to_string(),
            due_at: dt(2026, 1, 12, 10, 0),
            notes: None,
            remind_minutes_before: None,
            component_id: Some(component(name)),
            score: Some(score),
        })
        .unwrap();
    };
    for (i, s) in [12.0, 11.0, 13.0, 14.0].into_iter().enumerate() {
        score("Quizzes", &format!("Quiz {}", i + 1), s);
    }
    score("Mid", "Mid", 60.0);

    let view = marks_view(&p.course_marks(course.id).unwrap());
    let row = |name: &str| view.rows.iter().find(|r| r.0 == name).unwrap().1.clone();
    assert_eq!(row("Quizzes"), "39.0 of 45");
    assert_eq!(row("Attendance"), "30.0 of 30");
    assert_eq!(view.secured, "129.0 of 300");
    assert_eq!(view.projected, "86.0% \u{b7} A+");
    assert_eq!(view.needed_caption, "Score needed in the final, out of 150");
    let needed = |letter: &str| {
        view.needed
            .iter()
            .find(|r| r.letter == letter)
            .unwrap()
            .text
            .clone()
    };
    assert_eq!(needed("A+"), "111.0");
    assert_eq!(needed("A"), "96.0");
    assert_eq!(needed("D"), "Secured");
    assert!(view.assumptions.is_empty());
    assert_eq!(
        gpa_header_text(&p.term_gpa().unwrap()),
        "Term GPA 4.00 (projected)"
    );
}

/// R35/R36: zero earned marks print "0.0 of 45", never "-0.0 of 45" (negative zero).
#[test]
fn t_negative_zero_prints_without_minus() {
    assert_eq!(format_marks(-0.0, 45.0), "0.0 of 45");
    assert_eq!(format_marks(-0.0, 75.0), "0.0 of 75");
    assert_eq!(format_marks(-0.0, 150.0), "0.0 of 150");
    assert_eq!(format_mark(-0.0), "0");
}

/// R35/R36: a value that rounds to zero at the printed precision has no minus sign.
#[test]
fn t_tiny_negative_rounds_to_unsigned_zero() {
    assert_eq!(format_marks(-0.04, 45.0), "0.0 of 45");
    assert_eq!(format_mark(-0.001), "0");
    assert_eq!(format_pct(Some(-0.04)), "0.0%");
    assert_eq!(format_pct(Some(-0.0)), "0.0%");
    assert_eq!(format_projected(Some(-0.0), Some("F")), "0.0% \u{b7} F");
    assert_eq!(format_projected(Some(-0.04), None), "0.0%");
    assert_eq!(needed_text(&Needed::Score(-0.0)), "0.0");
    assert_eq!(needed_text(&Needed::Score(-0.04)), "0.0");
    let zero_gpa = GpaSummary {
        gpa: Some(-0.0),
        status: GpaStatus::Projected,
        included: 1,
        left_out: 0,
    };
    assert_eq!(gpa_header_text(&zero_gpa), "Term GPA 0.00 (projected)");
}

/// Real negatives keep their minus sign.
#[test]
fn t_real_negatives_keep_minus() {
    assert_eq!(format_marks(-1.5, 45.0), "-1.5 of 45");
    assert_eq!(format_mark(-2.5), "-2.5");
    assert_eq!(format_pct(Some(-12.34)), "-12.3%");
    assert_eq!(needed_text(&Needed::Score(-3.0)), "-3.0");
}

// ---- M18: saved rule lines (R87) ----

fn saved_lines_set(band_90_credit: f64, attendance_weight: f64) -> RuleSet {
    RuleSet {
        id: 3,
        name: "Theory".to_string(),
        components: vec![
            RuleComponent {
                id: 1,
                rule_set_id: 3,
                position: 0,
                name: "Attendance".to_string(),
                kind: ComponentKind::Attendance,
                weight_pct: attendance_weight,
                item_count: None,
                best_count: None,
                item_max: None,
                is_final: false,
                bands: vec![
                    AttendanceBand {
                        min_pct: 95.0,
                        credit_pct: 100.0,
                    },
                    AttendanceBand {
                        min_pct: 90.0,
                        credit_pct: band_90_credit,
                    },
                ],
            },
            RuleComponent {
                id: 2,
                rule_set_id: 3,
                position: 1,
                name: "Final".to_string(),
                kind: ComponentKind::Items,
                weight_pct: 90.0,
                item_count: Some(1),
                best_count: Some(1),
                item_max: None,
                is_final: true,
                bands: Vec::new(),
            },
        ],
    }
}

/// R87: the saved lines list each component once, each band indented under its attendance
/// component; a changed band and a changed weight change exactly those two lines.
#[test]
fn t80_rule_set_lines_follow_the_stored_set() {
    let before = rule_set_lines(&saved_lines_set(80.0, 10.0));
    assert_eq!(
        before,
        [
            "Attendance: Attendance \u{b7} 10%",
            "  \u{2265} 95% \u{2192} 100%",
            "  \u{2265} 90% \u{2192} 80%",
            "Final: Items \u{b7} 90% \u{b7} 1 of 1 \u{b7} final",
        ]
    );

    let after = rule_set_lines(&saved_lines_set(70.0, 15.0));
    assert_eq!(after.len(), before.len());
    let changed: Vec<usize> = (0..before.len())
        .filter(|&i| before[i] != after[i])
        .collect();
    assert_eq!(
        changed,
        [0, 2],
        "only the weight line and the 90 band line differ"
    );
    assert_eq!(after[0], "Attendance: Attendance \u{b7} 15%");
    assert_eq!(after[2], "  \u{2265} 90% \u{2192} 70%");
}

// ---- M18 toast texts (R88-R91) ----
//
// Page wiring (toast shown, page message left empty, hint above the Save button) has no headless
// test: there is no headless Slint backend. It is covered by the manual smoke in the final report.

/// R89: each confirmation text, exact; the three older messages keep their wording.
#[test]
fn t85_confirmation_texts_are_exact() {
    assert_eq!(term_saved_text(), "Term saved.");
    assert_eq!(holidays_saved_text(), "Holidays saved.");
    assert_eq!(holiday_removed_text(), "Holiday removed.");
    assert_eq!(settings_saved_text(), "Settings saved.");
    assert_eq!(template_copied_text(), "Template copied");
    assert_eq!(prompt_copied_text(), "AI prompt copied");
    assert_eq!(rule_set_saved_text(), "Rule set saved.");
    assert_eq!(grade_scale_saved_text(), "Grade scale saved.");
    // The existing message functions are unchanged (their own tests cover the paths).
    assert_eq!(
        backup_saved_message(&PathBuf::from("/b/x.json")),
        "Backup saved to /b/x.json"
    );
    assert_eq!(
        restored_message(&PathBuf::from("/b/pre.sqlite")),
        "Backup restored. Previous data saved to /b/pre.sqlite"
    );
    assert_eq!(
        csv_exported_message(&PathBuf::from("/e")),
        "Exported attendance.csv and assessments.csv to /e"
    );
}

/// R88, R90: saving a rule set twice (2 s apart) gives one toast; the second save restarts the
/// 5 s timer, so it is still visible at 6 s and gone at 7 s.
#[test]
fn t81_repeat_rule_set_save_restarts_timer() {
    use crate::toast::ToastQueue;
    use std::time::Duration;

    let mut queue = ToastQueue::new();
    let first = queue.add(&rule_set_saved_text(), Duration::from_secs(0));
    let second = queue.add(&rule_set_saved_text(), Duration::from_secs(2));
    assert_eq!(first, second);

    let at_6 = queue.visible(Duration::from_secs(6));
    assert_eq!(at_6.len(), 1);
    assert_eq!(at_6[0].text, "Rule set saved.");
    assert!(queue.visible(Duration::from_secs(7)).is_empty());
}

/// R91: the hint above the Save button when a row fails to parse.
#[test]
fn t82_fix_rows_hint_text() {
    assert_eq!(
        fix_rows_hint_text(),
        "Fix the rows marked in red before saving."
    );
}

/// R90: the grade scale save text, and it shows as its own toast beside "Rule set saved.".
#[test]
fn t83_grade_scale_saved_is_its_own_toast() {
    use crate::toast::ToastQueue;
    use std::time::Duration;

    assert_eq!(grade_scale_saved_text(), "Grade scale saved.");

    let mut queue = ToastQueue::new();
    let now = Duration::from_secs(1);
    let rules = queue.add(&rule_set_saved_text(), now);
    let scale = queue.add(&grade_scale_saved_text(), now);
    assert_ne!(rules, scale);

    let texts: Vec<&str> = queue
        .visible(now)
        .iter()
        .map(|toast| toast.text.as_str())
        .collect();
    assert_eq!(texts, ["Rule set saved.", "Grade scale saved."]);
}

// ---- M13: Progress page ----

/// T62, R63: how many of the remaining classes can still be missed.
#[test]
fn t62_classes_you_can_still_miss() {
    assert_eq!(
        miss_line(10, 2, 8, Some(75)),
        "You can miss 3 of the 8 remaining classes and stay at or above 75%"
    );
    assert_eq!(
        miss_line(10, 4, 4, Some(75)),
        "Attend all 4 remaining classes to stay at or above 75%"
    );
    assert_eq!(
        miss_line(5, 5, 2, Some(75)),
        "75% is out of reach, even attending all 2 remaining classes"
    );
    assert_eq!(
        miss_line(20, 0, 2, Some(75)),
        "You can miss all 2 remaining classes and stay at or above 75%"
    );
    assert_eq!(miss_line(3, 1, 0, Some(75)), "No classes left to mark");
    assert_eq!(miss_line(10, 2, 8, None), "");
}

/// R63, D4: a negative numerator is floored (`div_euclid`), so a small negative gives k < 0.
#[test]
fn r63_miss_line_floors_negative_numerator() {
    // 100 * (6 + 2) - 75 * (6 + 3 + 2) = -25, k = -1.
    assert_eq!(
        miss_line(6, 3, 2, Some(75)),
        "75% is out of reach, even attending all 2 remaining classes"
    );
    // 100 * (7 + 2) - 75 * (7 + 3 + 2) = 0, k = 0.
    assert_eq!(
        miss_line(7, 3, 2, Some(75)),
        "Attend all 2 remaining classes to stay at or above 75%"
    );
}

/// R63, D6: the noun is singular when the remaining count is 1.
#[test]
fn r63_miss_line_singular_class() {
    assert_eq!(
        miss_line(10, 0, 1, Some(75)),
        "You can miss all 1 remaining class and stay at or above 75%"
    );
}

/// R62, D6: the attendance line next to the bar.
#[test]
fn r62_attendance_line() {
    assert_eq!(
        attendance_line(14, 2, Some(87.5)),
        "87.5% · 14 of 16 classes"
    );
    assert_eq!(attendance_line(1, 0, Some(100.0)), "100.0% · 1 of 1 class");
    assert_eq!(attendance_line(0, 0, None), "No classes marked yet");
}

/// R62, D7: target text, bar fill, and target marker position.
#[test]
fn r62_target_text_and_bar() {
    assert_eq!(progress_target_text(Some(75)), "Target 75%");
    assert_eq!(progress_target_text(None), "No target");
    assert_eq!(bar_fraction(Some(87.5)), 0.875);
    assert_eq!(bar_fraction(None), 0.0);
    assert_eq!(bar_fraction(Some(100.0)), 1.0);
    assert_eq!(target_fraction(Some(75)), Some(0.75));
    assert_eq!(target_fraction(None), None);
}

fn needed_row(letter: &str, needed: Needed) -> NeededRow {
    NeededRow {
        letter: letter.to_string(),
        min_pct: 0.0,
        needed,
    }
}

fn needed_outlook(rows: Vec<NeededRow>) -> FinalOutlook {
    FinalOutlook::Needed {
        rows,
        final_item_max: 150.0,
        assumptions: vec![],
    }
}

/// T64, R64: the best grade line names the highest letter that is still reachable.
#[test]
fn t64_best_reachable_grade() {
    let outlook = needed_outlook(vec![
        needed_row("A+", Needed::NotReachable),
        needed_row("A", Needed::Score(96.0)),
        needed_row("A-", Needed::Score(80.0)),
    ]);
    assert_eq!(
        best_grade_line(Some(&outlook)),
        "A needs 96.0 of 150 in the final"
    );
    let secured = needed_outlook(vec![
        needed_row("A+", Needed::Secured),
        needed_row("A", Needed::Secured),
    ]);
    assert_eq!(best_grade_line(Some(&secured)), "A+ is already secured");
    let actual = FinalOutlook::Actual {
        letter: Some("A".to_string()),
    };
    assert_eq!(best_grade_line(Some(&actual)), "Final grade: A");
}

/// R64, D10: the best grade line is empty when there is no outlook or nothing is reachable.
#[test]
fn r64_best_grade_line_hidden() {
    assert_eq!(best_grade_line(None), "");
    let none_reachable = needed_outlook(vec![
        needed_row("A+", Needed::NotReachable),
        needed_row("A", Needed::NotReachable),
    ]);
    assert_eq!(best_grade_line(Some(&none_reachable)), "");
}

/// R64, D9: the marks line for not graded, no marks yet, and secured of assessed.
#[test]
fn r64_marks_line() {
    let not_graded = CourseMarks {
        graded: false,
        ..course_marks(None)
    };
    assert_eq!(progress_marks_line(&not_graded), "Not graded");
    let no_marks = CourseMarks {
        projected_pct: None,
        ..course_marks(None)
    };
    assert_eq!(progress_marks_line(&no_marks), "No marks entered yet");
    assert_eq!(
        progress_marks_line(&course_marks(None)),
        "Secured 129.0 of 150 assessed marks"
    );
}

fn grade_row(letter: &str, min_pct: f64, points: f64) -> GradeRow {
    GradeRow {
        letter: letter.to_string(),
        min_pct,
        points,
    }
}

fn progress_scale() -> Vec<GradeRow> {
    vec![
        grade_row("A+", 80.0, 4.0),
        grade_row("B", 65.0, 3.0),
        grade_row("B-", 60.0, 2.75),
        grade_row("F", 0.0, 0.0),
    ]
}

/// T63, R65: status from attendance and projected grade points.
#[test]
fn t63_progress_status() {
    use ProgressStatus::*;
    assert_eq!(
        progress_status(true, Some(70.0), Some(75), None, None),
        AtRisk
    );
    assert_eq!(
        progress_status(false, Some(78.0), Some(75), None, None),
        Watch
    );
    assert_eq!(
        progress_status(false, Some(90.0), Some(75), Some(75.0), Some(3.0)),
        OnTrack
    );
    assert_eq!(
        progress_status(false, Some(90.0), Some(75), Some(72.0), Some(2.75)),
        Watch
    );
    assert_eq!(
        progress_status(false, Some(90.0), Some(75), Some(40.0), Some(0.0)),
        AtRisk
    );
    assert_eq!(
        progress_status(false, None, Some(75), None, None),
        NotEnoughData
    );
}

/// R65, D12: status boundaries (the Watch band is pct < target + 5; no target needs no band).
#[test]
fn r65_status_boundaries() {
    use ProgressStatus::*;
    assert_eq!(
        progress_status(false, Some(80.0), Some(75), None, None),
        OnTrack
    );
    assert_eq!(
        progress_status(false, Some(79.9), Some(75), None, None),
        Watch
    );
    assert_eq!(
        progress_status(false, Some(60.0), None, None, None),
        OnTrack
    );
    assert_eq!(
        progress_status(false, None, None, Some(86.0), Some(4.0)),
        OnTrack
    );
}

/// R65, D11: grade points come from the scale by letter.
#[test]
fn r65_letter_points() {
    let scale = progress_scale();
    assert_eq!(letter_points(Some("B"), &scale), Some(3.0));
    assert_eq!(letter_points(Some("Z"), &scale), None);
    assert_eq!(letter_points(None, &scale), None);
}

/// R65: status labels and chip kinds.
#[test]
fn r65_label_and_chip() {
    use ProgressStatus::*;
    assert_eq!(progress_status_label(AtRisk), "At risk");
    assert_eq!(progress_status_label(Watch), "Watch");
    assert_eq!(progress_status_label(OnTrack), "On track");
    assert_eq!(progress_status_label(NotEnoughData), "Not enough data");
    assert_eq!(progress_chip(AtRisk), ChipKind::Absent);
    assert_eq!(progress_chip(Watch), ChipKind::Pending);
    assert_eq!(progress_chip(OnTrack), ChipKind::Attended);
    assert_eq!(progress_chip(NotEnoughData), ChipKind::Cancelled);
}

fn progress_summary(
    code: &str,
    attended: u32,
    absent: u32,
    pending: u32,
    pct: Option<f64>,
    target: Option<i64>,
    below_target: bool,
) -> AttendanceSummary {
    AttendanceSummary {
        course_id: 1,
        course_code: code.to_string(),
        course_name: format!("{code} name"),
        attended,
        absent,
        cancelled: 0,
        pending,
        pct,
        target,
        below_target,
    }
}

fn ungraded_marks() -> CourseMarks {
    CourseMarks {
        graded: false,
        projected_pct: None,
        letter: None,
        outlook: None,
        ..course_marks(None)
    }
}

/// R61: every card field comes from its source function; target and grading variants.
#[test]
fn r61_progress_card_fields() {
    let scale = progress_scale();
    let card = progress_card(
        &progress_summary("CSE101", 14, 2, 8, Some(87.5), Some(75), false),
        &course_marks(None),
        &scale,
    );
    assert_eq!(card.code, "CSE101");
    assert_eq!(card.name, "CSE101 name");
    assert_eq!(card.status, ProgressStatus::OnTrack);
    assert_eq!(card.status_label, "On track");
    assert_eq!(card.chip, ChipKind::Attended);
    assert_eq!(card.attendance_figure, "87.5%");
    assert_eq!(card.attendance_line, "87.5% · 14 of 16 classes");
    assert_eq!(card.target_text, "Target 75%");
    assert_eq!(card.bar_fill, 0.875);
    assert!(card.has_marker);
    assert_eq!(card.marker_at, 0.75);
    assert_eq!(card.miss_line, miss_line(14, 2, 8, Some(75)));
    assert_eq!(card.marks_figure, "86.0% · A+");
    assert_eq!(card.marks_line, "Secured 129.0 of 150 assessed marks");
    assert_eq!(card.best_grade_line, "");

    let ungraded = progress_card(
        &progress_summary("CSE101", 14, 2, 8, Some(87.5), Some(75), false),
        &ungraded_marks(),
        &scale,
    );
    assert_eq!(ungraded.marks_figure, "—");
    assert_eq!(ungraded.marks_line, "Not graded");

    let no_target = progress_card(
        &progress_summary("CSE101", 14, 2, 8, Some(87.5), None, false),
        &course_marks(None),
        &scale,
    );
    assert!(!no_target.has_marker);
    assert_eq!(no_target.marker_at, 0.0);
    assert_eq!(no_target.miss_line, "");
}

/// T65, R61: cards sort by status then code; the header carries the GPA and the status counts.
#[test]
fn t65_progress_order_and_counts() {
    let scale = progress_scale();
    let ungraded = ungraded_marks();
    let cards = vec![
        progress_card(
            &progress_summary("AAA101", 18, 2, 5, Some(90.0), Some(75), false),
            &course_marks(None),
            &scale,
        ),
        progress_card(
            &progress_summary("BBB101", 0, 0, 10, None, Some(75), false),
            &ungraded,
            &scale,
        ),
        progress_card(
            &progress_summary("CCC101", 7, 3, 5, Some(70.0), Some(75), true),
            &ungraded,
            &scale,
        ),
        progress_card(
            &progress_summary("DDD101", 39, 11, 5, Some(78.0), Some(75), false),
            &ungraded,
            &scale,
        ),
    ];
    let statuses: Vec<_> = cards.iter().map(|card| card.status).collect();
    assert_eq!(
        statuses,
        vec![
            ProgressStatus::OnTrack,
            ProgressStatus::NotEnoughData,
            ProgressStatus::AtRisk,
            ProgressStatus::Watch,
        ]
    );
    let gpa = GpaSummary {
        gpa: Some(3.67),
        status: GpaStatus::Projected,
        included: 1,
        left_out: 0,
    };
    let view = progress_view(&gpa, cards);
    let codes: Vec<_> = view.cards.iter().map(|card| card.code.as_str()).collect();
    assert_eq!(codes, vec!["CCC101", "DDD101", "AAA101", "BBB101"]);
    assert_eq!(view.counts_text, "1 on track · 1 to watch · 1 at risk");
    assert_eq!(view.gpa_text, gpa_header_text(&gpa));
}

/// R61: cards with the same status are ordered by course code.
#[test]
fn r61_same_status_sorted_by_code() {
    let scale = progress_scale();
    let ungraded = ungraded_marks();
    let cards = vec![
        progress_card(
            &progress_summary("ZZZ101", 7, 3, 5, Some(70.0), Some(75), true),
            &ungraded,
            &scale,
        ),
        progress_card(
            &progress_summary("AAA101", 7, 3, 5, Some(70.0), Some(75), true),
            &ungraded,
            &scale,
        ),
    ];
    let gpa = GpaSummary {
        gpa: None,
        status: GpaStatus::Projected,
        included: 0,
        left_out: 0,
    };
    let view = progress_view(&gpa, cards);
    let codes: Vec<_> = view.cards.iter().map(|card| card.code.as_str()).collect();
    assert_eq!(codes, vec!["AAA101", "ZZZ101"]);
}

/// R61, D13: empty-state texts and the counts text with no cards.
#[test]
fn r61_empty_text() {
    assert_eq!(
        progress_empty_text(false),
        "No term yet. Create one in Settings → Term."
    );
    assert_eq!(
        progress_empty_text(true),
        "No courses yet. Add one on the Courses page."
    );
    assert_eq!(
        progress_counts_text(&[]),
        "0 on track · 0 to watch · 0 at risk"
    );
}

// ---- M12: bi-weekly slot form (R56, R58) ----

/// Term start used by the T61 tests: a Monday. 2026-01-12 is the Monday of week 2 (even).
fn t61_term_start() -> NaiveDate {
    d(2026, 1, 5)
}

fn t61_today() -> NaiveDate {
    d(2026, 1, 12)
}

/// R56, T61: bi-weekly and "Class this week" ticked on an even week stores Even.
#[test]
fn t61_class_this_week_ticked_stores_current_parity() {
    assert_eq!(
        slot_parity_from_form(t61_term_start(), t61_today(), true, true),
        Some(WeekParity::Even)
    );
}

/// R56, T61: bi-weekly with "Class this week" not ticked stores the other parity.
#[test]
fn t61_class_this_week_unticked_stores_other_parity() {
    assert_eq!(
        slot_parity_from_form(t61_term_start(), t61_today(), true, false),
        Some(WeekParity::Odd)
    );
}

/// R56, T61: an unticked bi-weekly box stores no parity, whatever "Class this week" says.
#[test]
fn t61_weekly_form_stores_no_parity() {
    for class_this_week in [true, false] {
        assert_eq!(
            slot_parity_from_form(t61_term_start(), t61_today(), false, class_this_week),
            None
        );
    }
}

/// R56, T61: editing an odd slot on 01-12 pre-fills bi-weekly ticked, "Class this week" not.
#[test]
fn t61_edit_odd_slot_prefills_not_this_week() {
    assert_eq!(
        slot_form_from_parity(t61_term_start(), t61_today(), Some(WeekParity::Odd)),
        (true, false)
    );
}

/// R56, T61: editing an even slot on 01-12 pre-fills both boxes ticked.
#[test]
fn t61_edit_even_slot_prefills_this_week() {
    assert_eq!(
        slot_form_from_parity(t61_term_start(), t61_today(), Some(WeekParity::Even)),
        (true, true)
    );
}

/// R56, T61: editing a weekly slot pre-fills bi-weekly unticked and "Class this week" ticked.
#[test]
fn t61_edit_weekly_slot_prefills_defaults() {
    assert_eq!(
        slot_form_from_parity(t61_term_start(), t61_today(), None),
        (false, true)
    );
}

/// R56, T61: form to parity to form returns the same boxes for all four combinations, except
/// that an unticked bi-weekly box always comes back with "Class this week" ticked.
#[test]
fn t61_form_round_trip() {
    for (biweekly, class_this_week) in [(true, true), (true, false), (false, true), (false, false)]
    {
        let parity =
            slot_parity_from_form(t61_term_start(), t61_today(), biweekly, class_this_week);
        let back = slot_form_from_parity(t61_term_start(), t61_today(), parity);
        let expected = if biweekly {
            (true, class_this_week)
        } else {
            (false, true)
        };
        assert_eq!(back, expected);
    }
}

/// R54, R56, T61: a term starting on a Wednesday counts its own week as week 1 (odd), so the
/// following Monday is week 2 (even).
#[test]
fn t61_wednesday_start_term_counts_own_week_as_week_one() {
    let start = d(2026, 1, 7);
    assert_eq!(
        slot_parity_from_form(start, d(2026, 1, 7), true, true),
        Some(WeekParity::Odd)
    );
    assert_eq!(
        slot_parity_from_form(start, d(2026, 1, 9), true, true),
        Some(WeekParity::Odd)
    );
    assert_eq!(
        slot_parity_from_form(start, d(2026, 1, 12), true, true),
        Some(WeekParity::Even)
    );
    assert_eq!(
        slot_form_from_parity(start, d(2026, 1, 12), Some(WeekParity::Odd)),
        (true, false)
    );
}

/// R56, T61: `parse_slot_form` copies the week parity into the input unchanged.
#[test]
fn t61_parse_slot_form_passes_week_parity_through() {
    for parity in [None, Some(WeekParity::Odd), Some(WeekParity::Even)] {
        let input = parse_slot_form(
            Some(1),
            1,
            TimeText {
                digits: "900",
                pm: false,
            },
            TimeText {
                digits: "1030",
                pm: false,
            },
            "",
            "01-09-2026",
            "",
            parity,
        )
        .unwrap();
        assert_eq!(input.week_parity, parity);
    }
}

/// R58, T61: a slot row is flagged bi-weekly exactly when the slot has a week parity.
#[test]
fn t61_slot_row_biweekly_flag() {
    let courses = [course(1, "CSE101")];
    let mut s = slot(None, None);
    assert!(!slot_row(&s, &courses).biweekly);
    s.week_parity = Some(WeekParity::Odd);
    assert!(slot_row(&s, &courses).biweekly);
    s.week_parity = Some(WeekParity::Even);
    assert!(slot_row(&s, &courses).biweekly);
}

/// R58, T61: a session row carries the session view's bi-weekly flag.
#[test]
fn t61_session_row_biweekly_flag() {
    let mut view = pending_view(1, d(2026, 1, 12));
    assert!(!session_row(&view).biweekly);
    view.biweekly = true;
    assert!(session_row(&view).biweekly);
}

/// R58, T61: the label next to the time of a bi-weekly session or slot.
#[test]
fn t61_every_other_week_label() {
    assert_eq!(EVERY_OTHER_WEEK, "Every other week");
}

/// R58: the label is empty for a weekly session or slot and "Every other week" for a bi-weekly one.
#[test]
fn biweekly_label_follows_the_flag() {
    assert_eq!(biweekly_label(true), EVERY_OTHER_WEEK);
    assert_eq!(biweekly_label(false), "");
}

/// R58: "Move to next week" shows for a pending bi-weekly session dated today or later.
#[test]
fn can_shift_biweekly_needs_biweekly_pending_and_not_past_date() {
    let today = d(2026, 1, 12);
    let mut view = pending_view(1, today);
    view.biweekly = true;
    assert!(can_shift_biweekly(&view, today));
    view.session.date = d(2026, 1, 20);
    assert!(can_shift_biweekly(&view, today));
    view.session.date = d(2026, 1, 11);
    assert!(!can_shift_biweekly(&view, today));
}

/// R58: a weekly session, or one that is not pending, never offers the button.
#[test]
fn can_shift_biweekly_false_for_weekly_and_marked_sessions() {
    let today = d(2026, 1, 12);
    let weekly = pending_view(1, today);
    assert!(!can_shift_biweekly(&weekly, today));
    for status in [Status::Attended, Status::Absent, Status::Cancelled] {
        let mut view = session_view(2, today, status, "Marked", true);
        view.biweekly = true;
        assert!(!can_shift_biweekly(&view, today));
    }
}

/// R58, R89: the toast text after a move.
#[test]
fn moved_next_week_text_is_the_toast() {
    assert_eq!(moved_next_week_text(), "Moved to next week.");
}
