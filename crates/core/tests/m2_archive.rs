//! Archived courses: hidden from the dashboard, history kept, sync leaves them alone.

mod common;

use std::collections::HashSet;

use common::*;
use planner_core::{AssessmentKind, SyncReport};

/// R3, R5: archiving hides the course from the dashboard, keeps its sessions and summary,
/// and sync neither creates nor deletes its sessions (T15).
#[test]
fn t15_archive_hides_course_but_keeps_history() {
    let (mut p, term, cse, _, _) = synced_fixture();
    p.create_assessment(assessment_input(
        cse.id,
        AssessmentKind::Quiz,
        "Overdue quiz",
        "2026-01-06T10:00",
    ))
    .unwrap();
    p.create_assessment(assessment_input(
        cse.id,
        AssessmentKind::Quiz,
        "Soon quiz",
        "2026-01-10T10:00",
    ))
    .unwrap();
    let before = sessions_of(&p, cse.id);

    p.set_archived(cse.id, true).unwrap();

    // Not in the dashboard: no sessions, assessments, awaiting, or below-target rows.
    let board = p.dashboard(dt("2026-01-08T12:00")).unwrap();
    assert!(board.today.is_empty());
    assert!(board.next_class.is_none());
    assert!(board.awaiting.is_empty());
    assert!(board.below_target.is_empty());
    assert!(board.upcoming.iter().all(|a| a.course_id != cse.id));
    assert!(board.overdue.iter().all(|a| a.course_id != cse.id));
    let board = p.dashboard(dt("2026-01-05T10:00")).unwrap();
    assert!(board.today.is_empty());
    assert!(board.awaiting.is_empty());

    // Sessions are kept and the summary is still computable.
    assert_eq!(sessions_of(&p, cse.id), before);
    let summary = p.attendance_summary(cse.id).unwrap();
    assert_eq!(summary.pending, 6);
    assert!(p.all_summaries().unwrap().is_empty());
    assert!(p
        .sessions_between(d("2026-01-05"), d("2026-01-25"), after_term())
        .unwrap()
        .is_empty());

    // Sync leaves the archived course's sessions alone, even when a holiday would remove one.
    p.set_holidays(term.id, &holidays(&["2026-01-19"])).unwrap();
    let report = p.sync_sessions(d("2026-01-14")).unwrap();
    assert_eq!(report, SyncReport::default());
    assert_eq!(sessions_of(&p, cse.id), before);
}

/// R3: sync creates no sessions for a course archived before its first sync (T15).
#[test]
fn t15_archived_course_gets_no_new_sessions() {
    let (mut p, _, cse) = fixture();
    p.add_slot(slot_a(cse.id)).unwrap();
    p.add_slot(slot_b(cse.id)).unwrap();
    p.set_archived(cse.id, true).unwrap();

    let report = p.sync_sessions(d("2026-01-05")).unwrap();
    assert_eq!(report, SyncReport::default());
    assert!(sessions_of(&p, cse.id).is_empty());
}

/// R3, R4: unarchive then sync leaves the same sessions, with no duplicates (T28).
#[test]
fn t28_unarchive_and_sync_creates_no_duplicates() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let before = sessions_of(&p, cse.id);

    p.set_archived(cse.id, true).unwrap();
    p.set_archived(cse.id, false).unwrap();
    let report = p.sync_sessions(d("2026-01-05")).unwrap();

    assert_eq!(report, SyncReport::default());
    let after = sessions_of(&p, cse.id);
    assert_eq!(after, before);
    let keys: HashSet<_> = after.iter().map(|s| (s.slot_id, s.date)).collect();
    assert_eq!(keys.len(), after.len());
}

/// R3, R4: generation resumes after unarchiving a course that was archived before any sync (T28).
#[test]
fn t28_generation_resumes_after_unarchive() {
    let (mut p, _, cse) = fixture();
    p.add_slot(slot_a(cse.id)).unwrap();
    p.add_slot(slot_b(cse.id)).unwrap();
    p.set_archived(cse.id, true).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();

    p.set_archived(cse.id, false).unwrap();
    let report = p.sync_sessions(d("2026-01-05")).unwrap();
    assert_eq!(report.created, 6);
    assert_eq!(report.deleted, 0);
    p.sync_sessions(d("2026-01-05")).unwrap();
    assert_eq!(sessions_of(&p, cse.id).len(), 6);
}
