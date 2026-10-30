//! M14 archive a finished term and list past terms (R68, R70; T67, T68, T69).

mod common;

use common::*;
use planner_core::{
    AssessmentInput, AssessmentKind, Error, Planner, ReminderKind, SyncReport, Term, TermInput,
};

const SPRING_END_MESSAGE: &str = "This term ends on May 1, 2026; archive it after that";

fn term_input(name: &str, start: &str, end: &str) -> TermInput {
    TermInput {
        id: None,
        name: name.into(),
        start_date: d(start),
        end_date: d(end),
    }
}

/// Active term "Spring 2026" (2026-01-05 to 2026-05-01) with course CSE101, slots A and B
/// synced from the first day. Returns (planner, term, course id).
fn spring() -> (Planner, Term, i64) {
    let mut p = Planner::open_in_memory().unwrap();
    let term = p
        .save_term(term_input("Spring 2026", "2026-01-05", "2026-05-01"))
        .unwrap();
    let course = p.create_course(fixture_course()).unwrap();
    p.add_slot(slot_a(course.id)).unwrap();
    p.add_slot(slot_b(course.id)).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    (p, term, course.id)
}

/// R68 (T67): archiving on or before the end date is rejected with the exact message, and the
/// term stays active.
#[test]
fn t67_archive_before_or_on_end_is_rejected() {
    let (mut p, term, course_id) = spring();
    let sessions = sessions_of(&p, course_id);
    assert!(!sessions.is_empty());

    for today in ["2026-04-30", "2026-05-01"] {
        match p.archive_term(term.id, d(today)) {
            Err(Error::Validation(msg)) => assert_eq!(msg, SPRING_END_MESSAGE, "today {today}"),
            other => panic!("today {today}: expected Validation, got {other:?}"),
        }
        assert_eq!(p.active_term().unwrap(), Some(term.clone()));
    }
    assert_eq!(sessions_of(&p, course_id), sessions);
    assert!(p.past_terms().unwrap().is_empty());
}

/// R68 (T67): archiving the day after the end date succeeds, there is no active term, and
/// the term's courses and sessions are unchanged.
#[test]
fn t67_archive_after_end_keeps_data() {
    let (mut p, term, course_id) = spring();
    let sessions = sessions_of(&p, course_id);
    let summary = p.attendance_summary(course_id).unwrap();
    assert!(!sessions.is_empty());

    p.archive_term(term.id, d("2026-05-02")).unwrap();

    assert_eq!(p.active_term().unwrap(), None);
    assert_eq!(sessions_of(&p, course_id), sessions);
    assert_eq!(p.attendance_summary(course_id).unwrap(), summary);
    let past = p.past_terms().unwrap();
    assert_eq!(past.len(), 1);
    assert_eq!(past[0].term.id, term.id);
    assert!(!past[0].term.is_active);
    assert_eq!(past[0].courses, 1);
}

/// R68 (T67): an id that does not exist is a `NotFound` error for entity "term".
#[test]
fn t67_archive_unknown_term_is_not_found() {
    let (mut p, term, _) = spring();

    match p.archive_term(term.id + 100, d("2026-05-02")) {
        Err(Error::NotFound { entity, id }) => {
            assert_eq!(entity, "term");
            assert_eq!(id, term.id + 100);
        }
        other => panic!("expected NotFound, got {other:?}"),
    }
    assert_eq!(p.active_term().unwrap(), Some(term));
}

/// R68 (T67): archiving a term that is already inactive is an Ok no-op, even before its end
/// date (D3), and it does not touch the active term.
#[test]
fn t67_archive_inactive_term_is_noop() {
    let (mut p, spring, _) = spring();
    let next = p
        .save_term(term_input("Fall 2026", "2026-09-01", "2026-12-18"))
        .unwrap();
    let before = p.past_terms().unwrap();
    assert_eq!(before.len(), 1);
    assert_eq!(before[0].term.id, spring.id);

    p.archive_term(spring.id, d("2026-03-01")).unwrap();
    p.archive_term(spring.id, d("2026-05-02")).unwrap();

    assert_eq!(p.past_terms().unwrap(), before);
    assert_eq!(p.active_term().unwrap().map(|t| t.id), Some(next.id));
}

