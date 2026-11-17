//! M26: an extra class added from the Dashboard (R139-R142, docs/spec.md §13.21, T130).
//!
//! The Dashboard handler calls `add_makeup` with the selected course and the form values; this
//! test drives the same call. Fixture: term 2026-01-05 (Mon) to 2026-02-01, course CSE101 with a
//! Monday slot A and a Wednesday slot B, synced with today = 2026-01-05.

mod common;

use common::*;
use planner_core::{SessionKind, Status};

/// R141, R142, T22 (T130): an extra class is a `makeup` session with no `replaces_session_id`;
/// it shows in that day's list, waits in Awaiting attendance once it has ended, counts when
/// marked attended, and leaves the course's other sessions unchanged.
#[test]
fn t130_extra_class_is_a_counted_makeup_without_replacement() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let before = sessions_of(&p, cse.id);

    // Wednesday 2026-01-14 already has slot B at 11:00; the extra class is at 15:00.
    let added = p
        .add_makeup(makeup_input(cse.id, "2026-01-14", "15:00"))
        .unwrap();

    assert_eq!(added.kind, SessionKind::Makeup);
    assert_eq!(added.replaces_session_id, None);
    assert_eq!(added.status, Status::Pending);

    // The course's other sessions are unchanged.
    let after = sessions_of(&p, cse.id);
    let others: Vec<_> = after.iter().filter(|s| s.id != added.id).cloned().collect();
    assert_eq!(others, before);

    // It appears in that day's list.
    let day = d("2026-01-14");
    let listed: Vec<i64> = p
        .sessions_between(day, day, dt("2026-01-14T08:00"))
        .unwrap()
        .into_iter()
        .map(|v| v.session.id)
        .collect();
    assert!(listed.contains(&added.id));

    // Awaiting attendance only after it ends (16:00).
    let running = p.dashboard(dt("2026-01-14T15:30")).unwrap();
    assert!(running.awaiting.iter().all(|v| v.session.id != added.id));
    let ended = p.dashboard(dt("2026-01-14T16:00")).unwrap();
    assert!(ended.awaiting.iter().any(|v| v.session.id == added.id));

    // Marking it attended counts it like any other session (T22).
    let pending_before = p.attendance_summary(cse.id).unwrap();
    mark(&mut p, added.id, Status::Attended, dt("2026-01-14T16:00"));
    let summary = p.attendance_summary(cse.id).unwrap();
    assert_eq!(summary.attended, pending_before.attended + 1);
    assert_eq!(summary.pending, pending_before.pending - 1);
}

/// R17, R142 (T130): an extra class that is the soonest pending session becomes the Next class.
#[test]
fn t130_extra_class_can_become_next_class() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let added = p
        .add_makeup(makeup_input(cse.id, "2026-01-05", "07:00"))
        .unwrap();

    let board = p.dashboard(dt("2026-01-05T06:00")).unwrap();

    assert_eq!(board.next_class.unwrap().session.id, added.id);
}
