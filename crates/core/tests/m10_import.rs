//! M10: course and timetable JSON import (R42-R50, docs/spec.md §13.4, T44-T50).
//!
//! Fixtures that name the seeded rule sets "Theory" and "Lab" need M9's `list_rule_sets`.

mod common;

use chrono::Weekday;
use common::*;
use planner_core::{
    Course, Error, ImportIssue, ImportPlan, ImportReport, Planner, Slot, SlotInput, TermInput,
};
use serde_json::{json, Value};

/// Effective date used by most tests: a Wednesday inside the Semester 7 term.
const EFFECTIVE: &str = "2026-01-14";

/// In-memory planner with no term (M9 seeds the rule sets).
fn fresh() -> Planner {
    Planner::open_in_memory().unwrap()
}

/// In-memory planner with an active term.
fn with_term(name: &str, start: &str, end: &str) -> Planner {
    let mut p = fresh();
    p.save_term(TermInput {
        id: None,
        name: name.into(),
        start_date: d(start),
        end_date: d(end),
    })
    .unwrap();
    p
}

/// Planner with the active term "Semester 7" (2026-01-05 to 2026-05-01), the term of `term_json`.
fn semester() -> Planner {
    with_term("Semester 7", "2026-01-05", "2026-05-01")
}

/// A course in the active term, optionally archived.
#[allow(dead_code)] // used by later test groups
fn add_course(p: &mut Planner, code: &str, archived: bool) -> Course {
    let course = p.create_course(course_input(code)).unwrap();
    if archived {
        p.set_archived(course.id, true).unwrap();
    }
    p.list_courses(true)
        .unwrap()
        .into_iter()
        .find(|c| c.id == course.id)
        .unwrap()
}

/// A weekly slot for a course, valid from `from`.
#[allow(dead_code)] // used by later test groups
fn add_slot(
    p: &mut Planner,
    course: &Course,
    day: Weekday,
    start: &str,
    end: &str,
    room: Option<&str>,
    from: &str,
) -> Slot {
    p.add_slot(SlotInput {
        course_id: course.id,
        weekday: day,
        start_time: t(start),
        end_time: t(end),
        room: room.map(String::from),
        valid_from: d(from),
        valid_to: None,
    })
    .unwrap()
}

/// The whole database as export JSON at a fixed moment: equal text means an unchanged database.
fn snapshot(p: &Planner) -> String {
    p.export_json(dt("2026-01-14T00:00")).unwrap()
}

/// Import text from a JSON value.
fn doc(v: Value) -> String {
    v.to_string()
}

fn term_json() -> Value {
    json!({"name": "Semester 7", "start": "2026-01-05", "end": "2026-05-01"})
}

/// A valid document without a term and with one minimal course.
fn base() -> Value {
    json!({
        "format": "academic-planner-courses",
        "version": 1,
        "courses": [{"code": "CSE 4711", "name": "Artificial Intelligence"}]
    })
}

/// A valid document with the Semester 7 term and the given courses.
#[allow(dead_code)] // used by later test groups
fn with_courses(courses: Value) -> String {
    doc(json!({
        "format": "academic-planner-courses",
        "version": 1,
        "term": term_json(),
        "courses": courses
    }))
}

/// A valid document without a term and the given courses.
fn no_term(courses: Value) -> String {
    doc(json!({
        "format": "academic-planner-courses",
        "version": 1,
        "courses": courses
    }))
}

/// A document with one course `CSE 4711` named "Artificial Intelligence" plus `extra` keys.
fn course_with(extra: Value) -> String {
    let mut course = json!({"code": "CSE 4711", "name": "Artificial Intelligence"});
    course
        .as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    no_term(json!([course]))
}

/// A document with one new course that has the single slot `slot`.
fn one_slot(slot: Value) -> String {
    course_with(json!({"slots": [slot]}))
}

fn effective() -> chrono::NaiveDate {
    d(EFFECTIVE)
}

fn preview(p: &Planner, text: &str) -> ImportPlan {
    p.preview_import(text, effective()).unwrap()
}

fn apply(p: &mut Planner, text: &str) -> planner_core::Result<ImportReport> {
    p.apply_import(text, effective(), effective())
}

/// Sorted paths of the issues.
fn paths(issues: &[ImportIssue]) -> Vec<String> {
    let mut out: Vec<String> = issues.iter().map(|i| i.path.clone()).collect();
    out.sort();
    out
}

