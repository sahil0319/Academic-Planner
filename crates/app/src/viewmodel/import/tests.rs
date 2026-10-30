//! Unit tests for the import-panel view model (contracts/ui-import.md, M10 T023). No database:
//! core types are built directly; each test checks one formatting or gate function against
//! its contract row. Times are 12-hour (R31, via `format_time_range`); dates use the long
//! `Month D, YYYY` form (`format_date`, M8).

use super::*;
use chrono::{NaiveTime, Weekday};
use planner_core::{CourseUpdate, FieldValue, ImportField, RuleSetRef, Slot, SyncReport};

fn d(y: i32, m: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, day).unwrap()
}

fn t(h: u32, m: u32) -> NaiveTime {
    NaiveTime::from_hms_opt(h, m, 0).unwrap()
}

fn issue(path: &str, message: &str) -> ImportIssue {
    ImportIssue {
        path: path.to_string(),
        message: message.to_string(),
    }
}

fn import_slot(
    weekday: Weekday,
    start: NaiveTime,
    end: NaiveTime,
    room: Option<&str>,
) -> ImportSlot {
    ImportSlot {
        weekday,
        start,
        end,
        room: room.map(str::to_string),
        week_parity: None,
    }
}

fn stored_slot(weekday: Weekday, start: NaiveTime, end: NaiveTime, room: Option<&str>) -> Slot {
    Slot {
        id: 1,
        course_id: 1,
        weekday,
        start_time: start,
        end_time: end,
        room: room.map(str::to_string),
        valid_from: d(2026, 1, 5),
        valid_to: None,
        week_parity: None,
    }
}

fn course_add(code: &str, name: &str, credits: Option<f64>, rule_set: Option<&str>) -> CourseAdd {
    CourseAdd {
        code: code.to_string(),
        name: name.to_string(),
        instructor: None,
        room: None,
        min_attendance_pct: None,
        credits,
        rule_set: rule_set.map(|n| RuleSetRef {
            id: 1,
            name: n.to_string(),
        }),
        slots: Vec::new(),
    }
}

fn course_update(code: &str) -> CourseUpdate {
    CourseUpdate {
        course_id: 1,
        code: code.to_string(),
        archived: false,
        changes: Vec::new(),
        slots: None,
    }
}

fn term(name: &str) -> ImportTerm {
    ImportTerm {
        name: name.to_string(),
        start_date: d(2026, 1, 5),
        end_date: d(2026, 5, 1),
    }
}

