//! M12: bi-weekly slots (R54-R60, docs/spec.md §13.7, T55-T60).
//!
//! The fixtures use a term from 2026-01-05 (Mon) to 2026-02-01 (Sun), so odd weeks start on
//! 01-05, 01-19 and even weeks on 01-12, 01-26.

mod common;

use chrono::NaiveDate;
use common::*;
use planner_core::{
    term_week_parity, Course, Error, Planner, Slot, SlotInput, Status, TermInput, WeekParity,
};
use rusqlite::Connection;
use serde_json::{json, Value};
use tempfile::TempDir;

/// Planner with an active term 2026-01-05..2026-02-01 and course CSE101.
fn biweekly_fixture() -> (Planner, Course) {
    let mut p = Planner::open_in_memory().unwrap();
    p.save_term(TermInput {
        id: None,
        name: "Winter 2026".into(),
        start_date: d("2026-01-05"),
        end_date: d("2026-02-01"),
    })
    .unwrap();
    let course = p.create_course(fixture_course()).unwrap();
    (p, course)
}

/// Monday 09:00 slot A with the given parity, valid from 01-05.
fn parity_slot(course_id: i64, parity: Option<WeekParity>) -> SlotInput {
    SlotInput {
        week_parity: parity,
        ..slot_a(course_id)
    }
}

/// Planner with an odd Monday slot of CSE101 added and synced on 01-05.
fn odd_monday() -> (Planner, Course, Slot) {
    let (mut p, course) = biweekly_fixture();
    let slot = p
        .add_slot(parity_slot(course.id, Some(WeekParity::Odd)))
        .unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    (p, course, slot)
}

/// Slots of any course valid on `on`, ordered like `list_slots`.
fn slots_on(p: &Planner, on: &str) -> Vec<Slot> {
    p.list_slots(d(on)).unwrap()
}

// ---- T55: R54 ----

/// R54: term weeks run Monday to Sunday, week 1 holds the term start, odd weeks are 1, 3, 5.
#[test]
fn t55_term_week_parity_values() {
    let start = d("2026-01-05");
    let cases = [
        ("2026-01-05", WeekParity::Odd),
        ("2026-01-11", WeekParity::Odd),
        ("2026-01-12", WeekParity::Even),
        ("2026-01-21", WeekParity::Odd),
        ("2026-01-04", WeekParity::Even),
    ];
    for (day, expected) in cases {
        assert_eq!(term_week_parity(start, d(day)), expected, "date {day}");
    }
}

/// R54: a term that starts on a Wednesday counts its own week as week 1, including the
/// Monday before the start.
#[test]
fn t55_wednesday_start_counts_own_week_as_week_one() {
    let start = d("2026-01-07");
    let cases = [
        ("2026-01-05", WeekParity::Odd),
        ("2026-01-07", WeekParity::Odd),
        ("2026-01-11", WeekParity::Odd),
        ("2026-01-12", WeekParity::Even),
        ("2026-01-19", WeekParity::Odd),
        ("2026-01-04", WeekParity::Even),
    ];
    for (day, expected) in cases {
        assert_eq!(term_week_parity(start, d(day)), expected, "date {day}");
    }
}

/// R54: dates well before the term keep alternating.
#[test]
fn t55_dates_before_term_alternate() {
    let start = d("2026-01-05");
    assert_eq!(term_week_parity(start, d("2025-12-29")), WeekParity::Even);
    assert_eq!(term_week_parity(start, d("2025-12-22")), WeekParity::Odd);
    assert_eq!(term_week_parity(start, d("2025-12-15")), WeekParity::Even);
}

// ---- T56: R55 ----

/// R55: an odd Monday slot produces sessions only in odd weeks; a slot with no parity still
/// produces every week.
#[test]
fn t56_parity_slot_produces_only_matching_weeks() {
    let (mut p, course) = biweekly_fixture();
    p.add_slot(parity_slot(course.id, Some(WeekParity::Odd)))
        .unwrap();
    p.add_slot(slot_b(course.id)).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();

    let all = dates(&sessions_of(&p, course.id));
    let mondays: Vec<&String> = all
        .iter()
        .filter(|day| {
            ["2026-01-05", "2026-01-12", "2026-01-19", "2026-01-26"].contains(&day.as_str())
        })
        .collect();
    assert_eq!(mondays, ["2026-01-05", "2026-01-19"]);
    let wednesdays: Vec<&String> = all
        .iter()
        .filter(|day| {
            ["2026-01-07", "2026-01-14", "2026-01-21", "2026-01-28"].contains(&day.as_str())
        })
        .collect();
    assert_eq!(
        wednesdays,
        ["2026-01-07", "2026-01-14", "2026-01-21", "2026-01-28"]
    );
}

