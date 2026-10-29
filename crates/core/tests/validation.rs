//! T24 / R2: every invalid input returns `Error::Validation` and stores nothing.

mod common;

use common::*;
use planner_core::{
    AssessmentFilter, AssessmentInput, AssessmentKind, CourseInput, Error, Planner, Result,
    SlotInput, TermInput,
};

fn assert_validation<T: std::fmt::Debug>(result: Result<T>) {
    assert!(
        matches!(result, Err(Error::Validation(_))),
        "expected Error::Validation, got {result:?}"
    );
}

fn assessment(course_id: i64, title: &str) -> AssessmentInput {
    AssessmentInput {
        course_id,
        kind: AssessmentKind::Assignment,
        title: title.into(),
        due_at: dt("2026-01-16T23:59"),
        notes: None,
        remind_minutes_before: None,
        component_id: None,
        score: None,
    }
}

fn other_term() -> TermInput {
    TermInput {
        id: None,
        name: "Summer 2026".into(),
        start_date: d("2026-05-04"),
        end_date: d("2026-07-31"),
    }
}

#[test]
fn t24_term_end_before_start() {
    let mut p = Planner::open_in_memory().unwrap();
    let bad = TermInput {
        end_date: d("2026-01-04"),
        ..fixture_term()
    };
    assert_validation(p.save_term(bad.clone()));
    assert_eq!(p.active_term().unwrap(), None);

    // Updating an existing term to invalid dates leaves it unchanged.
    let term = p.save_term(fixture_term()).unwrap();
    assert_validation(p.save_term(TermInput {
        id: Some(term.id),
        ..bad
    }));
    assert_eq!(p.active_term().unwrap(), Some(term));
}

#[test]
fn t24_term_end_equal_start_is_valid() {
    let mut p = Planner::open_in_memory().unwrap();
    let one_day = TermInput {
        end_date: d("2026-01-05"),
        ..fixture_term()
    };
    assert!(p.save_term(one_day).is_ok());
}

#[test]
fn t24_slot_end_not_after_start() {
    let (mut p, _, cse) = fixture();
    for end in ["09:00", "08:59"] {
        assert_validation(p.add_slot(SlotInput {
            end_time: t(end),
            ..slot_a(cse.id)
        }));
    }
    assert!(p.list_slots(d("2026-01-12")).unwrap().is_empty());
}

#[test]
fn t24_attendance_target_out_of_range() {
    let (mut p, _, cse) = fixture();
    for pct in [-1, 101] {
        assert_validation(p.create_course(CourseInput {
            min_attendance_pct: Some(pct),
            credits: None,
            rule_set_id: None,
            ..course_input("MTH110")
        }));
        assert_validation(p.update_course(
            cse.id,
            CourseInput {
                min_attendance_pct: Some(pct),
                credits: None,
                rule_set_id: None,
                ..fixture_course()
            },
        ));
    }
    assert_eq!(p.list_courses(true).unwrap(), [cse]);
}

#[test]
fn t24_target_bounds_and_none_are_valid() {
    let (mut p, _, _) = fixture();
    for (code, pct) in [("A0", Some(0)), ("A100", Some(100)), ("ANONE", None)] {
        let course = p
            .create_course(CourseInput {
                min_attendance_pct: pct,
                credits: None,
                rule_set_id: None,
                ..course_input(code)
            })
            .unwrap();
        assert_eq!(course.min_attendance_pct, pct);
    }
}

#[test]
fn t24_empty_course_code() {
    let (mut p, _, cse) = fixture();
    for code in ["", "   "] {
        assert_validation(p.create_course(course_input(code)));
        assert_validation(p.update_course(
            cse.id,
            CourseInput {
                code: code.into(),
                ..fixture_course()
            },
        ));
    }
    assert_eq!(p.list_courses(true).unwrap(), [cse]);
}

#[test]
fn t24_empty_course_name() {
    let (mut p, _, cse) = fixture();
    for name in ["", " \t "] {
        assert_validation(p.create_course(CourseInput {
            name: name.into(),
            ..course_input("MTH110")
        }));
        assert_validation(p.update_course(
            cse.id,
            CourseInput {
                name: name.into(),
                ..fixture_course()
            },
        ));
    }
    assert_eq!(p.list_courses(true).unwrap(), [cse]);
}

#[test]
fn t24_empty_assessment_title() {
    let (mut p, _, cse) = fixture();
    for title in ["", "  "] {
        assert_validation(p.create_assessment(assessment(cse.id, title)));
    }
    assert!(p
        .list_assessments(AssessmentFilter::All)
        .unwrap()
        .is_empty());

    let a = p.create_assessment(assessment(cse.id, "Essay")).unwrap();
    assert_validation(p.update_assessment(a.id, assessment(cse.id, "")));
    assert_eq!(p.list_assessments(AssessmentFilter::All).unwrap(), [a]);
}

#[test]
fn t24_duplicate_course_code_in_term() {
    let (mut p, _, cse) = fixture();
    assert_validation(p.create_course(course_input("CSE101")));

    let other = p.create_course(course_input("MTH110")).unwrap();
    assert_validation(p.update_course(other.id, course_input("CSE101")));
    assert_eq!(p.list_courses(true).unwrap(), [cse, other]);
}

#[test]
fn t24_same_code_in_other_term_is_valid() {
    let (mut p, first_term, _) = fixture();
    let second_term = p.save_term(other_term()).unwrap();
    let course = p.create_course(fixture_course()).unwrap();
    assert_eq!(course.term_id, second_term.id);
    assert_ne!(course.term_id, first_term.id);
}

#[test]
fn t24_update_keeping_own_code_is_valid() {
    let (mut p, _, cse) = fixture();
    let updated = p
        .update_course(
            cse.id,
            CourseInput {
                name: "Renamed".into(),
                ..fixture_course()
            },
        )
        .unwrap();
    assert_eq!(updated.code, "CSE101");
    assert_eq!(updated.name, "Renamed");
}
