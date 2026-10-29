//! Setting `check_updates_on_start` (§3, §13.1 UI, R29): storage, default, backup export, and
//! restore validation. Contract: specs/007-m7-updater/contracts/core-settings.md.

mod common;

use common::*;
use planner_core::{Error, Planner, Settings, ThemeMode};
use serde_json::{json, Value};

const NOW: &str = "2026-01-25T18:30";

fn parse(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn settings_off() -> Settings {
    Settings {
        class_reminder_minutes: 30,
        upcoming_window_days: 14,
        notifications_enabled: false,
        theme: ThemeMode::Dark,
        check_updates_on_start: false,
    }
}

/// A backup of `p` whose `settings` object is edited by `edit`.
fn backup_with_settings(
    p: &Planner,
    edit: impl FnOnce(&mut serde_json::Map<String, Value>),
) -> String {
    let mut doc = parse(&p.export_json(dt(NOW)).unwrap());
    edit(doc["settings"].as_object_mut().unwrap());
    serde_json::to_string(&doc).unwrap()
}

/// §3, §13.1 UI, R29: a fresh database reads the startup check as on.
#[test]
fn check_updates_fresh_db_is_on() {
    let p = Planner::open_in_memory().unwrap();
    assert!(p.settings().unwrap().check_updates_on_start);
}

/// §3, §13.1 UI, R29: a saved "off" reads back, and every other setting is kept.
#[test]
fn check_updates_saved_off_reads_back_and_keeps_other_settings() {
    let mut p = Planner::open_in_memory().unwrap();
    let s = settings_off();
    p.save_settings(&s).unwrap();
    let read = p.settings().unwrap();
    assert!(!read.check_updates_on_start);
    assert_eq!(read, s);
}

/// §3, §13.1 UI, R21: the JSON export holds `settings.check_updates_on_start` as `"0"` when off.
#[test]
fn check_updates_export_contains_key() {
    let mut p = Planner::open_in_memory().unwrap();
    p.save_settings(&settings_off()).unwrap();
    let doc = parse(&p.export_json(dt(NOW)).unwrap());
    assert_eq!(doc["settings"]["check_updates_on_start"], json!("0"));
}

/// §3, §13.1 UI, R22: a backup without the key restores fine and the setting reads as on.
#[test]
fn check_updates_backup_without_key_restores_as_on() {
    let mut p = Planner::open_in_memory().unwrap();
    p.save_settings(&settings_off()).unwrap();
    let backup = backup_with_settings(&p, |settings| {
        settings.remove("check_updates_on_start");
    });
    p.restore_json(&backup).unwrap();
    assert!(p.settings().unwrap().check_updates_on_start);
}

/// §3, §13.1 UI, R22: a value other than "0" or "1" is rejected with `Error::Validation` and
/// nothing changes (all-or-nothing restore).
#[test]
fn check_updates_restore_rejects_bad_value() {
    for bad in ["yes", "true"] {
        let mut p = Planner::open_in_memory().unwrap();
        p.save_settings(&settings_off()).unwrap();
        let before = p.export_json(dt(NOW)).unwrap();
        let backup = backup_with_settings(&p, |settings| {
            settings.insert("check_updates_on_start".into(), json!(bad));
        });
        let err = p
            .restore_json(&backup)
            .expect_err(&format!("value {bad:?} should be rejected"));
        assert!(
            matches!(err, Error::Validation(_)),
            "value {bad:?}: expected Error::Validation, got {err:?}"
        );
        assert_eq!(p.export_json(dt(NOW)).unwrap(), before, "value {bad:?}");
        assert_eq!(p.settings().unwrap(), settings_off());
    }
}