/// R55: a holiday on 01-19 skips that week's session and does not move the pattern to 01-26.
#[test]
fn t56_holiday_skips_without_shifting_the_pattern() {
    let (mut p, course, _) = odd_monday();
    assert_eq!(
        dates(&sessions_of(&p, course.id)),
        ["2026-01-05", "2026-01-19"]
    );

    let term = p.active_term().unwrap().unwrap();
    p.set_holidays(term.id, &holidays(&["2026-01-19"])).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();

    assert_eq!(dates(&sessions_of(&p, course.id)), ["2026-01-05"]);
}

/// R55, R6: syncing again with a parity slot changes nothing.
#[test]
fn t56_sync_is_idempotent_with_parity() {
    let (mut p, course, _) = odd_monday();
    let before = sessions_of(&p, course.id);
    let report = p.sync_sessions(d("2026-01-05")).unwrap();
    assert_eq!((report.created, report.deleted), (0, 0));
    assert_eq!(sessions_of(&p, course.id), before);
}

/// R55, R7: the stored parity round trips through `add_slot` and `list_slots`.
#[test]
fn t56_parity_is_stored_on_slots() {
    let (mut p, course) = biweekly_fixture();
    let even = p
        .add_slot(parity_slot(course.id, Some(WeekParity::Even)))
        .unwrap();
    assert_eq!(even.week_parity, Some(WeekParity::Even));
    let weekly = p.add_slot(slot_b(course.id)).unwrap();
    assert_eq!(weekly.week_parity, None);

    let listed = slots_on(&p, "2026-01-12");
    assert_eq!(listed.len(), 2);
    assert_eq!(
        listed.iter().find(|s| s.id == even.id).unwrap().week_parity,
        Some(WeekParity::Even)
    );
    assert_eq!(
        listed
            .iter()
            .find(|s| s.id == weekly.id)
            .unwrap()
            .week_parity,
        None
    );
}

/// R7, R55: changing a slot's parity from a date keeps the old row (ended the day before),
/// starts a new row on that date, keeps past sessions, and replaces later pending ones.
#[test]
fn t56_change_slot_parity_keeps_old_row_and_starts_new_one() {
    let (mut p, course, old) = odd_monday();
    let new = p
        .change_slot(
            old.id,
            parity_slot(course.id, Some(WeekParity::Even)),
            d("2026-01-12"),
        )
        .unwrap();

    assert_ne!(new.id, old.id);
    assert_eq!(new.week_parity, Some(WeekParity::Even));
    assert_eq!(new.valid_from, d("2026-01-12"));

    let old_row = slots_on(&p, "2026-01-11")
        .into_iter()
        .find(|s| s.id == old.id)
        .expect("old row still listed on its last day");
    assert_eq!(old_row.week_parity, Some(WeekParity::Odd));
    assert_eq!(old_row.valid_to, Some(d("2026-01-11")));
    let on_change: Vec<i64> = slots_on(&p, "2026-01-12").iter().map(|s| s.id).collect();
    assert_eq!(on_change, [new.id]);

    assert_eq!(
        dates(&sessions_of(&p, course.id)),
        ["2026-01-05", "2026-01-12", "2026-01-26"]
    );
}

/// R7, R55: turning a bi-weekly slot into a weekly one from a date adds the skipped weeks.
#[test]
fn t56_change_slot_to_weekly_adds_skipped_weeks() {
    let (mut p, course, old) = odd_monday();
    let new = p
        .change_slot(old.id, parity_slot(course.id, None), d("2026-01-12"))
        .unwrap();
    assert_eq!(new.week_parity, None);
    assert_eq!(
        dates(&sessions_of(&p, course.id)),
        ["2026-01-05", "2026-01-12", "2026-01-19", "2026-01-26"]
    );
}

