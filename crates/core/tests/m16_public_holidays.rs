//! M16 (docs/spec.md §13.11): the cached government public holidays of a country. The cache
//! never changes classes by itself, backups and CSV leave it out, and the country setting
//! rides in backups like every other setting.

mod common;

use common::*;
use planner_core::{DayOffKind, Holiday, Planner, PublicHoliday};
use serde_json::{json, Value};

const NOW: &str = "2026-01-25T18:30";

fn ph(date: &str, name: &str) -> PublicHoliday {
    PublicHoliday {
        date: d(date),
        name: name.into(),
    }
}

/// The cached rows of `country` over a range wide enough to hold every test date.
fn cached(p: &Planner, country: &str) -> Vec<PublicHoliday> {
    p.public_holidays_between(country, d("2020-01-01"), d("2030-12-31"))
        .unwrap()
}

fn export(p: &Planner) -> Value {
    serde_json::from_str(&p.export_json(dt(NOW)).unwrap()).unwrap()
}

fn holidays_of(p: &Planner) -> Vec<Holiday> {
    p.active_term().unwrap().unwrap().holidays
}

/// R77: setting a country replaces only that country's cached rows; other countries stay.
#[test]
fn t77_set_public_holidays_replaces_one_country() {
    let mut p = Planner::open_in_memory().unwrap();
    p.set_public_holidays(
        "BD",
        &[
            ph("2026-02-21", "Language Martyrs' Day"),
            ph("2026-03-26", "Independence Day"),
            ph("2026-12-16", "Victory Day"),
        ],
    )
    .unwrap();
    p.set_public_holidays("IN", &[ph("2026-01-26", "Republic Day")])
        .unwrap();

    let second = [
        ph("2026-04-14", "Pohela Boishakh"),
        ph("2026-05-01", "May Day"),
    ];
    p.set_public_holidays("BD", &second).unwrap();

    assert_eq!(cached(&p, "BD"), second);
    assert_eq!(cached(&p, "IN"), [ph("2026-01-26", "Republic Day")]);
}

/// R77, R79: the range is inclusive on both ends, rows come back ordered by date then name,
/// and another country never appears.
#[test]
fn t77_between_is_inclusive_and_ordered() {
    let mut p = Planner::open_in_memory().unwrap();
    p.set_public_holidays(
        "BD",
        &[
            ph("2026-03-10", "Zeta"),
            ph("2026-02-01", "Before the range"),
            ph("2026-03-10", "Alpha"),
            ph("2026-03-31", "Last day"),
            ph("2026-03-01", "First day"),
            ph("2026-04-01", "After the range"),
        ],
    )
    .unwrap();
    p.set_public_holidays("IN", &[ph("2026-03-15", "Other country")])
        .unwrap();

    let got = p
        .public_holidays_between("BD", d("2026-03-01"), d("2026-03-31"))
        .unwrap();

    assert_eq!(
        got,
        [
            ph("2026-03-01", "First day"),
            ph("2026-03-10", "Alpha"),
            ph("2026-03-10", "Zeta"),
            ph("2026-03-31", "Last day"),
        ]
    );
    assert!(p
        .public_holidays_between("US", d("2026-03-01"), d("2026-03-31"))
        .unwrap()
        .is_empty());
}

/// R77: an empty list clears the country's rows and leaves other countries alone.
#[test]
fn t77_empty_set_clears_the_country() {
    let mut p = Planner::open_in_memory().unwrap();
    p.set_public_holidays("BD", &[ph("2026-03-26", "Independence Day")])
        .unwrap();
    p.set_public_holidays("IN", &[ph("2026-01-26", "Republic Day")])
        .unwrap();

    p.set_public_holidays("BD", &[]).unwrap();

    assert!(cached(&p, "BD").is_empty());
    assert_eq!(cached(&p, "IN"), [ph("2026-01-26", "Republic Day")]);
}

/// R77, D18: a repeated `(date, name)` does not fail and is stored once.
#[test]
fn t77_duplicate_date_and_name_does_not_fail() {
    let mut p = Planner::open_in_memory().unwrap();

    p.set_public_holidays(
        "BD",
        &[
            ph("2026-03-26", "Independence Day"),
            ph("2026-03-26", "Independence Day"),
            ph("2026-03-26", "Another name"),
        ],
    )
    .unwrap();

    assert_eq!(
        cached(&p, "BD"),
        [
            ph("2026-03-26", "Another name"),
            ph("2026-03-26", "Independence Day")
        ]
    );
}

/// R80: the cache is not in a backup: no `public_holiday` key and none of the cached names.
#[test]
fn t77_backup_has_no_public_holidays() {
    let (mut p, _, _) = fixture();
    p.set_public_holidays(
        "BD",
        &[
            ph("2026-01-14", "Zorblax Festival"),
            ph("2026-01-20", "Quuxmas"),
        ],
    )
    .unwrap();

    let text = p.export_json(dt(NOW)).unwrap();
    let doc: Value = serde_json::from_str(&text).unwrap();

    assert!(!doc.as_object().unwrap().contains_key("public_holiday"));
    assert!(!text.contains("Zorblax Festival"));
    assert!(!text.contains("Quuxmas"));
}