/// R68 (T68): once archived, a term is ignored by sync (R5 deletions included), the
/// dashboard, and reminders, even for a moment inside its old date range.
#[test]
fn t68_archived_term_is_ignored_by_sync_dashboard_and_reminders() {
    // Monday 2026-02-02: class A at 09:00 (reminder lead 15 minutes, fires 08:45) and a quiz
    // due 09:30 with a 45 minute lead (fires 08:45).
    let now = dt("2026-02-02T08:50");
    let setup = || {
        let (mut p, term, course_id) = spring();
        p.create_assessment(AssessmentInput {
            remind_minutes_before: Some(45),
            ..assessment_input(
                course_id,
                AssessmentKind::Quiz,
                "Quiz 1",
                "2026-02-02T09:30",
            )
        })
        .unwrap();
        // A holiday added after sync: an active term's next sync would delete this session.
        p.set_holidays(term.id, &holidays(&["2026-02-09"])).unwrap();
        (p, term, course_id)
    };

    // Control: while the term is active the same moment has both reminders and a board.
    let (mut control, _, _) = setup();
    let due = control.due_reminders(now).unwrap();
    assert_eq!(due.len(), 2);
    assert_eq!(due[0].kind, ReminderKind::Session);
    assert_eq!(due[1].kind, ReminderKind::Assessment);
    assert!(!control.dashboard(now).unwrap().today.is_empty());
    assert!(control.sync_sessions(d("2026-02-02")).unwrap().deleted > 0);

    let (mut p, term, course_id) = setup();
    let sessions = sessions_of(&p, course_id);
    p.archive_term(term.id, d("2026-05-02")).unwrap();

    assert_eq!(
        p.sync_sessions(d("2026-02-02")).unwrap(),
        SyncReport::default()
    );
    assert_eq!(sessions_of(&p, course_id), sessions);

    let board = p.dashboard(now).unwrap();
    assert!(board.today.is_empty());
    assert!(board.next_class.is_none());
    assert!(board.upcoming.is_empty());
    assert!(board.overdue.is_empty());
    assert!(board.awaiting.is_empty());
    assert!(board.below_target.is_empty());
    assert!(p.all_summaries().unwrap().is_empty());

    assert!(p.due_reminders(now).unwrap().is_empty());
}

/// R70 (T69): past terms come newest `end_date` first with the course count (archived
/// courses included), the overall attendance (counts summed over every course, R13
/// rounding), and the R40 GPA of that term. The active term is never listed.
#[test]
fn t69_past_terms_newest_first_with_counts_attendance_and_gpa() {
    let mut p = Planner::open_in_memory().unwrap();

    // Past term 1 (ends first), made inactive by the next save_term, never archived.
    // Graded A+ course, an ungraded course, and an archived graded course.
    let winter = p.save_term(fixture_term()).unwrap();
    let a_plus = graded_course(&mut p, "CSE101", 3.0, "Theory");
    let mid = component_id(&p, "Theory", "Mid");
    scored(&mut p, a_plus.id, mid, "Mid", 60.0);
    let ungraded = p.create_course(course_input("CSE102")).unwrap();
    mark_attendance(&mut p, ungraded.id, 3, 1);
    let archived = graded_course(&mut p, "CSE103", 3.0, "Theory");
    scored(&mut p, archived.id, mid, "Mid", 30.0);
    mark_attendance(&mut p, archived.id, 2, 1);
    p.set_archived(archived.id, true).unwrap();

    // Past term 2, archived after its end: a B course (62.0%), a graded course with nothing
    // assessed (left out of the GPA), and an ungraded course with 7 of 8 attended.
    let spring = p
        .save_term(term_input("Spring 2026", "2026-01-05", "2026-05-01"))
        .unwrap();
    let b = graded_course(&mut p, "CSE201", 3.0, "Theory");
    scored(&mut p, b.id, mid, "Mid", 46.5);
    graded_course(&mut p, "CSE202", 3.0, "Theory");
    let seven_of_eight = p.create_course(course_input("CSE203")).unwrap();
    mark_attendance(&mut p, seven_of_eight.id, 7, 1);
    p.archive_term(spring.id, d("2026-05-02")).unwrap();

    // Past term 3 (ends last): one course, nothing marked, made inactive by the next save_term.
    let summer = p
        .save_term(term_input("Summer 2026", "2026-06-01", "2026-07-31"))
        .unwrap();
    p.create_course(course_input("CSE301")).unwrap();

    // The active term has marks too; it must not show up.
    let fall = p
        .save_term(term_input("Fall 2026", "2026-09-01", "2026-12-18"))
        .unwrap();
    let current = p.create_course(course_input("CSE401")).unwrap();
    mark_attendance(&mut p, current.id, 1, 0);

    let past = p.past_terms().unwrap();
    let ids: Vec<i64> = past.iter().map(|s| s.term.id).collect();
    assert_eq!(ids, [summer.id, spring.id, winter.id]);
    assert!(past.iter().all(|s| !s.term.is_active));
    assert_eq!(p.active_term().unwrap().map(|t| t.id), Some(fall.id));
    let names: Vec<&str> = past.iter().map(|s| s.term.name.as_str()).collect();
    assert_eq!(names, ["Summer 2026", "Spring 2026", "Winter 2026"]);

    let courses: Vec<u32> = past.iter().map(|s| s.courses).collect();
    assert_eq!(courses, [1, 3, 3]);

    // Nothing marked: no attendance and no GPA.
    assert_pct(past[0].attendance_pct, None);
    assert_pct(past[0].gpa, None);
    // 7 of 8 attended = 87.5; the B course gives 3.00 (the unassessed course is left out).
    assert_pct(past[1].attendance_pct, Some(87.5));
    assert_pct(past[1].gpa, Some(3.0));
    // (3 + 2) attended of (3 + 1 + 2 + 1): 5 / 7 = 71.4, summed over courses (an average of
    // the per-course percentages would be 70.8). GPA is the A+ course alone (4.00): the
    // archived graded course and the ungraded course are excluded.
    assert_pct(past[2].attendance_pct, Some(71.4));
    assert_pct(past[2].gpa, Some(4.0));
}