/// R58: `SessionView.biweekly` is true for sessions of a bi-weekly slot only, not for weekly
/// slots or makeups.
#[test]
fn t56_session_view_biweekly_flag() {
    let (mut p, course) = biweekly_fixture();
    p.add_slot(parity_slot(course.id, Some(WeekParity::Odd)))
        .unwrap();
    p.add_slot(slot_b(course.id)).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    p.add_makeup(makeup_input(course.id, "2026-01-06", "14:00"))
        .unwrap();

    let views = p.sessions_for_course(course.id, after_term()).unwrap();
    assert!(views.iter().any(|v| v.biweekly));
    for view in &views {
        let day = view.session.date;
        let expected = view.session.kind == planner_core::SessionKind::Regular
            && chrono::Datelike::weekday(&day) == chrono::Weekday::Mon;
        assert_eq!(view.biweekly, expected, "session on {day}");
    }
}

// ---- T57: R57 ----

/// R57: shifting from 01-19 keeps the attended 01-05 session, deletes pending 01-19, creates
/// 01-26; the old slot ends 01-18 and a new even slot starts 01-19.
#[test]
fn t57_shift_biweekly_splits_slot_and_moves_pending_session() {
    let (mut p, course, old) = odd_monday();
    let first = regular_on(&p, course.id, "2026-01-05");
    mark(&mut p, first.id, Status::Attended, dt("2026-01-05T10:00"));

    p.shift_biweekly(course.id, d("2026-01-19")).unwrap();

    let sessions = sessions_of(&p, course.id);
    assert_eq!(dates(&sessions), ["2026-01-05", "2026-01-26"]);
    assert_eq!(sessions[0].id, first.id);
    assert_eq!(sessions[0].status, Status::Attended);

    let old_row = slots_on(&p, "2026-01-18")
        .into_iter()
        .find(|s| s.id == old.id)
        .expect("old slot valid on 01-18");
    assert_eq!(old_row.valid_to, Some(d("2026-01-18")));
    assert_eq!(old_row.week_parity, Some(WeekParity::Odd));

    let current = slots_on(&p, "2026-01-19");
    assert_eq!(current.len(), 1);
    assert_ne!(current[0].id, old.id);
    assert_eq!(current[0].week_parity, Some(WeekParity::Even));
    assert_eq!(current[0].valid_from, d("2026-01-19"));
    assert_eq!(current[0].valid_to, None);
}

/// R57: a bi-weekly slot that starts on or after `from` is flipped in place, with no new row.
#[test]
fn t57_slot_starting_on_from_is_flipped_in_place() {
    let (mut p, course) = biweekly_fixture();
    let slot = p
        .add_slot(SlotInput {
            valid_from: d("2026-01-19"),
            ..parity_slot(course.id, Some(WeekParity::Odd))
        })
        .unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    assert_eq!(dates(&sessions_of(&p, course.id)), ["2026-01-19"]);

    p.shift_biweekly(course.id, d("2026-01-19")).unwrap();

    let rows = slots_on(&p, "2026-01-19");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, slot.id);
    assert_eq!(rows[0].week_parity, Some(WeekParity::Even));
    assert_eq!(rows[0].valid_from, d("2026-01-19"));
    assert_eq!(dates(&sessions_of(&p, course.id)), ["2026-01-26"]);
}

/// R57, R5: past sessions and non-pending sessions are never touched by a shift.
#[test]
fn t57_shift_leaves_past_and_non_pending_sessions_alone() {
    let (mut p, course, _) = odd_monday();
    let past_pending = regular_on(&p, course.id, "2026-01-05");
    let cancelled = regular_on(&p, course.id, "2026-01-19");
    mark(
        &mut p,
        cancelled.id,
        Status::Cancelled,
        dt("2026-01-05T08:00"),
    );

    p.shift_biweekly(course.id, d("2026-01-19")).unwrap();

    let sessions = sessions_of(&p, course.id);
    assert_eq!(dates(&sessions), ["2026-01-05", "2026-01-19", "2026-01-26"]);
    assert_eq!(sessions[0].id, past_pending.id);
    assert_eq!(sessions[0].status, Status::Pending);
    assert_eq!(sessions[1].id, cancelled.id);
    assert_eq!(sessions[1].status, Status::Cancelled);
}

