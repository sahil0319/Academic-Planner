//! Assessment storage: create, update, complete, delete, list (All / Completed).

mod common;

use common::*;
use planner_core::{Assessment, AssessmentFilter, AssessmentInput, AssessmentKind, Error, Planner};

fn quiz(course_id: i64, title: &str, due: &str) -> AssessmentInput {
    AssessmentInput {
        course_id,
        kind: AssessmentKind::Quiz,
        title: title.into(),
        due_at: dt(due),
        notes: None,
        remind_minutes_before: None,
    }
}

fn titles(list: &[Assessment]) -> Vec<&str> {
    list.iter().map(|a| a.title.as_str()).collect()
}

fn all(p: &Planner) -> Vec<Assessment> {
    p.list_assessments(AssessmentFilter::All).unwrap()
}

#[test]
fn assessment_create_update_round_trip() {
    let (mut p, _, cse) = fixture();
    let created = p
        .create_assessment(AssessmentInput {
            notes: Some("Chapters 1-3".into()),
            remind_minutes_before: Some(60),
            ..quiz(cse.id, "Quiz 1", "2026-01-09T10:00")
        })
        .unwrap();
    assert_eq!(created.course_id, cse.id);
    assert_eq!(created.course_code, "CSE101");
    assert_eq!(created.kind, AssessmentKind::Quiz);
    assert_eq!(created.title, "Quiz 1");
    assert_eq!(created.due_at, dt("2026-01-09T10:00"));
    assert_eq!(created.notes.as_deref(), Some("Chapters 1-3"));
    assert_eq!(created.remind_minutes_before, Some(60));
    assert!(!created.completed);

    let updated = p
        .update_assessment(
            created.id,
            AssessmentInput {
                kind: AssessmentKind::Exam,
                title: "Midterm".into(),
                ..quiz(cse.id, "", "2026-01-16T14:00")
            },
        )
        .unwrap();
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.kind, AssessmentKind::Exam);
    assert_eq!(updated.title, "Midterm");
    assert_eq!(updated.due_at, dt("2026-01-16T14:00"));
    assert_eq!(updated.notes, None);
    assert_eq!(updated.remind_minutes_before, None);
    assert_eq!(all(&p), [updated]);
}

#[test]
fn set_completed_toggles() {
    let (mut p, _, cse) = fixture();
    let a = p
        .create_assessment(quiz(cse.id, "Quiz 1", "2026-01-09T10:00"))
        .unwrap();
    p.set_completed(a.id, true).unwrap();
    assert!(all(&p)[0].completed);
    p.set_completed(a.id, false).unwrap();
    assert!(!all(&p)[0].completed);
}

#[test]
fn delete_assessment_removes_row() {
    let (mut p, _, cse) = fixture();
    let a = p
        .create_assessment(quiz(cse.id, "Quiz 1", "2026-01-09T10:00"))
        .unwrap();
    let b = p
        .create_assessment(quiz(cse.id, "Quiz 2", "2026-01-16T10:00"))
        .unwrap();
    p.delete_assessment(a.id).unwrap();
    assert_eq!(all(&p), [b]);
}

#[test]
fn list_all_sorted_by_due_at() {
    let (mut p, _, cse) = fixture();
    let other = p.create_course(course_input("ABC200")).unwrap();
    p.create_assessment(quiz(cse.id, "Late", "2026-01-20T09:00"))
        .unwrap();
    p.create_assessment(quiz(other.id, "Early", "2026-01-06T23:59"))
        .unwrap();
    p.create_assessment(quiz(cse.id, "Middle", "2026-01-13T08:00"))
        .unwrap();
    assert_eq!(titles(&all(&p)), ["Early", "Middle", "Late"]);
}

#[test]
fn list_completed_only_completed() {
    let (mut p, _, cse) = fixture();
    let done = p
        .create_assessment(quiz(cse.id, "Done", "2026-01-09T10:00"))
        .unwrap();
    p.create_assessment(quiz(cse.id, "Open", "2026-01-08T10:00"))
        .unwrap();
    p.set_completed(done.id, true).unwrap();
    let completed = p.list_assessments(AssessmentFilter::Completed).unwrap();
    assert_eq!(titles(&completed), ["Done"]);
}

#[test]
fn list_excludes_archived_course() {
    let (mut p, _, cse) = fixture();
    let other = p.create_course(course_input("ABC200")).unwrap();
    p.create_assessment(quiz(cse.id, "Hidden", "2026-01-09T10:00"))
        .unwrap();
    p.create_assessment(quiz(other.id, "Shown", "2026-01-10T10:00"))
        .unwrap();
    p.set_archived(cse.id, true).unwrap();
    assert_eq!(titles(&all(&p)), ["Shown"]);
    p.set_archived(cse.id, false).unwrap();
    assert_eq!(titles(&all(&p)), ["Hidden", "Shown"]);
}

#[test]
fn missing_assessment_is_not_found() {
    let (mut p, _, cse) = fixture();
    let missing = 999;
    assert!(matches!(
        p.update_assessment(missing, quiz(cse.id, "X", "2026-01-09T10:00")),
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        p.set_completed(missing, true),
        Err(Error::NotFound { .. })
    ));
    assert!(matches!(
        p.delete_assessment(missing),
        Err(Error::NotFound { .. })
    ));
}
