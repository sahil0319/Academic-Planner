//! M15 (docs/spec.md §13.10): a day off has a kind (holiday, mid exam, final exam); every kind
//! skips classes, one kind per date, and backups carry the kind.

mod common;

use common::*;
use planner_core::{DayOffKind, Error, Holiday, Planner, Result, SCHEMA_VERSION};
use rusqlite::Connection;
use serde_json::{json, Value};
use tempfile::TempDir;

const NOW: &str = "2026-01-25T18:30";
const LATER: &str = "2026-02-01T08:00";

fn day_off(date: &str, label: Option<&str>, kind: DayOffKind) -> Holiday {
    Holiday {
        date: d(date),
        label: label.map(Into::into),
        kind,
    }
}

/// The message of a `Validation` error; panics on any other result.
fn validation_message(result: Result<()>) -> String {
    match result {
        Err(Error::Validation(message)) => message,
        other => panic!("expected Error::Validation, got {other:?}"),
    }
}

fn holidays_of(p: &Planner) -> Vec<Holiday> {
    p.active_term().unwrap().unwrap().holidays
}

fn export(p: &Planner, now: &str) -> String {
    p.export_json(dt(now)).unwrap()
}

fn parse(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn without_exported_at(text: &str) -> Value {
    let mut doc = parse(text);
    doc.as_object_mut().unwrap().remove("exported_at").unwrap();
    doc
}

/// A synced fixture with one day off of each kind: holiday 01-07, mid exam 01-14, final exam
/// 01-21, all labeled.
fn planner_with_three_kinds() -> Planner {
    let (mut p, term, _, _, _) = synced_fixture();
    p.set_holidays(
        term.id,
        &[
            day_off("2026-01-07", Some("Break"), DayOffKind::Holiday),
            day_off("2026-01-14", Some("Week 2"), DayOffKind::Midterm),
            day_off("2026-01-21", None, DayOffKind::Final),
        ],
    )
    .unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    p
}

/// R71, R72: a mid exam day and a final exam day skip classes like a holiday; other slot days
/// keep their sessions.
#[test]
fn t71_exam_days_skip_classes() {
    let (mut p, term, cse, _, _) = synced_fixture();
    assert_eq!(
        dates(&sessions_of(&p, cse.id)),
        [
            "2026-01-05",
            "2026-01-07",
            "2026-01-12",
            "2026-01-14",
            "2026-01-19",
            "2026-01-21"
        ]
    );

    p.set_holidays(
        term.id,
        &[
            day_off("2026-01-12", None, DayOffKind::Midterm),
            day_off("2026-01-19", None, DayOffKind::Final),
        ],
    )
    .unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();

    assert_eq!(
        dates(&sessions_of(&p, cse.id)),
        ["2026-01-05", "2026-01-07", "2026-01-14", "2026-01-21"]
    );
    let kinds: Vec<(String, DayOffKind)> = holidays_of(&p)
        .iter()
        .map(|h| (ymd(h.date), h.kind))
        .collect();
    assert_eq!(
        kinds,
        [
            ("2026-01-12".to_string(), DayOffKind::Midterm),
            ("2026-01-19".to_string(), DayOffKind::Final)
        ]
    );
}

/// R71, R72: a holiday, a mid exam day, and a final exam day skip classes alike; dropping the
/// final exam day and syncing brings its class back.
#[test]
fn t71_holiday_midterm_final_skip_alike() {
    let (mut p, term, cse, _, _) = synced_fixture();
    let all_three = [
        day_off("2026-01-07", None, DayOffKind::Holiday),
        day_off("2026-01-12", None, DayOffKind::Midterm),
        day_off("2026-01-14", None, DayOffKind::Final),
    ];
    p.set_holidays(term.id, &all_three).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    assert_eq!(
        dates(&sessions_of(&p, cse.id)),
        ["2026-01-05", "2026-01-19", "2026-01-21"]
    );

    p.set_holidays(term.id, &all_three[..2]).unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    assert_eq!(
        dates(&sessions_of(&p, cse.id)),
        ["2026-01-05", "2026-01-14", "2026-01-19", "2026-01-21"]
    );
}

/// R71: a date that already has a day off of another kind is rejected with a message naming
/// the stored kind; nothing changes.
#[test]
fn t72_date_with_other_kind_is_rejected() {
    let (mut p, term, cse, _, _) = synced_fixture();
    p.set_holidays(
        term.id,
        &[day_off("2026-01-12", Some("Break"), DayOffKind::Holiday)],
    )
    .unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    let holidays_before = holidays_of(&p);
    let sessions_before = sessions_of(&p, cse.id);

    let message = validation_message(p.set_holidays(
        term.id,
        &[
            day_off("2026-01-12", Some("Break"), DayOffKind::Holiday),
            day_off("2026-01-12", None, DayOffKind::Midterm),
        ],
    ));

    assert_eq!(message, "January 12, 2026 is already a holiday");
    assert_eq!(holidays_of(&p), holidays_before);
    assert_eq!(sessions_of(&p, cse.id), sessions_before);
}

/// R71: with nothing stored for the date, the message names the kind of the first entry.
#[test]
fn t72_date_listed_twice_names_first_kind() {
    let (mut p, term, _, _, _) = synced_fixture();

    let message = validation_message(p.set_holidays(
        term.id,
        &[
            day_off("2026-01-19", None, DayOffKind::Final),
            day_off("2026-01-19", None, DayOffKind::Midterm),
        ],
    ));

    assert_eq!(message, "January 19, 2026 is already a final exam day");
    assert!(holidays_of(&p).is_empty());
}

/// R71: all three kinds are stored in one call and loaded sorted by date with their labels and
/// kinds.
#[test]
fn t72_kinds_stored_and_loaded() {
    let (mut p, term, _, _, _) = synced_fixture();

    p.set_holidays(
        term.id,
        &[
            day_off("2026-01-21", Some("Finals"), DayOffKind::Final),
            day_off("2026-01-07", None, DayOffKind::Holiday),
            day_off("2026-01-14", Some("Week 2"), DayOffKind::Midterm),
        ],
    )
    .unwrap();

    assert_eq!(
        holidays_of(&p),
        [
            day_off("2026-01-07", None, DayOffKind::Holiday),
            day_off("2026-01-14", Some("Week 2"), DayOffKind::Midterm),
            day_off("2026-01-21", Some("Finals"), DayOffKind::Final),
        ]
    );
}

/// R72, R25: opening a version 2 file database adds the kind column, and the rows that
/// existed stay holidays.
#[test]
fn t72_existing_rows_stay_holiday_after_migration() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("planner.sqlite");
    {
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON").unwrap();
        conn.execute_batch(include_str!("../src/migrations/001_init.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../src/migrations/002_grading.sql"))
            .unwrap();
        conn.execute_batch(include_str!("../src/grading_seed.sql"))
            .unwrap();
        conn.pragma_update(None, "user_version", 2).unwrap();
        conn.execute(
            "INSERT INTO term (id, name, start_date, end_date, is_active) \
             VALUES (1, 'Winter 2026', '2026-01-05', '2026-01-25', 1)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO holiday (term_id, date, label) VALUES (1, '2026-01-19', 'Break')",
            [],
        )
        .unwrap();
    }

    let p = Planner::open(&path).unwrap();

    let conn = Connection::open(&path).unwrap();
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(version, SCHEMA_VERSION);
    assert_eq!(
        holidays_of(&p),
        [day_off("2026-01-19", Some("Break"), DayOffKind::Holiday)]
    );
}