/// R57: weekly slots of the same course are not changed by a shift.
#[test]
fn t57_shift_ignores_weekly_slots_of_the_course() {
    let (mut p, course, _) = odd_monday();
    let weekly = p.add_slot(slot_b(course.id)).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    let before: Vec<String> = dates(&sessions_of(&p, course.id))
        .into_iter()
        .filter(|day| {
            ["2026-01-07", "2026-01-14", "2026-01-21", "2026-01-28"].contains(&day.as_str())
        })
        .collect();

    p.shift_biweekly(course.id, d("2026-01-19")).unwrap();

    let row = slots_on(&p, "2026-01-19")
        .into_iter()
        .find(|s| s.id == weekly.id)
        .unwrap();
    assert_eq!(row.week_parity, None);
    assert_eq!(row.valid_to, None);
    let after: Vec<String> = dates(&sessions_of(&p, course.id))
        .into_iter()
        .filter(|day| {
            ["2026-01-07", "2026-01-14", "2026-01-21", "2026-01-28"].contains(&day.as_str())
        })
        .collect();
    assert_eq!(after, before);
}

// ---- T58: R57 errors ----

/// R57: a course with only weekly slots is a validation error, and nothing changes.
#[test]
fn t58_all_weekly_course_is_validation_error() {
    let (mut p, course) = biweekly_fixture();
    p.add_slot(slot_a(course.id)).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    let before = sessions_of(&p, course.id);

    let result = p.shift_biweekly(course.id, d("2026-01-19"));
    assert!(matches!(result, Err(Error::Validation(_))), "{result:?}");
    assert_eq!(sessions_of(&p, course.id), before);
    assert_eq!(slots_on(&p, "2026-01-19").len(), 1);
}

/// R57: an unknown course is `Error::NotFound`.
#[test]
fn t58_unknown_course_is_not_found() {
    let (mut p, _) = biweekly_fixture();
    let result = p.shift_biweekly(9999, d("2026-01-19"));
    assert!(matches!(result, Err(Error::NotFound { .. })), "{result:?}");
}

/// R57: a bi-weekly slot that is not valid on `from` does not count.
#[test]
fn t58_biweekly_slot_not_valid_on_from_is_validation_error() {
    let (mut p, course) = biweekly_fixture();
    p.add_slot(SlotInput {
        valid_to: Some(d("2026-01-10")),
        ..parity_slot(course.id, Some(WeekParity::Odd))
    })
    .unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();

    let result = p.shift_biweekly(course.id, d("2026-01-19"));
    assert!(matches!(result, Err(Error::Validation(_))), "{result:?}");
}

// ---- T59: R59 import ----

/// Planner with the active term "Semester 7" (2026-01-05 to 2026-05-01).
fn semester() -> Planner {
    let mut p = Planner::open_in_memory().unwrap();
    p.save_term(TermInput {
        id: None,
        name: "Semester 7".into(),
        start_date: d("2026-01-05"),
        end_date: d("2026-05-01"),
    })
    .unwrap();
    p
}

/// Import effective date of the T59 cases: Monday 2026-01-12, an even term week.
fn effective() -> NaiveDate {
    d("2026-01-12")
}

/// A document with one course `CSE 4711` whose single Monday slot is `slot`.
fn doc_with_slot(slot: Value) -> String {
    json!({
        "format": "academic-planner-courses",
        "version": 1,
        "courses": [{"code": "CSE 4711", "name": "Artificial Intelligence", "slots": [slot]}]
    })
    .to_string()
}

fn monday_slot(extra: Option<(&str, Value)>) -> String {
    let mut slot = json!({"day": "Mon", "start": "09:00", "end": "10:00"});
    if let Some((key, value)) = extra {
        slot[key] = value;
    }
    doc_with_slot(slot)
}

