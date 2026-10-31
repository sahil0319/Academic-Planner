//! M20: export courses and routine as JSON (R96-R100, docs/spec.md §13.15, T92-T95).
//!
//! Fixtures: term 2026-01-05 (Mon) to 2026-02-01 (Sun), today = 2026-01-12 (Mon), which is an
//! even term week (R54), so an even slot meets "this_week" and an odd slot "next_week".

mod common;

use chrono::{NaiveDate, Weekday};
use common::*;
use planner_core::{
    term_week_parity, ComponentKind, CourseInput, Error, Planner, RuleComponentInput, RuleSetInput,
    SlotInput, TermInput, WeekParity,
};
use serde_json::Value;

fn today() -> NaiveDate {
    d("2026-01-12")
}

fn term() -> TermInput {
    TermInput {
        id: None,
        name: "Winter 2026".into(),
        start_date: d("2026-01-05"),
        end_date: d("2026-02-01"),
    }
}

fn rule_set_id(p: &Planner, name: &str) -> i64 {
    p.list_rule_sets()
        .unwrap()
        .into_iter()
        .find(|s| s.name == name)
        .unwrap_or_else(|| panic!("no rule set {name}"))
        .id
}

fn slot(
    course_id: i64,
    day: Weekday,
    start: &str,
    end: &str,
    parity: Option<WeekParity>,
) -> SlotInput {
    SlotInput {
        course_id,
        weekday: day,
        start_time: t(start),
        end_time: t(end),
        room: None,
        valid_from: d("2026-01-05"),
        valid_to: None,
        week_parity: parity,
    }
}

/// Planner A: CSE101 (graded Theory, weekly Mon slot with a room plus an even bi-weekly Wed
/// slot), ABC200 (graded Lab, one slot ended before today and one current odd Tue slot), and
/// the archived OLD300. Returns the planner and the ids of CSE101 and ABC200.
fn planner_a() -> (Planner, i64, i64) {
    let mut p = Planner::open_in_memory().unwrap();
    p.save_term(term()).unwrap();
    let theory = rule_set_id(&p, "Theory");
    let lab = rule_set_id(&p, "Lab");

    let cse = p
        .create_course(CourseInput {
            code: "CSE101".into(),
            name: "Intro to Programming".into(),
            instructor: Some("Dr. Rahman".into()),
            room: Some("A-101".into()),
            notes: Some("private note".into()),
            min_attendance_pct: Some(75),
            credits: Some(3.0),
            rule_set_id: Some(theory),
        })
        .unwrap();
    p.add_slot(SlotInput {
        room: Some("B-2".into()),
        ..slot(cse.id, Weekday::Mon, "09:00", "10:15", None)
    })
    .unwrap();
    p.add_slot(slot(
        cse.id,
        Weekday::Wed,
        "14:30",
        "16:00",
        Some(WeekParity::Even),
    ))
    .unwrap();

    let abc = p
        .create_course(CourseInput {
            code: "ABC200".into(),
            name: "Lab Work".into(),
            instructor: None,
            room: None,
            notes: None,
            min_attendance_pct: None,
            credits: Some(1.5),
            rule_set_id: Some(lab),
        })
        .unwrap();
    p.add_slot(SlotInput {
        valid_to: Some(d("2026-01-11")),
        ..slot(abc.id, Weekday::Mon, "08:00", "09:00", None)
    })
    .unwrap();
    p.add_slot(slot(
        abc.id,
        Weekday::Tue,
        "11:00",
        "12:30",
        Some(WeekParity::Odd),
    ))
    .unwrap();

    let old = p.create_course(course_input("OLD300")).unwrap();
    p.add_slot(slot(old.id, Weekday::Fri, "10:00", "11:00", None))
        .unwrap();
    p.set_archived(old.id, true).unwrap();

    (p, cse.id, abc.id)
}

fn export(p: &Planner, ids: Option<&[i64]>) -> Value {
    serde_json::from_str(&p.export_courses_json(ids, today()).unwrap()).unwrap()
}