/// The text is invalid: preview reports an error at `path` and plans nothing, `apply_import`
/// returns `Error::Validation` naming the path, and the database is unchanged. Returns the issue.
fn assert_invalid(p: &mut Planner, text: &str, path: &str) -> ImportIssue {
    let before = snapshot(p);
    let plan = preview(p, text);
    let issue = plan
        .errors
        .iter()
        .find(|e| e.path == path)
        .unwrap_or_else(|| panic!("no error at {path}; errors: {:?}", plan.errors))
        .clone();
    assert!(plan.add.is_empty() && plan.update.is_empty() && plan.term.is_none());
    match apply(p, text) {
        Err(Error::Validation(message)) => assert!(message.contains(path), "{message}"),
        other => panic!("expected Error::Validation, got {other:?}"),
    }
    assert_eq!(snapshot(p), before, "database changed by an invalid file");
    issue
}

/// The text previews without errors; returns the plan.
fn assert_valid(p: &Planner, text: &str) -> ImportPlan {
    let plan = preview(p, text);
    assert!(plan.errors.is_empty(), "{:?}", plan.errors);
    plan
}

// ---- US3: validation (R42) ----

/// R42: all four mistakes are reported at once, each with its path; `apply_import` writes
/// nothing and its error names every path (T48).
#[test]
fn t48_errors_collected_with_paths_db_unchanged() {
    let mut p = semester();
    let text = no_term(json!([
        {"code": "CSE 4711", "name": "A", "slots": [{"day": "Mo", "start": "10:00", "end": "11:00"}]},
        {"code": "CSE 4712", "name": "B", "rule_set": "Foo",
         "slots": [{"day": "Mon", "start": "10:00", "end": "09:00"}]},
        {"code": "cse 4711", "name": "C"}
    ]));
    let before = snapshot(&p);
    let plan = preview(&p, &text);
    let expected = [
        "courses[0].slots[0].day",
        "courses[1].rule_set",
        "courses[1].slots[0].end",
        "courses[2].code",
    ];
    assert_eq!(paths(&plan.errors), expected);
    assert!(plan.add.is_empty() && plan.update.is_empty() && plan.term.is_none());
    match apply(&mut p, &text) {
        Err(Error::Validation(message)) => {
            for path in expected {
                assert!(message.contains(path), "{path} missing from {message}");
            }
        }
        other => panic!("expected Error::Validation, got {other:?}"),
    }
    assert_eq!(snapshot(&p), before);
}

/// R42: text that is not JSON gives one document error with line and column.
#[test]
fn r42_not_json_is_document_error() {
    let mut p = semester();
    let issue = assert_invalid(&mut p, "{\n  \"format\": ", "document");
    assert!(issue.message.contains("line"), "{}", issue.message);
    assert!(issue.message.contains("column"), "{}", issue.message);
}

/// R42: a JSON root that is not an object is a document error.
#[test]
fn r42_root_not_object() {
    let mut p = semester();
    assert_invalid(&mut p, "[]", "document");
    assert_invalid(&mut p, "\"text\"", "document");
}

/// R42: `format` must be present and equal "academic-planner-courses".
#[test]
fn r42_format_missing_or_wrong() {
    let mut p = semester();
    let mut missing = base();
    missing.as_object_mut().unwrap().remove("format");
    assert_invalid(&mut p, &doc(missing), "format");
    for wrong in [json!("other"), json!(5), json!(null)] {
        let mut v = base();
        v["format"] = wrong;
        assert_invalid(&mut p, &doc(v), "format");
    }
}

/// R42: `version` must be the integer 1; `2` and the text "1" are errors.
#[test]
fn r42_version_not_1() {
    let mut p = semester();
    let mut missing = base();
    missing.as_object_mut().unwrap().remove("version");
    assert_invalid(&mut p, &doc(missing), "version");
    for wrong in [json!(2), json!("1")] {
        let mut v = base();
        v["version"] = wrong;
        assert_invalid(&mut p, &doc(v), "version");
    }
}

/// R42: `courses` must be present and a list.
#[test]
fn r42_courses_missing_or_not_list() {
    let mut p = semester();
    let mut missing = base();
    missing.as_object_mut().unwrap().remove("courses");
    assert_invalid(&mut p, &doc(missing), "courses");
    for wrong in [json!({}), json!("x"), json!(null)] {
        let mut v = base();
        v["courses"] = wrong;
        assert_invalid(&mut p, &doc(v), "courses");
    }
}

/// R42: every entry of `courses` must be an object.
#[test]
fn r42_course_not_object() {
    let mut p = semester();
    assert_invalid(&mut p, &no_term(json!(["CSE 4711"])), "courses[0]");
    assert_invalid(&mut p, &no_term(json!([null])), "courses[0]");
}