/// Imports `text` into a fresh Semester 7 planner at the effective date and returns the
/// planner plus the parity of the one imported slot.
fn import_parity(text: &str) -> (Planner, Option<WeekParity>) {
    let mut p = semester();
    p.apply_import(text, effective(), effective()).unwrap();
    let slots = slots_on(&p, "2026-01-12");
    assert_eq!(slots.len(), 1, "one slot imported");
    let parity = slots[0].week_parity;
    (p, parity)
}

/// R59, R54: `this_week` becomes the parity of the effective date's week (even for 01-12).
#[test]
fn t59_import_this_week_is_parity_of_effective_week() {
    let (p, parity) = import_parity(&monday_slot(Some(("biweekly", json!("this_week")))));
    assert_eq!(parity, Some(WeekParity::Even));
    let course = p.list_courses(true).unwrap().remove(0);
    let first: Vec<String> = dates(&sessions_of(&p, course.id))
        .into_iter()
        .take(3)
        .collect();
    assert_eq!(first, ["2026-01-12", "2026-01-26", "2026-02-09"]);
}

/// R59, R54: `next_week` becomes the other parity (odd for 01-12).
#[test]
fn t59_import_next_week_is_the_other_parity() {
    let (p, parity) = import_parity(&monday_slot(Some(("biweekly", json!("next_week")))));
    assert_eq!(parity, Some(WeekParity::Odd));
    let course = p.list_courses(true).unwrap().remove(0);
    let first: Vec<String> = dates(&sessions_of(&p, course.id))
        .into_iter()
        .take(3)
        .collect();
    assert_eq!(first, ["2026-01-19", "2026-02-02", "2026-02-16"]);
}

/// R59: `"biweekly": null` and an absent key both mean every week.
#[test]
fn t59_import_null_and_absent_mean_weekly() {
    let (_, null_parity) = import_parity(&monday_slot(Some(("biweekly", Value::Null))));
    assert_eq!(null_parity, None);
    let (_, absent_parity) = import_parity(&monday_slot(None));
    assert_eq!(absent_parity, None);
}

/// R59, R45, R48: importing the same bi-weekly file again plans nothing.
#[test]
fn t59_second_import_of_same_file_is_empty() {
    let text = monday_slot(Some(("biweekly", json!("this_week"))));
    let (mut p, _) = import_parity(&text);

    let plan = p.preview_import(&text, effective()).unwrap();
    assert!(plan.errors.is_empty(), "{:?}", plan.errors);
    assert!(plan.is_empty(), "{plan:?}");

    let before = p.export_json(dt("2026-01-14T00:00")).unwrap();
    p.apply_import(&text, effective(), effective()).unwrap();
    assert_eq!(p.export_json(dt("2026-01-14T00:00")).unwrap(), before);
}

/// R45, R59: R45 compares parity, so the same slot with the other parity is a slot change
/// that ends the old slot and adds the new one.
#[test]
fn t59_changed_parity_is_a_slot_change() {
    let (mut p, _) = import_parity(&monday_slot(Some(("biweekly", json!("this_week")))));
    let other = monday_slot(Some(("biweekly", json!("next_week"))));

    let plan = p.preview_import(&other, effective()).unwrap();
    assert!(plan.errors.is_empty(), "{:?}", plan.errors);
    assert_eq!(plan.update.len(), 1, "{plan:?}");
    let change = plan.update[0].slots.as_ref().expect("slots change planned");
    assert_eq!((change.end.len(), change.add.len()), (1, 1));

    p.apply_import(&other, effective(), effective()).unwrap();
    let current = slots_on(&p, "2026-01-12");
    assert_eq!(current.len(), 1);
    assert_eq!(current[0].week_parity, Some(WeekParity::Odd));
}

/// R59: any other value is a validation error at `courses[0].slots[0].biweekly`, and a
/// failed apply writes nothing.
#[test]
fn t59_import_invalid_biweekly_value_is_error_with_path() {
    let mut p = semester();
    let text = monday_slot(Some(("biweekly", json!("monthly"))));
    let before = p.export_json(dt("2026-01-14T00:00")).unwrap();

    let plan = p.preview_import(&text, effective()).unwrap();
    assert!(
        plan.errors
            .iter()
            .any(|issue| issue.path == "courses[0].slots[0].biweekly"),
        "{:?}",
        plan.errors
    );
    assert!(plan.add.is_empty() && plan.update.is_empty());

    let result = p.apply_import(&text, effective(), effective());
    match result {
        Err(Error::Validation(message)) => {
            assert!(
                message.contains("courses[0].slots[0].biweekly"),
                "{message}"
            )
        }
        other => panic!("expected validation error, got {other:?}"),
    }
    assert_eq!(p.export_json(dt("2026-01-14T00:00")).unwrap(), before);
}

