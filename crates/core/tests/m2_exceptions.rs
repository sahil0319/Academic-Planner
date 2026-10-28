//! Exceptions: cancel, reschedule, standalone makeup.

mod common;

use common::*;
use planner_core::{Error, SessionKind, Status, SyncReport};

/// R8: a cancelled session stays cancelled after re-sync and its slot is untouched (T08).
#[test]
fn t08_cancelled_session_survives_resync() {
    let (mut p, _, cse, a, _) = synced_fixture();
    let slots_before = p.list_slots(d("2026-01-12")).unwrap();
    let original = regular_on(&p, cse.id, "2026-01-12");
    mark(
        &mut p,
        original.id,
        Status::Cancelled,
        dt("2026-01-05T08:00"),
    );

    let report = p.sync_sessions(d("2026-01-05")).unwrap();
    assert_eq!(report, SyncReport::default());

    let after = regular_on(&p, cse.id, "2026-01-12");
    assert_eq!(after.id, original.id);
    assert_eq!(after.status, Status::Cancelled);
    assert_eq!(after.slot_id, Some(a.id));
    assert_eq!(sessions_of(&p, cse.id).len(), 6);
    assert_eq!(p.list_slots(d("2026-01-12")).unwrap(), slots_before);
}

/// R9: rescheduling cancels the original, creates a makeup that replaces it, and the
/// original displays "Rescheduled → date" (T09).
#[test]
fn t09_reschedule_creates_makeup_and_cancels_original() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let original = regular_on(&p, cse.id, "2026-01-12");

    let makeup = p
        .reschedule(original.id, makeup_input(cse.id, "2026-01-13", "14:00"))
        .unwrap();
    assert_eq!(makeup.kind, SessionKind::Makeup);
    assert_eq!(makeup.slot_id, None);
    assert_eq!(makeup.replaces_session_id, Some(original.id));
    assert_eq!(makeup.date, d("2026-01-13"));
    assert_eq!(makeup.start_time, t("14:00"));
    assert_eq!(makeup.end_time, t("15:00"));

    mark(&mut p, makeup.id, Status::Attended, dt("2026-01-13T15:00"));

    let summary = p.attendance_summary(cse.id).unwrap();
    assert_eq!(summary.attended, 1);
    assert_eq!(summary.cancelled, 1);

    let views = p.sessions_for_course(cse.id, after_term()).unwrap();
    let orig_view = views.iter().find(|v| v.session.id == original.id).unwrap();
    assert_eq!(orig_view.session.status, Status::Cancelled);
    assert_eq!(orig_view.display_status, "Rescheduled → 2026-01-13");
    let makeup_view = views.iter().find(|v| v.session.id == makeup.id).unwrap();
    assert_eq!(makeup_view.display_status, "Attended");
}

/// R9: an attended session cannot be rescheduled; nothing changes (T19).
#[test]
fn t19_reschedule_attended_session_is_error() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let original = regular_on(&p, cse.id, "2026-01-05");
    mark(
        &mut p,
        original.id,
        Status::Attended,
        dt("2026-01-05T10:00"),
    );
    let before = sessions_of(&p, cse.id);

    let result = p.reschedule(original.id, makeup_input(cse.id, "2026-01-06", "09:00"));
    assert!(matches!(result, Err(Error::Validation(_))), "{result:?}");

    assert_eq!(sessions_of(&p, cse.id), before);
    assert_eq!(
        regular_on(&p, cse.id, "2026-01-05").status,
        Status::Attended
    );
}

/// R9: an absent session cannot be rescheduled either; nothing changes (T19).
#[test]
fn t19_reschedule_absent_session_is_error() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let original = regular_on(&p, cse.id, "2026-01-05");
    mark(&mut p, original.id, Status::Absent, dt("2026-01-05T10:00"));
    let before = sessions_of(&p, cse.id);

    let result = p.reschedule(original.id, makeup_input(cse.id, "2026-01-06", "09:00"));
    assert!(matches!(result, Err(Error::Validation(_))), "{result:?}");

    assert_eq!(sessions_of(&p, cse.id), before);
}

/// R10: a standalone makeup replaces nothing and counts as attended once marked (T22).
#[test]
fn t22_standalone_makeup_counts_as_attended() {
    let (mut p, _, cse) = fixture();
    let makeup = p
        .add_makeup(makeup_input(cse.id, "2026-01-08", "16:00"))
        .unwrap();
    assert_eq!(makeup.kind, SessionKind::Makeup);
    assert_eq!(makeup.slot_id, None);
    assert_eq!(makeup.replaces_session_id, None);
    assert_eq!(makeup.status, Status::Pending);

    mark(&mut p, makeup.id, Status::Attended, dt("2026-01-08T17:00"));

    let summary = p.attendance_summary(cse.id).unwrap();
    assert_eq!(summary.attended, 1);
    assert_eq!(summary.absent, 0);
    assert_eq!(summary.cancelled, 0);
    assert_eq!(summary.pending, 0);
    assert_pct(summary.pct, Some(100.0));
}

/// R5, R9, R12: a rescheduled original reset to pending is kept by sync even when its date
/// becomes a holiday, because a makeup references it (human decision 2026-10-29).
#[test]
fn r5_sync_keeps_session_referenced_by_makeup() {
    let (mut p, term, cse, _, _) = synced_fixture();
    let original = regular_on(&p, cse.id, "2026-01-19");
    let makeup = p
        .reschedule(original.id, makeup_input(cse.id, "2026-01-20", "14:00"))
        .unwrap();
    mark(&mut p, original.id, Status::Pending, dt("2026-01-05T08:00"));
    // 2026-01-21 is also a holiday: its unreferenced pending session is still deleted.
    p.set_holidays(term.id, &holidays(&["2026-01-19", "2026-01-21"]))
        .unwrap();

    let report = p.sync_sessions(d("2026-01-05")).unwrap();
    assert_eq!(
        report,
        SyncReport {
            created: 0,
            deleted: 1
        }
    );

    let kept = regular_on(&p, cse.id, "2026-01-19");
    assert_eq!(kept.id, original.id);
    assert_eq!(kept.status, Status::Pending);
    let makeup_after = sessions_of(&p, cse.id)
        .into_iter()
        .find(|s| s.id == makeup.id)
        .expect("makeup still exists");
    assert_eq!(makeup_after.replaces_session_id, Some(original.id));
    assert!(!dates(&sessions_of(&p, cse.id)).contains(&"2026-01-21".to_string()));
}
