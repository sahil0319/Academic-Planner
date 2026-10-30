//! Session generation: sync creates, is idempotent, honors holidays and inclusive ranges.

mod common;

use chrono::Weekday;
use common::*;
use planner_core::{SessionKind, SlotInput, Status, SyncReport};

const SIX_DATES: [&str; 6] = [
    "2026-01-05",
    "2026-01-07",
    "2026-01-12",
    "2026-01-14",
    "2026-01-19",
    "2026-01-21",
];

/// R4: sync creates one regular session per slot weekday in the term (T04).
#[test]
fn t04_sync_generates_six_sessions() {
    let (mut p, _, cse) = fixture();
    let a = p.add_slot(slot_a(cse.id)).unwrap();
    let b = p.add_slot(slot_b(cse.id)).unwrap();

    let report = p.sync_sessions(d("2026-01-05")).unwrap();
    assert_eq!(
        report,
        SyncReport {
            created: 6,
            deleted: 0
        }
    );

    let sessions = sessions_of(&p, cse.id);
    assert_eq!(dates(&sessions), SIX_DATES);
    for s in &sessions {
        assert_eq!(s.kind, SessionKind::Regular);
        assert_eq!(s.status, Status::Pending);
        assert_eq!(s.replaces_session_id, None);
        let (slot, start, end) = if s.date.format("%a").to_string() == "Mon" {
            (&a, t("09:00"), t("10:00"))
        } else {
            (&b, t("11:00"), t("12:00"))
        };
        assert_eq!(s.slot_id, Some(slot.id));
        assert_eq!(s.start_time, start);
        assert_eq!(s.end_time, end);
        assert_eq!(s.course_id, cse.id);
    }
}

/// R4: a session copies the room of its slot (FR-001), here an explicit slot room.
#[test]
fn t04_session_copies_slot_room() {
    let (mut p, _, cse) = fixture();
    p.add_slot(SlotInput {
        room: Some("B-202".into()),
        ..slot_a(cse.id)
    })
    .unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    let sessions = sessions_of(&p, cse.id);
    assert_eq!(dates(&sessions), ["2026-01-05", "2026-01-12", "2026-01-19"]);
    assert!(sessions.iter().all(|s| s.room.as_deref() == Some("B-202")));
}

/// R4: syncing twice leaves the same rows with the same ids and reports no change (T05).
#[test]
fn t05_sync_twice_same_rows_same_ids() {
    let (mut p, _, cse, _, _) = synced_fixture();
    let before = sessions_of(&p, cse.id);
    assert_eq!(before.len(), 6);

    let report = p.sync_sessions(d("2026-01-05")).unwrap();
    assert_eq!(report, SyncReport::default());
    assert_eq!(sessions_of(&p, cse.id), before);
}

/// R4: a holiday set before the first sync produces no session on that day (T06).
#[test]
fn t06_holiday_before_sync_skips_that_day() {
    let (mut p, term, cse) = fixture();
    p.add_slot(slot_a(cse.id)).unwrap();
    p.add_slot(slot_b(cse.id)).unwrap();
    p.set_holidays(term.id, &holidays(&["2026-01-12"])).unwrap();

    let report = p.sync_sessions(d("2026-01-05")).unwrap();
    assert_eq!(report.created, 5);
    let sessions = sessions_of(&p, cse.id);
    assert_eq!(sessions.len(), 5);
    assert!(sessions.iter().all(|s| s.date != d("2026-01-12")));
}

/// R4, R5: a holiday added later removes the future pending session but keeps a past one (T07).
#[test]
fn t07_holiday_added_after_sync() {
    let (mut p, term, cse, _, _) = synced_fixture();
    p.set_holidays(term.id, &holidays(&["2026-01-19", "2026-01-05"]))
        .unwrap();

    let report = p.sync_sessions(d("2026-01-14")).unwrap();
    assert_eq!(report.created, 0);
    assert_eq!(report.deleted, 1);
    assert_eq!(
        dates(&sessions_of(&p, cse.id)),
        [
            "2026-01-05",
            "2026-01-07",
            "2026-01-12",
            "2026-01-14",
            "2026-01-21"
        ]
    );
}

/// R4: the range is inclusive at the end; a slot valid_to = term end on a slot weekday
/// still produces the session on the end date (T14).
#[test]
fn t14_session_on_term_end_date_is_generated() {
    let (mut p, term, cse) = fixture();
    // The fixture term ends on Sunday 2026-01-25.
    assert_eq!(term.end_date.format("%a").to_string(), "Sun");
    p.add_slot(SlotInput {
        weekday: Weekday::Sun,
        valid_to: Some(term.end_date),
        week_parity: None,
        ..slot_a(cse.id)
    })
    .unwrap();

    p.sync_sessions(d("2026-01-05")).unwrap();
    assert_eq!(
        dates(&sessions_of(&p, cse.id)),
        ["2026-01-11", "2026-01-18", "2026-01-25"]
    );
}

/// R4: an open-ended slot (valid_to = None) also reaches the term end date (T14).
#[test]
fn t14_open_ended_slot_reaches_term_end_date() {
    let (mut p, term, cse) = fixture();
    p.add_slot(SlotInput {
        weekday: Weekday::Sun,
        valid_to: None,
        week_parity: None,
        ..slot_a(cse.id)
    })
    .unwrap();

    p.sync_sessions(d("2026-01-05")).unwrap();
    let sessions = sessions_of(&p, cse.id);
    assert_eq!(sessions.last().map(|s| s.date), Some(term.end_date));
}
