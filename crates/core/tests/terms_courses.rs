//! R1 (one active term), holidays, course CRUD, and R3 (archived courses hidden from
//! default lists).

mod common;

use common::*;
use planner_core::{DayOffKind, Error, Holiday, Planner, TermInput};

#[test]
fn r01_no_term_means_no_active_term() {
    let p = Planner::open_in_memory().unwrap();
    assert_eq!(p.active_term().unwrap(), None);
}

#[test]
fn r01_saved_term_becomes_active() {
    let mut p = Planner::open_in_memory().unwrap();
    let term = p.save_term(fixture_term()).unwrap();
    assert!(term.is_active);
    assert_eq!(term.name, "Winter 2026");
    assert_eq!(term.start_date, d("2026-01-05"));
    assert_eq!(term.end_date, d("2026-01-25"));
    assert!(term.holidays.is_empty());
    assert_eq!(p.active_term().unwrap(), Some(term));
}

#[test]
fn r01_second_term_becomes_only_active() {
    let mut p = Planner::open_in_memory().unwrap();
    let first = p.save_term(fixture_term()).unwrap();
    let second = p
        .save_term(TermInput {
            id: None,
            name: "Summer 2026".into(),
            start_date: d("2026-05-04"),
            end_date: d("2026-07-31"),
        })
        .unwrap();
    assert_ne!(first.id, second.id);
    assert_eq!(p.active_term().unwrap().map(|t| t.id), Some(second.id));

    // Saving the first term again makes it the only active one.
    let first_again = p
        .save_term(TermInput {
            id: Some(first.id),
            ..fixture_term()
        })
        .unwrap();
    assert!(first_again.is_active);
    assert_eq!(p.active_term().unwrap().map(|t| t.id), Some(first.id));
}

#[test]
fn r01_update_term_keeps_id_and_active() {
    let mut p = Planner::open_in_memory().unwrap();
    let term = p.save_term(fixture_term()).unwrap();
    let updated = p
        .save_term(TermInput {
            id: Some(term.id),
            name: "Winter 2026 (revised)".into(),
            end_date: d("2026-01-31"),
            ..fixture_term()
        })
        .unwrap();
    assert_eq!(updated.id, term.id);
    assert!(updated.is_active);
    assert_eq!(updated.name, "Winter 2026 (revised)");
    assert_eq!(updated.end_date, d("2026-01-31"));
    assert_eq!(p.active_term().unwrap(), Some(updated));
}

#[test]
fn update_missing_term_is_not_found() {
    let mut p = Planner::open_in_memory().unwrap();
    let result = p.save_term(TermInput {
        id: Some(42),
        ..fixture_term()
    });
    assert!(matches!(result, Err(Error::NotFound { id: 42, .. })));
    assert_eq!(p.active_term().unwrap(), None);
}

#[test]
fn set_holidays_replaces_set() {
    let mut p = Planner::open_in_memory().unwrap();
    let term = p.save_term(fixture_term()).unwrap();
    let h = |date: &str, label: Option<&str>| Holiday {
        date: d(date),
        label: label.map(Into::into),
        kind: DayOffKind::Holiday,
    };

    p.set_holidays(
        term.id,
        &[h("2026-01-19", Some("MLK Day")), h("2026-01-12", None)],
    )
    .unwrap();
    let holidays = p.active_term().unwrap().unwrap().holidays;
    assert_eq!(
        holidays,
        [h("2026-01-12", None), h("2026-01-19", Some("MLK Day"))]
    );

    p.set_holidays(term.id, &[h("2026-01-14", Some("Snow day"))])
        .unwrap();
    let holidays = p.active_term().unwrap().unwrap().holidays;
    assert_eq!(holidays, [h("2026-01-14", Some("Snow day"))]);

    p.set_holidays(term.id, &[]).unwrap();
    assert!(p.active_term().unwrap().unwrap().holidays.is_empty());

    assert!(matches!(
        p.set_holidays(term.id + 1, &[h("2026-01-12", None)]),
        Err(Error::NotFound { .. })
    ));
}

#[test]
fn courses_create_update_list_in_active_term() {
    let (mut p, term, cse) = fixture();
    assert_eq!(cse.term_id, term.id);
    assert_eq!(cse.code, "CSE101");
    assert_eq!(cse.name, "Intro to Programming");
    assert_eq!(cse.room.as_deref(), Some("A-101"));
    assert_eq!(cse.min_attendance_pct, Some(75));
    assert!(!cse.archived);

    let abc = p.create_course(course_input("ABC200")).unwrap();
    let listed: Vec<_> = p.list_courses(false).unwrap();
    assert_eq!(listed, [abc.clone(), cse.clone()], "ordered by code");

    let updated = p
        .update_course(
            cse.id,
            planner_core::CourseInput {
                name: "Programming I".into(),
                instructor: Some("Dr. Ada".into()),
                min_attendance_pct: None,
                credits: None,
                rule_set_id: None,
                ..fixture_course()
            },
        )
        .unwrap();
    assert_eq!(updated.id, cse.id);
    assert_eq!(updated.name, "Programming I");
    assert_eq!(updated.instructor.as_deref(), Some("Dr. Ada"));
    assert_eq!(updated.min_attendance_pct, None);
    assert_eq!(p.list_courses(false).unwrap(), [abc, updated]);

    // Courses of a term that is no longer active are not listed.
    p.save_term(TermInput {
        id: None,
        name: "Summer 2026".into(),
        start_date: d("2026-05-04"),
        end_date: d("2026-07-31"),
    })
    .unwrap();
    assert!(p.list_courses(true).unwrap().is_empty());
}

#[test]
fn create_course_without_active_term_fails() {
    let mut p = Planner::open_in_memory().unwrap();
    assert!(matches!(
        p.create_course(fixture_course()),
        Err(Error::NoActiveTerm)
    ));
    assert!(p.list_courses(true).unwrap().is_empty());
}

#[test]
fn update_missing_course_is_not_found() {
    let (mut p, _, cse) = fixture();
    assert!(matches!(
        p.update_course(cse.id + 100, fixture_course()),
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        p.set_archived(cse.id + 100, true),
        Err(Error::NotFound { .. })
    ));
}

#[test]
fn r03_archived_hidden_from_default_list() {
    let (mut p, _, cse) = fixture();
    let other = p.create_course(course_input("MTH110")).unwrap();

    p.set_archived(cse.id, true).unwrap();
    assert_eq!(p.list_courses(false).unwrap(), std::slice::from_ref(&other));
    let all = p.list_courses(true).unwrap();
    let archived = all.iter().find(|c| c.id == cse.id).unwrap();
    assert!(archived.archived);
    assert_eq!(
        planner_core::Course {
            archived: false,
            ..archived.clone()
        },
        cse,
        "archiving changes only the flag"
    );

    p.set_archived(cse.id, false).unwrap();
    assert_eq!(p.list_courses(false).unwrap(), [cse, other]);
}