/// R42: `code` is required: missing, null, and blank are errors.
#[test]
fn r42_code_missing_null_or_blank() {
    let mut p = semester();
    assert_invalid(&mut p, &no_term(json!([{"name": "AI"}])), "courses[0].code");
    for bad in [json!(null), json!("   "), json!("")] {
        let text = no_term(json!([{"code": bad, "name": "AI"}]));
        assert_invalid(&mut p, &text, "courses[0].code");
    }
}

/// R42: a new course needs a name; absent, `null`, and blank are errors.
#[test]
fn r42_new_course_needs_name() {
    let mut p = semester();
    let absent = assert_invalid(
        &mut p,
        &no_term(json!([{"code": "CSE 4711"}])),
        "courses[0].name",
    );
    assert!(
        absent.message.contains("required for a new course"),
        "{}",
        absent.message
    );
    for bad in [json!(null), json!("  ")] {
        let text = no_term(json!([{"code": "CSE 4711", "name": bad}]));
        assert_invalid(&mut p, &text, "courses[0].name");
    }
}

/// R42: `slots: null` is an error that points to `[]` for "end all slots".
#[test]
fn r42_slots_null_is_error() {
    let mut p = semester();
    let issue = assert_invalid(
        &mut p,
        &course_with(json!({"slots": null})),
        "courses[0].slots",
    );
    assert!(
        issue.message.contains("use [] to end all slots"),
        "{}",
        issue.message
    );
}

/// R42: a malformed `term` is an error even when an active term makes it unnecessary.
#[test]
fn r42_term_malformed_with_active_term_is_error() {
    let mut p = semester();
    let mut v = base();
    v["term"] = json!({"name": "S", "start": "2026-1-5", "end": "2026-05-01"});
    assert_invalid(&mut p, &doc(v), "term.start");
}

/// R42: `term.end` before `term.start` is an error.
#[test]
fn r42_term_end_before_start() {
    let mut p = fresh();
    let mut v = base();
    v["term"] = json!({"name": "S", "start": "2026-05-01", "end": "2026-01-05"});
    assert_invalid(&mut p, &doc(v), "term.end");
}

/// R42: errors in different courses are all reported, not just the first.
#[test]
fn r42_errors_collected_across_courses() {
    let mut p = semester();
    let text = no_term(json!([
        {"code": "A 100", "name": "A", "credits": 0},
        {"code": "B 100", "name": "B"},
        {"code": "C 100", "name": "C"},
        {"code": "D 100", "name": "D", "min_attendance_pct": 101}
    ]));
    let plan = preview(&p, &text);
    assert_eq!(
        paths(&plan.errors),
        ["courses[0].credits", "courses[3].min_attendance_pct"]
    );
    assert_invalid(&mut p, &text, "courses[0].credits");
}

/// R42: `day` accepts names in any case, trimmed, and 1-7 as text or integer.
#[test]
fn r42_day_variants_accepted() {
    let p = semester();
    for day in [
        json!("Mon"),
        json!("monday"),
        json!("MONDAY"),
        json!(" Mon "),
        json!("1"),
        json!(1),
    ] {
        let plan = assert_valid(
            &p,
            &one_slot(json!({"day": day, "start": "10:00", "end": "11:00"})),
        );
        assert_eq!(
            plan.add[0].slots[0].weekday,
            Weekday::Mon,
            "day {day} should be Monday"
        );
    }
    for day in [json!("Sun"), json!(7)] {
        let plan = assert_valid(
            &p,
            &one_slot(json!({"day": day, "start": "10:00", "end": "11:00"})),
        );
        assert_eq!(plan.add[0].slots[0].weekday, Weekday::Sun, "day {day}");
    }
}

/// R42: bad `day` values are errors at the `.day` path.
#[test]
fn r42_day_variants_rejected() {
    let mut p = semester();
    for day in [
        json!("Mo"),
        json!("Tues"),
        json!(0),
        json!(8),
        json!("0"),
        json!(1.5),
        json!(true),
    ] {
        let text = one_slot(json!({"day": day, "start": "10:00", "end": "11:00"}));
        assert_invalid(&mut p, &text, "courses[0].slots[0].day");
    }
}

/// R42: times are 24-hour `H:MM` or `HH:MM`; other forms are errors at `.start` or `.end`.
#[test]
fn r42_times() {
    let mut p = semester();
    for end in ["9:30", "09:30", "23:59"] {
        let plan = assert_valid(
            &p,
            &one_slot(json!({"day": "Mon", "start": "00:00", "end": end})),
        );
        assert_eq!(plan.add[0].slots[0].end, t(&format!("{end:0>5}")));
    }
    let bad = [
        json!("24:00"),
        json!("9:5"),
        json!("10:30 AM"),
        json!("10:30:00"),
        json!(930),
    ];
    for value in bad {
        let start = one_slot(json!({"day": "Mon", "start": value, "end": "23:00"}));
        assert_invalid(&mut p, &start, "courses[0].slots[0].start");
        let end = one_slot(json!({"day": "Mon", "start": "01:00", "end": value}));
        assert_invalid(&mut p, &end, "courses[0].slots[0].end");
    }
}

