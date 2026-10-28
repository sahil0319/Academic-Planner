//! Changing and ending slots: past sessions are snapshots, future pending ones follow.

mod common;

use chrono::Weekday;
use common::*;
use planner_core::SlotInput;

/// R7, R5: moving slot A to Tuesday from 01-14 keeps past Mondays, drops Monday 19 and
/// creates Tuesday 20 (T10).
#[test]
fn t10_change_slot_weekday_effective_mid_term() {
    let (mut p, _, cse, a, _) = synced_fixture();
    let new_slot = p
        .change_slot(
            a.id,
            SlotInput {
                weekday: Weekday::Tue,
                valid_from: d("2026-01-14"),
                valid_to: None,
                ..slot_a(cse.id)
            },
            d("2026-01-14"),
        )
        .unwrap();
    assert_ne!(new_slot.id, a.id);
    assert_eq!(new_slot.weekday, Weekday::Tue);
    assert_eq!(new_slot.valid_from, d("2026-01-14"));

    // The old slot ends the day before the effective date.
    let before: Vec<i64> = p
        .list_slots(d("2026-01-13"))
        .unwrap()
        .iter()
        .map(|s| s.id)
        .collect();
    assert!(before.contains(&a.id));
    assert!(!before.contains(&new_slot.id));
    let after: Vec<i64> = p
        .list_slots(d("2026-01-14"))
        .unwrap()
        .iter()
        .map(|s| s.id)
        .collect();
    assert!(!after.contains(&a.id));
    assert!(after.contains(&new_slot.id));

    assert_eq!(
        dates(&sessions_of(&p, cse.id)),
        [
            "2026-01-05",
            "2026-01-07",
            "2026-01-12",
            "2026-01-14",
            "2026-01-20",
            "2026-01-21"
        ]
    );
}

/// R7: sessions are snapshots; a time change effective 01-14 leaves 01-05 and 01-12 alone (T11).
#[test]
fn t11_change_slot_time_keeps_past_session_times() {
    let (mut p, _, cse, a, _) = synced_fixture();
    p.change_slot(
        a.id,
        SlotInput {
            start_time: t("10:00"),
            end_time: t("11:00"),
            valid_from: d("2026-01-14"),
            valid_to: None,
            ..slot_a(cse.id)
        },
        d("2026-01-14"),
    )
    .unwrap();

    for day in ["2026-01-05", "2026-01-12"] {
        let s = regular_on(&p, cse.id, day);
        assert_eq!(s.start_time, t("09:00"), "{day}");
        assert_eq!(s.end_time, t("10:00"), "{day}");
    }
    // The first Monday from the effective date follows the new times.
    let s = regular_on(&p, cse.id, "2026-01-19");
    assert_eq!(s.start_time, t("10:00"));
    assert_eq!(s.end_time, t("11:00"));
}

/// R5, R7: ending slot A at 01-11 removes the later pending Mondays and keeps 01-05 (T27).
#[test]
fn t27_end_slot_removes_later_pending_sessions() {
    let (mut p, _, cse, a, _) = synced_fixture();
    p.end_slot(a.id, d("2026-01-11")).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();

    assert_eq!(
        dates(&sessions_of(&p, cse.id)),
        ["2026-01-05", "2026-01-07", "2026-01-14", "2026-01-21"]
    );
}

/// R7 (FR-008): end_slot syncs by itself, so the sessions are gone without an explicit sync.
#[test]
fn t27_end_slot_syncs_internally() {
    let (mut p, _, cse, a, _) = synced_fixture();
    p.end_slot(a.id, d("2026-01-11")).unwrap();

    assert_eq!(
        dates(&sessions_of(&p, cse.id)),
        ["2026-01-05", "2026-01-07", "2026-01-14", "2026-01-21"]
    );
}