/// R75: the export writes each day off's kind, a restore keeps it, and exporting again gives
/// the same document except `exported_at`.
#[test]
fn t73_backup_keeps_kinds() {
    let source = planner_with_three_kinds();
    let first = export(&source, NOW);

    let doc = parse(&first);
    let rows = doc["holidays"].as_array().unwrap();
    let kinds: Vec<&Value> = rows.iter().map(|row| &row["kind"]).collect();
    assert_eq!(
        kinds,
        [&json!("holiday"), &json!("midterm"), &json!("final")]
    );

    let mut fresh = Planner::open_in_memory().unwrap();
    fresh.restore_json(&first).unwrap();

    assert_eq!(holidays_of(&fresh), holidays_of(&source));
    let second = export(&fresh, LATER);
    assert_ne!(parse(&first)["exported_at"], parse(&second)["exported_at"]);
    assert_eq!(without_exported_at(&first), without_exported_at(&second));
}

/// R75, D24: a backup of schema version 2 restores every day off as a holiday, with the `kind`
/// keys removed and with them kept (an older document's `kind` is ignored).
#[test]
fn t73_older_backup_restores_every_day_off_as_holiday() {
    let source = planner_with_three_kinds();
    let mut with_kind = parse(&export(&source, NOW));
    with_kind["schema_version"] = json!(2);
    let mut without_kind = with_kind.clone();
    for row in without_kind["holidays"].as_array_mut().unwrap() {
        row.as_object_mut().unwrap().remove("kind").unwrap();
    }

    for doc in [without_kind, with_kind] {
        let mut fresh = Planner::open_in_memory().unwrap();
        fresh.restore_json(&doc.to_string()).unwrap();

        assert_eq!(
            holidays_of(&fresh),
            [
                day_off("2026-01-07", Some("Break"), DayOffKind::Holiday),
                day_off("2026-01-14", Some("Week 2"), DayOffKind::Holiday),
                day_off("2026-01-21", None, DayOffKind::Holiday),
            ]
        );
    }
}

/// R75, R22: a backup of the current version needs a `kind` on every day off and rejects an
/// unknown kind; the database stays unchanged.
#[test]
fn t73_current_backup_kind_missing_or_invalid_is_rejected() {
    let doc = parse(&export(&planner_with_three_kinds(), NOW));

    let mut missing = doc.clone();
    missing["holidays"][1]
        .as_object_mut()
        .unwrap()
        .remove("kind")
        .unwrap();
    let mut invalid = doc;
    invalid["holidays"][1]["kind"] = json!("exam");

    let (mut target, term, _) = fixture();
    target
        .set_holidays(
            term.id,
            &[day_off("2026-01-19", Some("Keep me"), DayOffKind::Final)],
        )
        .unwrap();
    let before = without_exported_at(&export(&target, NOW));

    let message = validation_message(target.restore_json(&missing.to_string()));
    assert!(message.contains("missing field `kind`"), "{message}");
    assert_eq!(without_exported_at(&export(&target, NOW)), before);

    let message = validation_message(target.restore_json(&invalid.to_string()));
    assert!(message.contains("holiday kind"), "{message}");
    assert_eq!(without_exported_at(&export(&target, NOW)), before);
}
