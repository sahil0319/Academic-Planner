//! Timetable slots (add, list valid on a date) and settings (§3 defaults, save).

mod common;

use chrono::Weekday;
use common::*;
use planner_core::{Planner, Settings, Slot, SlotInput};

fn ids(slots: &[Slot]) -> Vec<i64> {
    slots.iter().map(|s| s.id).collect()
}

#[test]
fn add_slot_returns_stored_values() {
    let (mut p, _, cse) = fixture();
    let input = SlotInput {
        room: Some("B-202".into()),
        valid_to: Some(d("2026-01-19")),
        ..slot_a(cse.id)
    };
    let slot = p.add_slot(input.clone()).unwrap();
    assert_eq!(slot.course_id, cse.id);
    assert_eq!(slot.weekday, Weekday::Mon);
    assert_eq!(slot.start_time, t("09:00"));
    assert_eq!(slot.end_time, t("10:00"));
    assert_eq!(slot.room.as_deref(), Some("B-202"));
    assert_eq!(slot.valid_from, input.valid_from);
    assert_eq!(slot.valid_to, input.valid_to);
    assert_eq!(p.list_slots(d("2026-01-12")).unwrap(), [slot]);
}

#[test]
fn list_slots_inclusive_validity() {
    let (mut p, _, cse) = fixture();
    let slot = p
        .add_slot(SlotInput {
            valid_from: d("2026-01-12"),
            valid_to: Some(d("2026-01-19")),
            ..slot_a(cse.id)
        })
        .unwrap();
    assert!(p.list_slots(d("2026-01-11")).unwrap().is_empty());
    assert_eq!(ids(&p.list_slots(d("2026-01-12")).unwrap()), [slot.id]);
    assert_eq!(ids(&p.list_slots(d("2026-01-19")).unwrap()), [slot.id]);
    assert!(p.list_slots(d("2026-01-20")).unwrap().is_empty());
}

#[test]
fn list_slots_open_end_until_term_end() {
    let (mut p, _, cse) = fixture();
    let slot = p.add_slot(slot_a(cse.id)).unwrap();
    assert_eq!(slot.valid_to, None);
    assert_eq!(ids(&p.list_slots(d("2026-01-25")).unwrap()), [slot.id]);
    assert!(p.list_slots(d("2026-01-26")).unwrap().is_empty());
}

#[test]
fn list_slots_excludes_archived_course() {
    let (mut p, _, cse) = fixture();
    let other = p.create_course(course_input("MTH110")).unwrap();
    p.add_slot(slot_a(cse.id)).unwrap();
    let kept = p.add_slot(slot_b(other.id)).unwrap();
    p.set_archived(cse.id, true).unwrap();
    assert_eq!(ids(&p.list_slots(d("2026-01-12")).unwrap()), [kept.id]);
}

#[test]
fn list_slots_ordered_by_weekday_then_start() {
    let (mut p, _, cse) = fixture();
    let wed = p.add_slot(slot_b(cse.id)).unwrap();
    let mon_late = p
        .add_slot(SlotInput {
            start_time: t("14:00"),
            end_time: t("15:30"),
            ..slot_a(cse.id)
        })
        .unwrap();
    let mon_early = p.add_slot(slot_a(cse.id)).unwrap();
    assert_eq!(
        ids(&p.list_slots(d("2026-01-12")).unwrap()),
        [mon_early.id, mon_late.id, wed.id]
    );
}

#[test]
fn settings_defaults_on_fresh_db() {
    let p = Planner::open_in_memory().unwrap();
    assert_eq!(
        p.settings().unwrap(),
        Settings {
            class_reminder_minutes: 15,
            upcoming_window_days: 7,
            notifications_enabled: true,
        }
    );
}

#[test]
fn save_settings_round_trip() {
    let mut p = Planner::open_in_memory().unwrap();
    let s = Settings {
        class_reminder_minutes: 30,
        upcoming_window_days: 14,
        notifications_enabled: false,
    };
    p.save_settings(&s).unwrap();
    assert_eq!(p.settings().unwrap(), s);

    let s2 = Settings {
        notifications_enabled: true,
        ..s
    };
    p.save_settings(&s2).unwrap();
    assert_eq!(p.settings().unwrap(), s2);
}