/// R42: dates must be `YYYY-MM-DD`.
#[test]
fn r42_dates() {
    let mut p = fresh();
    for bad in ["2026-1-5", "05/01/2026"] {
        let mut v = base();
        v["term"] = json!({"name": "S", "start": bad, "end": "2026-05-01"});
        assert_invalid(&mut p, &doc(v), "term.start");
        let mut v = base();
        v["term"] = json!({"name": "S", "start": "2026-01-05", "end": bad});
        assert_invalid(&mut p, &doc(v), "term.end");
    }
}

/// R42: `credits` is a number greater than 0; `1.5` is fine, `0`, `-1`, and "3" are not.
#[test]
fn r42_credits() {
    let mut p = semester();
    for bad in [json!(0), json!(-1), json!("3")] {
        assert_invalid(
            &mut p,
            &course_with(json!({"credits": bad})),
            "courses[0].credits",
        );
    }
    let plan = assert_valid(&p, &course_with(json!({"credits": 1.5})));
    assert_eq!(plan.add[0].credits, Some(1.5));
}

/// R42: `min_attendance_pct` is a whole number from 0 to 100.
#[test]
fn r42_min_attendance_pct() {
    let mut p = semester();
    for bad in [json!(101), json!(-1), json!(75.5), json!("75")] {
        assert_invalid(
            &mut p,
            &course_with(json!({"min_attendance_pct": bad})),
            "courses[0].min_attendance_pct",
        );
    }
    for good in [0, 100] {
        let plan = assert_valid(&p, &course_with(json!({"min_attendance_pct": good})));
        assert_eq!(plan.add[0].min_attendance_pct, Some(good));
    }
}

/// R42: an unknown `rule_set` is an error that lists the available names.
#[test]
fn r42_rule_set_unknown_lists_names() {
    let mut p = semester();
    let issue = assert_invalid(
        &mut p,
        &course_with(json!({"rule_set": "Foo"})),
        "courses[0].rule_set",
    );
    assert!(issue.message.contains("Theory"), "{}", issue.message);
    assert!(issue.message.contains("Lab"), "{}", issue.message);
}

/// R42: unknown keys at the root, `term`, course, and slot level are warnings with exact paths,
/// not errors; the plan is the same as without them and apply ignores them.
#[test]
fn r42_unknown_keys_warn_not_error() {
    let mut p = semester();
    let existing = p
        .create_course(planner_core::CourseInput {
            notes: Some("keep me".into()),
            ..course_input("CSE 4711")
        })
        .unwrap();
    let clean = doc(json!({
        "format": "academic-planner-courses", "version": 1,
        "term": term_json(),
        "courses": [{"code": "CSE 4711", "slots": [
            {"day": "Mon", "start": "10:30", "end": "11:45"},
            {"day": "Fri", "start": "14:30", "end": "15:45"}
        ]}]
    }));
    let noisy = doc(json!({
        "format": "academic-planner-courses", "version": 1, "comment": "hi",
        "term": {"name": "Semester 7", "start": "2026-01-05", "end": "2026-05-01", "id": 9},
        "courses": [{
            "code": "CSE 4711", "teacher": "Dr X", "notes": "overwritten",
            "archived": true, "slots": [
                {"day": "Mon", "start": "10:30", "end": "11:45"},
                {"day": "Fri", "start": "14:30", "end": "15:45", "type": "lab"}
            ]
        }]
    }));
    let plan = preview(&p, &noisy);
    assert!(plan.errors.is_empty(), "{:?}", plan.errors);
    assert_eq!(
        paths(&plan.warnings),
        [
            "comment",
            "courses[0].archived",
            "courses[0].notes",
            "courses[0].slots[1].type",
            "courses[0].teacher",
            "term.id"
        ]
    );
    let clean_plan = preview(&p, &clean);
    assert_eq!(
        ImportPlan {
            warnings: vec![],
            ..plan
        },
        ImportPlan {
            warnings: vec![],
            ..clean_plan
        }
    );
    apply(&mut p, &noisy).unwrap();
    let stored = p
        .list_courses(true)
        .unwrap()
        .into_iter()
        .find(|c| c.id == existing.id)
        .unwrap();
    assert_eq!(stored.notes.as_deref(), Some("keep me"));
    assert!(!stored.archived);
}