fn course<'a>(doc: &'a Value, code: &str) -> &'a Value {
    doc["courses"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["code"] == code)
        .unwrap_or_else(|| panic!("no course {code}"))
}

// ---- T92: R96, R97 ----

/// R96, R97: archived and ended slots are absent; formats and keys are right; personal data
/// is not exported.
#[test]
fn t92_export_all_non_archived_courses() {
    let (p, _, _) = planner_a();
    let text = p.export_courses_json(None, today()).unwrap();
    assert!(text.contains('\n'), "pretty-printed: {text}");
    let doc: Value = serde_json::from_str(&text).unwrap();

    assert_eq!(doc["format"], "academic-planner-courses");
    assert_eq!(doc["version"], 1);
    assert_eq!(doc["term"]["name"], "Winter 2026");
    assert_eq!(doc["term"]["start"], "2026-01-05");
    assert_eq!(doc["term"]["end"], "2026-02-01");

    let codes: Vec<&str> = doc["courses"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["code"].as_str().unwrap())
        .collect();
    assert_eq!(codes, ["ABC200", "CSE101"], "ordered by code, no archived");

    let cse = course(&doc, "CSE101");
    assert_eq!(cse["name"], "Intro to Programming");
    assert_eq!(cse["credits"], 3.0);
    assert_eq!(cse["rule_set"], "Theory");
    assert_eq!(cse["instructor"], "Dr. Rahman");
    assert_eq!(cse["room"], "A-101");
    assert_eq!(cse["min_attendance_pct"], 75);
    let slots = cse["slots"].as_array().unwrap();
    assert_eq!(slots.len(), 2);
    assert_eq!(slots[0]["day"], "Mon");
    assert_eq!(slots[0]["start"], "09:00");
    assert_eq!(slots[0]["end"], "10:15");
    assert_eq!(slots[0]["room"], "B-2");
    assert!(slots[0].get("biweekly").is_none(), "weekly slot: no key");
    assert_eq!(slots[1]["day"], "Wed");
    assert_eq!(slots[1]["start"], "14:30");
    assert_eq!(slots[1]["biweekly"], "this_week");

    let abc = course(&doc, "ABC200");
    assert_eq!(abc["credits"], 1.5);
    assert_eq!(abc["rule_set"], "Lab");
    assert_eq!(abc["instructor"], Value::Null);
    assert_eq!(abc["min_attendance_pct"], Value::Null);
    let slots = abc["slots"].as_array().unwrap();
    assert_eq!(slots.len(), 1, "the ended Monday slot is absent");
    assert_eq!(slots[0]["day"], "Tue");
    assert_eq!(slots[0]["biweekly"], "next_week");

    for banned in ["private note", "\"id\"", "notes", "session", "OLD300"] {
        assert!(!text.contains(banned), "{banned} must not be exported");
    }
}

/// R96, R97: a course that is not graded has no `credits` and no `rule_set` key; with no
/// active term the `term` key is omitted.
#[test]
fn t92_ungraded_course_and_missing_term_omit_keys() {
    let mut p = Planner::open_in_memory().unwrap();
    let doc = export(&p, None);
    assert!(doc.get("term").is_none());
    assert_eq!(doc["courses"], Value::Array(vec![]));

    p.save_term(term()).unwrap();
    p.create_course(course_input("PLAIN1")).unwrap();
    let doc = export(&p, None);
    let c = course(&doc, "PLAIN1");
    assert!(c.get("credits").is_none());
    assert!(c.get("rule_set").is_none());
    assert_eq!(c["slots"], Value::Array(vec![]));
}

// ---- T93: R96 ----