/// R76, R80: the country setting is in the backup `settings` map and survives a restore into a
/// fresh planner; an unset country stays unset (`None`).
#[test]
fn t77_backup_keeps_country_setting() {
    let (mut source, _, _) = fixture();
    assert_eq!(source.public_holiday_country().unwrap(), None);
    assert!(export(&source)["settings"]
        .get("public_holiday_country")
        .is_none());

    source.set_public_holiday_country("BD").unwrap();
    let text = source.export_json(dt(NOW)).unwrap();
    let doc: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(doc["settings"]["public_holiday_country"], json!("BD"));

    let mut fresh = Planner::open_in_memory().unwrap();
    fresh.restore_json(&text).unwrap();
    assert_eq!(
        fresh.public_holiday_country().unwrap().as_deref(),
        Some("BD")
    );
}

/// R80: a restore does not touch the cached rows; the country setting follows the backup.
#[test]
fn t77_restore_leaves_cached_rows_alone() {
    let (mut p, _, _) = fixture();
    p.set_public_holiday_country("US").unwrap();
    let before_cache = p.export_json(dt(NOW)).unwrap();

    p.set_public_holiday_country("BD").unwrap();
    let rows = [ph("2026-01-14", "Day A"), ph("2026-01-20", "Day B")];
    p.set_public_holidays("BD", &rows).unwrap();
    p.set_public_holidays("IN", &[ph("2026-01-26", "Republic Day")])
        .unwrap();

    p.restore_json(&before_cache).unwrap();

    assert_eq!(cached(&p, "BD"), rows);
    assert_eq!(cached(&p, "IN"), [ph("2026-01-26", "Republic Day")]);
    assert_eq!(p.public_holiday_country().unwrap().as_deref(), Some("US"));
}

/// R80, D24: a backup of an older schema version (5, before the cache table) still restores.
#[test]
fn t77_older_backup_still_restores() {
    let (source, _, _) = fixture();
    let mut doc = export(&source);
    doc["schema_version"] = json!(5);

    let mut fresh = Planner::open_in_memory().unwrap();
    fresh.restore_json(&doc.to_string()).unwrap();

    assert!(fresh.active_term().unwrap().is_some());
    assert!(cached(&fresh, "BD").is_empty());
}

/// R76: the country setting reads `None` when absent and returns the stored text unchanged.
#[test]
fn t77_country_setting_roundtrip() {
    let mut p = Planner::open_in_memory().unwrap();
    assert_eq!(p.public_holiday_country().unwrap(), None);

    p.set_public_holiday_country("off").unwrap();
    assert_eq!(p.public_holiday_country().unwrap().as_deref(), Some("off"));

    p.set_public_holiday_country("BD").unwrap();
    assert_eq!(p.public_holiday_country().unwrap().as_deref(), Some("BD"));
}

/// R5, R6, R71, R79: cached public holidays alone change no session. Once the day is stored as
/// a holiday labelled with the public name (what "Add selected" does) and synced, the pending
/// session on it is gone; a day that was already a day off keeps its kind.
#[test]
fn t78_added_public_holiday_is_a_holiday_and_sync_deletes_sessions() {
    let (mut p, term, cse, _, _) = synced_fixture();
    let all_six = [
        "2026-01-05",
        "2026-01-07",
        "2026-01-12",
        "2026-01-14",
        "2026-01-19",
        "2026-01-21",
    ];
    assert_eq!(dates(&sessions_of(&p, cse.id)), all_six);

    // Public holidays only fill the cache; they never skip classes (R79).
    p.set_public_holidays(
        "BD",
        &[ph("2026-01-12", "Day A"), ph("2026-01-19", "Day B")],
    )
    .unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();
    assert_eq!(dates(&sessions_of(&p, cse.id)), all_six);
    assert!(holidays_of(&p).is_empty());

    // 01-12 is a mid exam day already; only 01-19 is added, labelled with the public name.
    let midterm = Holiday {
        date: d("2026-01-12"),
        label: None,
        kind: DayOffKind::Midterm,
    };
    let added = Holiday {
        date: d("2026-01-19"),
        label: Some("Day B".into()),
        kind: DayOffKind::Holiday,
    };
    p.set_holidays(term.id, &[midterm.clone(), added.clone()])
        .unwrap();
    p.sync_sessions(d("2026-01-05")).unwrap();

    assert_eq!(
        dates(&sessions_of(&p, cse.id)),
        ["2026-01-05", "2026-01-07", "2026-01-14", "2026-01-21"]
    );
    assert_eq!(holidays_of(&p), [midterm, added]);
    assert_eq!(cached(&p, "BD").len(), 2);
}
