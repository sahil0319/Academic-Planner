//! M9 migration 002 and the stored credits, rule set, component link, and score fields.

mod common;

use common::*;
use planner_core::{AssessmentFilter, AssessmentKind, Error, Planner};
use rusqlite::Connection;
use tempfile::TempDir;

fn count(conn: &Connection, table: &str) -> i64 {
    conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

fn planner_with_term() -> Planner {
    let mut p = Planner::open_in_memory().unwrap();
    p.save_term(fixture_term()).unwrap();
    p
}

fn assessment_input(course_id: i64) -> planner_core::AssessmentInput {
    planner_core::AssessmentInput {
        course_id,
        kind: AssessmentKind::Quiz,
        title: "Quiz 1".into(),
        due_at: dt("2026-01-12T10:00"),
        notes: None,
        remind_minutes_before: None,
        component_id: None,
        score: None,
    }
}

/// R25, R26, R33, R36, §13.3: opening a version 1 file database upgrades it to version 2,
/// keeps the data, adds the new columns as `None`, writes the pre-migration copy, and seeds
/// the grading tables.
#[test]
fn m9_migration_002_upgrades_v1_file_db() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("planner.sqlite");
    {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        conn.execute_batch(include_str!("../src/migrations/001_init.sql"))
            .unwrap();
        conn.pragma_update(None, "user_version", 1).unwrap();
        conn.execute(
            "INSERT INTO term (id, name, start_date, end_date, is_active) \
             VALUES (1, 'Winter 2026', '2026-01-05', '2026-01-25', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO course (id, term_id, code, name) VALUES (1, 1, 'CSE101', 'Intro')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO assessment (id, course_id, kind, title, due_at) \
             VALUES (1, 1, 'quiz', 'Quiz 1', '2026-01-12T10:00')",
            [],
        )
        .unwrap();
    }

    let p = Planner::open(&path).unwrap();

    let conn = Connection::open(&path).unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, 3);
    assert!(dir
        .path()
        .join("backups/pre-migrate-v1-to-v3.sqlite")
        .exists());
    assert_eq!(count(&conn, "rule_set"), 2);
    assert_eq!(count(&conn, "rule_component"), 7);
    assert_eq!(count(&conn, "attendance_band"), 12);
    assert_eq!(count(&conn, "grade_scale"), 10);

    let courses = p.list_courses(true).unwrap();
    assert_eq!(courses.len(), 1);
    assert_eq!(courses[0].code, "CSE101");
    assert_eq!(courses[0].credits, None);
    assert_eq!(courses[0].rule_set_id, None);
    let assessments = p.list_assessments(AssessmentFilter::All).unwrap();
    assert_eq!(assessments.len(), 1);
    assert_eq!(assessments[0].title, "Quiz 1");
    assert_eq!(assessments[0].component_id, None);
    assert_eq!(assessments[0].score, None);
}

/// R33: credits and rule set are stored on create, read back, and cleared on update.
#[test]
fn r33_course_credits_and_rule_set_stored() {
    let mut p = planner_with_term();
    let created = p
        .create_course(planner_core::CourseInput {
            credits: Some(1.5),
            rule_set_id: Some(1),
            ..course_input("CSE101")
        })
        .unwrap();
    assert_eq!(created.credits, Some(1.5));
    assert_eq!(created.rule_set_id, Some(1));
    let listed = p.list_courses(false).unwrap();
    assert_eq!(listed[0].credits, Some(1.5));
    assert_eq!(listed[0].rule_set_id, Some(1));

    let updated = p
        .update_course(
            created.id,
            planner_core::CourseInput {
                credits: None,
                rule_set_id: None,
                ..course_input("CSE101")
            },
        )
        .unwrap();
    assert_eq!(updated.credits, None);
    assert_eq!(updated.rule_set_id, None);
    let listed = p.list_courses(false).unwrap();
    assert_eq!(listed[0].credits, None);
    assert_eq!(listed[0].rule_set_id, None);
}

/// R33: credits must be greater than 0.
#[test]
fn r33_credits_must_be_positive() {
    let mut p = planner_with_term();
    for bad in [0.0, -1.0] {
        let result = p.create_course(planner_core::CourseInput {
            credits: Some(bad),
            ..course_input("CSE101")
        });
        assert!(
            matches!(result, Err(Error::Validation(_))),
            "credits {bad}: {result:?}"
        );
    }
    assert!(p.list_courses(true).unwrap().is_empty());
}

/// R33: a rule set id that names no rule set is rejected.
#[test]
fn r33_unknown_rule_set_rejected() {
    let mut p = planner_with_term();
    let result = p.create_course(planner_core::CourseInput {
        credits: Some(3.0),
        rule_set_id: Some(99),
        ..course_input("CSE101")
    });
    assert!(matches!(result, Err(Error::Validation(_))), "{result:?}");
    assert!(p.list_courses(true).unwrap().is_empty());
}

/// R36: a score is stored and read back; a negative score is rejected.
#[test]
fn r36_score_stored_and_negative_rejected() {
    let mut p = planner_with_term();
    let course = p.create_course(course_input("CSE101")).unwrap();

    let created = p
        .create_assessment(planner_core::AssessmentInput {
            score: Some(12.0),
            ..assessment_input(course.id)
        })
        .unwrap();
    assert_eq!(created.score, Some(12.0));
    assert_eq!(created.component_id, None);
    let listed = p.list_assessments(AssessmentFilter::All).unwrap();
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].score, Some(12.0));

    let result = p.create_assessment(planner_core::AssessmentInput {
        score: Some(-1.0),
        ..assessment_input(course.id)
    });
    assert!(matches!(result, Err(Error::Validation(_))), "{result:?}");
    let result = p.update_assessment(
        created.id,
        planner_core::AssessmentInput {
            score: Some(-1.0),
            ..assessment_input(course.id)
        },
    );
    assert!(matches!(result, Err(Error::Validation(_))), "{result:?}");
}