/// R96: a list of ids limits the export; an unknown id is `NotFound`; an empty list gives an
/// empty `courses` array.
#[test]
fn t93_export_selected_courses() {
    let (p, cse, _) = planner_a();
    assert_eq!(export(&p, None)["courses"].as_array().unwrap().len(), 2);

    let doc = export(&p, Some(&[cse]));
    let courses = doc["courses"].as_array().unwrap();
    assert_eq!(courses.len(), 1);
    assert_eq!(courses[0]["code"], "CSE101");

    let err = p.export_courses_json(Some(&[999]), today()).unwrap_err();
    assert!(matches!(err, Error::NotFound { .. }), "{err:?}");

    let doc = export(&p, Some(&[]));
    assert_eq!(doc["courses"], Value::Array(vec![]));
}

// ---- T94: R98 ----

/// R98: an export imported into an empty planner (effective date = today) recreates term,
/// courses, slots, and the bi-weekly pattern; previewing it again is empty in both planners.
#[test]
fn t94_round_trip_through_import() {
    let (a, _, _) = planner_a();
    let text = a.export_courses_json(None, today()).unwrap();

    let mut b = Planner::open_in_memory().unwrap();
    b.apply_import(&text, today(), today()).unwrap();

    let term = b.active_term().unwrap().unwrap();
    assert_eq!(term.name, "Winter 2026");
    assert_eq!(term.start_date, d("2026-01-05"));
    assert_eq!(term.end_date, d("2026-02-01"));

    let courses = b.list_courses(false).unwrap();
    let codes: Vec<&str> = courses.iter().map(|c| c.code.as_str()).collect();
    assert_eq!(codes, ["ABC200", "CSE101"]);
    let cse = courses.iter().find(|c| c.code == "CSE101").unwrap();
    assert_eq!(cse.credits, Some(3.0));
    assert_eq!(cse.min_attendance_pct, Some(75));
    assert_eq!(cse.rule_set_id, Some(rule_set_id(&b, "Theory")));
    assert_eq!(cse.notes, None);

    let slots = b.list_slots(today()).unwrap();
    assert_eq!(slots.len(), 3);
    let wed = slots.iter().find(|s| s.weekday == Weekday::Wed).unwrap();
    assert_eq!(
        wed.week_parity,
        Some(term_week_parity(term.start_date, today()))
    );
    let tue = slots.iter().find(|s| s.weekday == Weekday::Tue).unwrap();
    assert_eq!(tue.week_parity, Some(WeekParity::Odd));

    assert!(a.preview_import(&text, today()).unwrap().is_empty());
    assert!(b.preview_import(&text, today()).unwrap().is_empty());
    assert_eq!(b.export_courses_json(None, today()).unwrap(), text);
}

// ---- T95: R98 ----

/// R98, R42: a rule set name the other planner does not have is an error at its path and
/// nothing is written.
#[test]
fn t95_unknown_rule_set_is_a_preview_error() {
    let (mut a, cse, _) = planner_a();
    let seminar = a
        .save_rule_set(RuleSetInput {
            id: None,
            name: "Seminar".into(),
            components: vec![RuleComponentInput {
                id: None,
                name: "Exam".into(),
                kind: ComponentKind::Items,
                weight_pct: 100.0,
                item_count: Some(1),
                best_count: Some(1),
                item_max: None,
                is_final: true,
                bands: Vec::new(),
            }],
        })
        .unwrap();
    a.update_course(
        cse,
        CourseInput {
            name: "Intro to Programming".into(),
            credits: Some(3.0),
            rule_set_id: Some(seminar.id),
            ..course_input("CSE101")
        },
    )
    .unwrap();

    let text = a.export_courses_json(Some(&[cse]), today()).unwrap();
    let mut b = Planner::open_in_memory().unwrap();
    let plan = b.preview_import(&text, today()).unwrap();
    assert_eq!(plan.errors.len(), 1, "{:?}", plan.errors);
    assert_eq!(plan.errors[0].path, "courses[0].rule_set");
    assert!(plan.errors[0].message.contains("Seminar"));

    let err = b.apply_import(&text, today(), today()).unwrap_err();
    assert!(matches!(err, Error::Validation(_)), "{err:?}");
    assert!(b.active_term().unwrap().is_none(), "nothing written");
}
