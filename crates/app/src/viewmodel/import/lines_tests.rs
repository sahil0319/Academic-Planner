//! Unit tests for `preview_lines` (the result area of the import panel, R42-R48). No database:
//! core types are built directly.

use super::*;
use chrono::{NaiveTime, Weekday};
use planner_core::{CourseUpdate, FieldValue, ImportField, RuleSetRef, Slot};

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

fn empty_plan() -> ImportPlan {
    ImportPlan {
        term: None,
        add: Vec::new(),
        update: Vec::new(),
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn texts(lines: &[PreviewLine]) -> Vec<(LineKind, &str)> {
    lines.iter().map(|l| (l.kind, l.text.as_str())).collect()
}

/// R42: errors replace the plan sections and come after a counted heading; warnings follow.
#[test]
fn preview_lines_errors_show_heading_errors_and_warnings() {
    let mut plan = empty_plan();
    plan.errors = vec![issue("courses[0].code", "required")];
    plan.warnings = vec![issue("comment", "unknown key")];
    assert_eq!(
        texts(&preview_lines(&plan, d(2026, 1, 14))),
        vec![
            (LineKind::Heading, "1 error — nothing will be imported"),
            (LineKind::Error, "courses[0].code: required"),
            (LineKind::Section, "Warnings"),
            (LineKind::Warning, "comment: unknown key"),
        ]
    );
}

/// R48: an empty plan shows only the no-changes summary.
#[test]
fn preview_lines_empty_plan_is_one_summary_line() {
    assert_eq!(
        texts(&preview_lines(&empty_plan(), d(2026, 1, 14))),
        vec![(LineKind::Summary, "No changes. Everything already matches.")]
    );
}

/// R43, R45, R47: term, added courses with their slots, and updated courses with field and
/// slot changes each get their own section; an archived match carries a tag.
#[test]
fn preview_lines_term_add_and_update_sections() {
    let mut plan = empty_plan();
    plan.term = Some(ImportTerm {
        name: "Semester 7".to_string(),
        start_date: d(2026, 1, 5),
        end_date: d(2026, 5, 1),
    });
    plan.add = vec![CourseAdd {
        code: "CSE 4711".to_string(),
        name: "AI".to_string(),
        instructor: None,
        room: None,
        min_attendance_pct: None,
        credits: Some(3.0),
        rule_set: Some(RuleSetRef {
            id: 1,
            name: "Theory".to_string(),
        }),
        slots: vec![ImportSlot {
            weekday: Weekday::Mon,
            start: t(10, 30),
            end: t(11, 45),
            room: None,
        }],
    }];
    plan.update = vec![CourseUpdate {
        course_id: 2,
        code: "EEE 1101".to_string(),
        archived: true,
        changes: vec![FieldChange {
            field: ImportField::Room,
            old: FieldValue::Text(None),
            new: FieldValue::Text(Some("301".to_string())),
        }],
        slots: Some(SlotChange {
            end: vec![Slot {
                id: 1,
                course_id: 2,
                weekday: Weekday::Tue,
                start_time: t(9, 0),
                end_time: t(10, 15),
                room: None,
                valid_from: d(2026, 1, 5),
                valid_to: None,
                week_parity: None,
            }],
            add: Vec::new(),
        }),
    }];
    assert_eq!(
        texts(&preview_lines(&plan, d(2026, 1, 14))),
        vec![
            (
                LineKind::Summary,
                "Creates term \"Semester 7\" · adds 1 course · updates 1 course"
            ),
            (LineKind::Section, "Term to create"),
            (LineKind::Item, "Semester 7, January 5, 2026 – May 1, 2026"),
            (LineKind::Section, "Courses to add"),
            (LineKind::Item, "CSE 4711 AI · 3 credits · Theory"),
            (LineKind::Detail, "Monday 10:30 AM–11:45 AM"),
            (LineKind::Section, "Courses to update"),
            (LineKind::Item, "EEE 1101 (archived)"),
            (LineKind::Detail, "Room: — → 301"),
            (
                LineKind::Detail,
                "Ends after January 13, 2026: Tuesday 9:00 AM–10:15 AM"
            ),
        ]
    );
}