// ---- T60: R60 backup and migration ----

const NOW: &str = "2026-01-25T18:30";

fn parse(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

/// Planner with an odd Monday slot and a weekly Wednesday slot, synced.
fn mixed_planner() -> Planner {
    let (mut p, course) = biweekly_fixture();
    p.add_slot(parity_slot(course.id, Some(WeekParity::Odd)))
        .unwrap();
    p.add_slot(slot_b(course.id)).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    p
}

/// R60: a backup carries `week_parity` on slot rows, `schema_version` is 3, and restoring it
/// keeps the parity (exporting again gives the same document except `exported_at`).
#[test]
fn t60_backup_round_trip_keeps_parity() {
    let source = mixed_planner();
    let text = source.export_json(dt(NOW)).unwrap();

    let doc = parse(&text);
    assert_eq!(doc["schema_version"], 3);
    let parities: Vec<&Value> = doc["slots"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| &row["week_parity"])
        .collect();
    assert!(parities.contains(&&json!("odd")), "{parities:?}");
    assert!(parities.contains(&&Value::Null), "{parities:?}");

    let mut target = Planner::open_in_memory().unwrap();
    target.restore_json(&text).unwrap();
    let slots = slots_on(&target, "2026-01-12");
    assert_eq!(slots.len(), 2);
    assert!(slots.iter().any(|s| s.week_parity == Some(WeekParity::Odd)));
    assert!(slots.iter().any(|s| s.week_parity.is_none()));

    let mut first = doc;
    let mut second = parse(&target.export_json(dt("2026-02-01T08:00")).unwrap());
    first.as_object_mut().unwrap().remove("exported_at");
    second.as_object_mut().unwrap().remove("exported_at");
    assert_eq!(first, second);
}

/// R60: a version 2 backup (no `week_parity` on slot rows) restores with every slot weekly,
/// even over a database that held a bi-weekly slot.
#[test]
fn t60_version_2_backup_restores_weekly_slots() {
    let mut doc = parse(&mixed_planner().export_json(dt(NOW)).unwrap());
    doc["schema_version"] = Value::from(2);
    for row in doc["slots"].as_array_mut().unwrap() {
        row.as_object_mut().unwrap().remove("week_parity").unwrap();
    }

    let mut target = mixed_planner();
    target
        .restore_json(&serde_json::to_string(&doc).unwrap())
        .unwrap();

    let slots = slots_on(&target, "2026-01-12");
    assert_eq!(slots.len(), 2);
    assert!(slots.iter().all(|s| s.week_parity.is_none()), "{slots:?}");
}

/// R60, R26: opening a version 2 file database applies migration 003, writes the
/// pre-migration copy, keeps the data, and leaves existing slots weekly.
#[test]
fn t60_open_version_2_db_applies_migration_003() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("planner.sqlite");
    {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        conn.execute_batch(include_str!("../src/migrations/001_init.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../src/migrations/002_grading.sql"))
            .unwrap();
        conn.pragma_update(None, "user_version", 2).unwrap();
        conn.execute(
            "INSERT INTO term (id, name, start_date, end_date, is_active) \
             VALUES (1, 'Winter 2026', '2026-01-05', '2026-02-01', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO course (id, term_id, code, name) VALUES (1, 1, 'CSE101', 'Intro')",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO slot (id, course_id, weekday, start_time, end_time, valid_from) \
             VALUES (1, 1, 1, '09:00', '10:00', '2026-01-05')",
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
        .join("backups/pre-migrate-v2-to-v3.sqlite")
        .exists());
    let stored: Option<String> = conn
        .query_row("SELECT week_parity FROM slot WHERE id = 1", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(stored, None);

    let slots = p.list_slots(d("2026-01-12")).unwrap();
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].week_parity, None);
}
