//! Attendance percentage, below-target flag, marking rules, and course summaries.

mod common;

use common::*;
use planner_core::{CourseInput, Error, Status};

/// R13, R15: 3 attended, 1 absent, 2 cancelled, 5 pending gives 75.0 (T01).
#[test]
fn t01_percentage_from_mixed_statuses() {
    let (mut p, _, cse, _, _) = synced_fixture();
    // 6 regular sessions plus 5 pending makeups make 11 sessions.
    add_marked_makeups(&mut p, cse.id, 5, Status::Pending);
    let regular: Vec<_> = sessions_of(&p, cse.id)
        .into_iter()
        .filter(|s| s.slot_id.is_some())
        .collect();
    assert_eq!(regular.len(), 6);
    let statuses = [
        Status::Attended,
        Status::Attended,
        Status::Attended,
        Status::Absent,
        Status::Cancelled,
        Status::Cancelled,
    ];
    for (s, status) in regular.iter().zip(statuses) {
        mark(&mut p, s.id, status, after_term());
    }

    let summary = p.attendance_summary(cse.id).unwrap();
    assert_eq!(summary.attended, 3);
    assert_eq!(summary.absent, 1);
    assert_eq!(summary.cancelled, 2);
    assert_eq!(summary.pending, 5);
    assert_pct(summary.pct, Some(75.0));
}

/// R13: with only cancelled and pending sessions the percentage is None (T02).
#[test]
fn t02_only_cancelled_and_pending_has_no_percentage() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let first = regular_on(&p, cse.id, "2026-01-05");
    mark(&mut p, first.id, Status::Cancelled, dt("2026-01-05T08:00"));

    let summary = p.attendance_summary(cse.id).unwrap();
    assert_eq!(summary.cancelled, 1);
    assert_eq!(summary.pending, 5);
    assert_eq!(summary.attended + summary.absent, 0);
    assert_pct(summary.pct, None);
    assert!(!summary.below_target);
}

/// R14: below target is true at 74.9, false at 75.0, false without a target (T03).
/// The ratios come from real sessions: 149/199 rounds to 74.9, 3/4 is exactly 75.0.
#[test]
fn t03_below_target_boundaries() {
    // 74.9 against target 75.
    let (mut p, _, cse) = fixture();
    add_marked_makeups(&mut p, cse.id, 149, Status::Attended);
    add_marked_makeups(&mut p, cse.id, 50, Status::Absent);
    let summary = p.attendance_summary(cse.id).unwrap();
    assert_eq!(summary.target, Some(75));
    assert_pct(summary.pct, Some(74.9));
    assert!(summary.below_target);

    // 75.0 against target 75.
    let (mut p, _, cse) = fixture();
    add_marked_makeups(&mut p, cse.id, 3, Status::Attended);
    add_marked_makeups(&mut p, cse.id, 1, Status::Absent);
    let summary = p.attendance_summary(cse.id).unwrap();
    assert_pct(summary.pct, Some(75.0));
    assert!(!summary.below_target);

    // No target: never below target.
    let (mut p, _, _) = fixture();
    let open = p
        .create_course(CourseInput {
            min_attendance_pct: None,
            ..course_input("MTH110")
        })
        .unwrap();
    add_marked_makeups(&mut p, open.id, 1, Status::Attended);
    add_marked_makeups(&mut p, open.id, 3, Status::Absent);
    let summary = p.attendance_summary(open.id).unwrap();
    assert_eq!(summary.target, None);
    assert_pct(summary.pct, Some(25.0));
    assert!(!summary.below_target);
}

/// R12: attended on a future session is a validation error; cancelling it succeeds (T12).
#[test]
fn t12_attended_on_future_session_rejected() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let future = regular_on(&p, cse.id, "2026-01-12");
    let now = dt("2026-01-11T12:00");

    let result = p.set_status(future.id, Status::Attended, now);
    assert!(matches!(result, Err(Error::Validation(_))), "{result:?}");
    let result = p.set_status(future.id, Status::Absent, now);
    assert!(matches!(result, Err(Error::Validation(_))), "{result:?}");
    assert_eq!(regular_on(&p, cse.id, "2026-01-12").status, Status::Pending);

    p.set_status(future.id, Status::Cancelled, now).unwrap();
    assert_eq!(
        regular_on(&p, cse.id, "2026-01-12").status,
        Status::Cancelled
    );
}

/// R12: `now >= start` is inclusive, so marking at exactly the start moment is allowed (T12).
#[test]
fn t12_attended_allowed_at_exact_start() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let s = regular_on(&p, cse.id, "2026-01-05");

    let result = p.set_status(s.id, Status::Attended, dt("2026-01-05T08:59"));
    assert!(matches!(result, Err(Error::Validation(_))), "{result:?}");

    p.set_status(s.id, Status::Attended, dt("2026-01-05T09:00"))
        .unwrap();
    assert_eq!(
        regular_on(&p, cse.id, "2026-01-05").status,
        Status::Attended
    );
}

/// R13, R15: a course with no slots has an all-zero summary and no percentage (T21).
#[test]
fn t21_course_without_slots_has_empty_summary() {
    let (mut p, _, cse) = fixture();
    p.sync_sessions(d("2026-01-05")).unwrap();

    let summary = p.attendance_summary(cse.id).unwrap();
    assert_eq!(summary.course_id, cse.id);
    assert_eq!(summary.course_code, "CSE101");
    assert_eq!(
        (
            summary.attended,
            summary.absent,
            summary.cancelled,
            summary.pending
        ),
        (0, 0, 0, 0)
    );
    assert_pct(summary.pct, None);
    assert!(!summary.below_target);
}

/// R12: attended can go back to pending, and the counts follow (T23).
#[test]
fn t23_attended_back_to_pending_updates_counts() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let s = regular_on(&p, cse.id, "2026-01-05");
    let now = dt("2026-01-05T10:00");

    mark(&mut p, s.id, Status::Attended, now);
    let summary = p.attendance_summary(cse.id).unwrap();
    assert_eq!((summary.attended, summary.pending), (1, 5));
    assert_pct(summary.pct, Some(100.0));

    mark(&mut p, s.id, Status::Pending, now);
    let summary = p.attendance_summary(cse.id).unwrap();
    assert_eq!((summary.attended, summary.pending), (0, 6));
    assert_pct(summary.pct, None);
    assert_eq!(regular_on(&p, cse.id, "2026-01-05").status, Status::Pending);
}