fn plan(term: Option<ImportTerm>, add: usize, update: usize) -> ImportPlan {
    ImportPlan {
        term,
        add: (0..add)
            .map(|i| course_add(&format!("CSE {i}"), "Course", None, None))
            .collect(),
        update: (0..update)
            .map(|i| course_update(&format!("EEE {i}")))
            .collect(),
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn change(field: ImportField, old: FieldValue, new: FieldValue) -> FieldChange {
    FieldChange { field, old, new }
}

fn text(s: &str) -> FieldValue {
    FieldValue::Text(Some(s.to_string()))
}

fn preview(text: &str, date_text: &str, has_errors: bool, is_empty: bool) -> PreviewState {
    PreviewState {
        text: text.to_string(),
        date_text: date_text.to_string(),
        has_errors,
        is_empty,
    }
}

// ---- issue_line, errors_heading ----

/// R42: an issue prints as `path: message`, with the JSON path first.
#[test]
fn issue_line_is_path_colon_message() {
    assert_eq!(
        issue_line(&issue("courses[2].slots[0].end", "must be after start")),
        "courses[2].slots[0].end: must be after start"
    );
}

/// R42: a document-level error uses the `document` path the same way.
#[test]
fn issue_line_document_path() {
    assert_eq!(
        issue_line(&issue("document", "not valid JSON (line 1, column 5)")),
        "document: not valid JSON (line 1, column 5)"
    );
}

/// R42: the heading is singular for one error and plural otherwise, with an em dash.
#[test]
fn errors_heading_singular_and_plural() {
    assert_eq!(errors_heading(1), "1 error — nothing will be imported");
    assert_eq!(errors_heading(3), "3 errors — nothing will be imported");
}

// ---- plan_summary ----

/// R47, R48: term, adds, and updates all appear, joined by ` · `.
#[test]
fn plan_summary_term_adds_and_updates() {
    assert_eq!(
        plan_summary(&plan(Some(term("Semester 7")), 5, 1)),
        "Creates term \"Semester 7\" · adds 5 courses · updates 1 course"
    );
}

/// R48: counts of one say `1 course`, counts of two or more say `N courses`.
#[test]
fn plan_summary_singular_and_plural_courses() {
    assert_eq!(
        plan_summary(&plan(Some(term("Semester 7")), 1, 2)),
        "Creates term \"Semester 7\" · adds 1 course · updates 2 courses"
    );
}

/// R43, R48: parts that are absent from the plan are left out of the summary.
#[test]
fn plan_summary_omits_absent_parts() {
    assert_eq!(plan_summary(&plan(None, 2, 0)), "adds 2 courses");
    assert_eq!(plan_summary(&plan(None, 0, 1)), "updates 1 course");
    assert_eq!(
        plan_summary(&plan(None, 3, 2)),
        "adds 3 courses · updates 2 courses"
    );
}

/// R48: an empty plan reads `No changes. Everything already matches.`
#[test]
fn plan_summary_empty_plan() {
    assert_eq!(
        plan_summary(&plan(None, 0, 0)),
        "No changes. Everything already matches."
    );
}

// ---- term_line ----

/// R47: the term line is `name, start – end` with long dates (`format_date`).
#[test]
fn term_line_name_and_long_dates() {
    assert_eq!(
        term_line(&term("Semester 7")),
        "Semester 7, January 5, 2026 – May 1, 2026"
    );
}

// ---- course_add_line ----

/// R43: code, name, credits, and rule set joined by ` · `.
#[test]
fn course_add_line_with_credits_and_rule_set() {
    assert_eq!(
        course_add_line(&course_add(
            "CSE 4711",
            "Artificial Intelligence",
            Some(3.0),
            Some("Theory")
        )),
        "CSE 4711 Artificial Intelligence · 3 credits · Theory"
    );
}

/// R43: without credits and rule set, only code and name are shown.
#[test]
fn course_add_line_without_credits_or_rule_set() {
    assert_eq!(
        course_add_line(&course_add(
            "CSE 4711",
            "Artificial Intelligence",
            None,
            None
        )),
        "CSE 4711 Artificial Intelligence"
    );
}

/// R43: each optional part appears alone when only it is set; fractional credits keep the
/// fraction.
#[test]
fn course_add_line_one_optional_part() {
    assert_eq!(
        course_add_line(&course_add("CSE 4712", "AI Lab", Some(1.5), None)),
        "CSE 4712 AI Lab · 1.5 credits"
    );
    assert_eq!(
        course_add_line(&course_add("CSE 4712", "AI Lab", None, Some("Lab"))),
        "CSE 4712 AI Lab · Lab"
    );
}

// ---- field_change_line ----

/// R44: a field set from empty shows `—` for the old value.
#[test]
fn field_change_line_none_old_value_is_dash() {
    assert_eq!(
        field_change_line(&change(
            ImportField::Instructor,
            FieldValue::Text(None),
            text("Dr. Rahman")
        )),
        "Instructor: — → Dr. Rahman"
    );
}

/// R44: a cleared field (`null`) shows `—` for the new value.
#[test]
fn field_change_line_none_new_value_is_dash() {
    assert_eq!(
        field_change_line(&change(
            ImportField::MinAttendancePct,
            FieldValue::Integer(Some(75)),
            FieldValue::Integer(None)
        )),
        "Target: 75% → —"
    );
}

/// R44: credits print without a trailing `.0` for whole numbers and keep `1.5`.
#[test]
fn field_change_line_credits() {
    assert_eq!(
        field_change_line(&change(
            ImportField::Credits,
            FieldValue::Number(Some(3.0)),
            FieldValue::Number(Some(1.5))
        )),
        "Credits: 3 → 1.5"
    );
    assert_eq!(
        field_change_line(&change(
            ImportField::Credits,
            FieldValue::Number(None),
            FieldValue::Number(Some(3.0))
        )),
        "Credits: — → 3"
    );
}

/// R44: the rule set shows its name.
#[test]
fn field_change_line_rule_set() {
    let rule_set = |name: &str| {
        FieldValue::RuleSet(Some(RuleSetRef {
            id: 1,
            name: name.to_string(),
        }))
    };
    assert_eq!(
        field_change_line(&change(
            ImportField::RuleSet,
            rule_set("Theory"),
            rule_set("Lab")
        )),
        "Rule set: Theory → Lab"
    );
    assert_eq!(
        field_change_line(&change(
            ImportField::RuleSet,
            rule_set("Theory"),
            FieldValue::RuleSet(None)
        )),
        "Rule set: Theory → —"
    );
}

/// R44: name and room use their own labels; the target gets a `%` suffix on both sides.
#[test]
fn field_change_line_name_room_and_target() {
    assert_eq!(
        field_change_line(&change(
            ImportField::Name,
            text("Intro to AI"),
            text("Artificial Intelligence")
        )),
        "Name: Intro to AI → Artificial Intelligence"
    );
    assert_eq!(
        field_change_line(&change(
            ImportField::Room,
            text("301"),
            FieldValue::Text(None)
        )),
        "Room: 301 → —"
    );
    assert_eq!(
        field_change_line(&change(
            ImportField::MinAttendancePct,
            FieldValue::Integer(Some(75)),
            FieldValue::Integer(Some(80))
        )),
        "Target: 75% → 80%"
    );
}

// ---- slot_line ----

/// R45, R31: a slot with a room shows weekday, 12-hour range, and `Room N`.
#[test]
fn slot_line_with_room() {
    assert_eq!(
        slot_line(&import_slot(
            Weekday::Mon,
            t(10, 30),
            t(11, 45),
            Some("301")
        )),
        "Monday 10:30 AM–11:45 AM · Room 301"
    );
}

/// R45, R31: a slot without a room has no room part; afternoon times print as PM.
#[test]
fn slot_line_without_room() {
    assert_eq!(
        slot_line(&import_slot(Weekday::Fri, t(14, 30), t(15, 45), None)),
        "Friday 2:30 PM–3:45 PM"
    );
}

/// R31: noon prints as 12:00 PM and midnight as 12:00 AM in a slot line.
#[test]
fn slot_line_noon_and_midnight_edges() {
    assert_eq!(
        slot_line(&import_slot(Weekday::Wed, t(11, 0), t(12, 15), None)),
        "Wednesday 11:00 AM–12:15 PM"
    );
    assert_eq!(
        slot_line(&import_slot(Weekday::Sun, t(0, 0), t(1, 0), None)),
        "Sunday 12:00 AM–1:00 AM"
    );
}

// ---- slot_change_lines ----

/// R45, R7: an ended slot reads `Ends after <E-1>` and an added slot `From <E>`, in that order;
/// effective date 2026-01-14 gives January 13 and January 14.
#[test]
fn slot_change_lines_end_then_add() {
    let change = SlotChange {
        end: vec![stored_slot(Weekday::Mon, t(10, 30), t(11, 45), None)],
        add: vec![import_slot(Weekday::Mon, t(11, 0), t(12, 15), None)],
    };
    assert_eq!(
        slot_change_lines(&change, d(2026, 1, 14)),
        vec![
            "Ends after January 13, 2026: Monday 10:30 AM–11:45 AM".to_string(),
            "From January 14, 2026: Monday 11:00 AM–12:15 PM".to_string(),
        ]
    );
}

/// R45: with `"slots": []` only `Ends after …` lines appear, one per ended slot.
#[test]
fn slot_change_lines_end_only() {
    let change = SlotChange {
        end: vec![
            stored_slot(Weekday::Mon, t(10, 30), t(11, 45), None),
            stored_slot(Weekday::Thu, t(9, 0), t(10, 15), None),
        ],
        add: Vec::new(),
    };
    assert_eq!(
        slot_change_lines(&change, d(2026, 1, 14)),
        vec![
            "Ends after January 13, 2026: Monday 10:30 AM–11:45 AM".to_string(),
            "Ends after January 13, 2026: Thursday 9:00 AM–10:15 AM".to_string(),
        ]
    );
}

/// R45: when the course had no current slots only `From …` lines appear; the room shows as in
/// `slot_line`.
#[test]
fn slot_change_lines_add_only_with_room() {
    let change = SlotChange {
        end: Vec::new(),
        add: vec![import_slot(Weekday::Fri, t(14, 30), t(15, 45), Some("301"))],
    };
    assert_eq!(
        slot_change_lines(&change, d(2026, 1, 14)),
        vec!["From January 14, 2026: Friday 2:30 PM–3:45 PM · Room 301".to_string()]
    );
}

// ---- import_report_text ----

/// R48: the result line gives added and updated courses, then sessions created and removed.
#[test]
fn import_report_text_counts() {
    let report = ImportReport {
        plan: plan(None, 5, 1),
        sync: SyncReport {
            created: 72,
            deleted: 3,
        },
    };
    assert_eq!(
        import_report_text(&report),
        "Imported: 5 added, 1 updated. 72 sessions created, 3 removed."
    );
}

/// R48: zero counts are printed as zeros.
#[test]
fn import_report_text_zero_counts() {
    let report = ImportReport {
        plan: plan(None, 0, 0),
        sync: SyncReport {
            created: 0,
            deleted: 0,
        },
    };
    assert_eq!(
        import_report_text(&report),
        "Imported: 0 added, 0 updated. 0 sessions created, 0 removed."
    );
}

// ---- can_apply ----

/// R48: Apply is allowed when the preview matches the text and date, has no errors, and is
/// not empty.
#[test]
fn can_apply_true_for_matching_clean_nonempty_preview() {
    let p = preview("{}", "14-01-2026", false, false);
    assert!(can_apply(Some(&p), "{}", "14-01-2026"));
}

/// R48: Apply is blocked when no preview exists.
#[test]
fn can_apply_false_without_preview() {
    assert!(!can_apply(None, "{}", "14-01-2026"));
}

/// R48, D19: Apply is blocked when the pasted text changed after the preview.
#[test]
fn can_apply_false_when_text_changed() {
    let p = preview("{}", "14-01-2026", false, false);
    assert!(!can_apply(Some(&p), "{ }", "14-01-2026"));
}

/// R48, R49, D19: Apply is blocked when the effective-date text changed after the preview.
#[test]
fn can_apply_false_when_date_changed() {
    let p = preview("{}", "14-01-2026", false, false);
    assert!(!can_apply(Some(&p), "{}", "15-01-2026"));
}

/// R42, R48: Apply is blocked when the preview has errors.
#[test]
fn can_apply_false_when_preview_has_errors() {
    let p = preview("{}", "14-01-2026", true, false);
    assert!(!can_apply(Some(&p), "{}", "14-01-2026"));
}

/// R48: Apply is blocked when the plan is empty.
#[test]
fn can_apply_false_when_plan_is_empty() {
    let p = preview("{}", "14-01-2026", false, true);
    assert!(!can_apply(Some(&p), "{}", "14-01-2026"));
}
