//! Appearance setting `theme` (§3, §5 `Settings`, human decision 2026-10-29): storage,
//! default, backup export, and restore validation.

mod common;

use common::*;
use planner_core::{Error, Planner, Settings, ThemeMode};
use serde_json::{json, Value};

const NOW: &str = "2026-01-25T18:30";

fn parse(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

/// A backup of a fresh planner whose `settings` object is edited by `edit`.
fn backup_with_settings(edit: impl FnOnce(&mut serde_json::Map<String, Value>)) -> String {
    let p = Planner::open_in_memory().unwrap();
    let mut doc = parse(&p.export_json(dt(NOW)).unwrap());
    edit(doc["settings"].as_object_mut().unwrap());
    serde_json::to_string(&doc).unwrap()
}

fn save_theme(p: &mut Planner, theme: ThemeMode) {
    p.save_settings(&Settings {
        theme,
        ..Settings::default()
    })
    .unwrap();
}

/// §5 Settings theme, human decision 2026-10-29: a fresh database reads `ThemeMode::System`.
#[test]
fn theme_fresh_db_theme_is_system() {
    let p = Planner::open_in_memory().unwrap();
    assert_eq!(p.settings().unwrap().theme, ThemeMode::System);
}

/// §5 Settings theme, human decision 2026-10-29: a saved Dark theme is read back as Dark.
#[test]
fn theme_saved_dark_reads_back() {
    let mut p = Planner::open_in_memory().unwrap();
    save_theme(&mut p, ThemeMode::Dark);
    assert_eq!(p.settings().unwrap().theme, ThemeMode::Dark);
}

/// §5 Settings theme, human decision 2026-10-29: a saved Light theme is read back as Light,
/// and saving System afterwards reads back as System.
#[test]
fn theme_saved_light_then_system_reads_back() {
    let mut p = Planner::open_in_memory().unwrap();
    save_theme(&mut p, ThemeMode::Light);
    assert_eq!(p.settings().unwrap().theme, ThemeMode::Light);
    save_theme(&mut p, ThemeMode::System);
    assert_eq!(p.settings().unwrap().theme, ThemeMode::System);
}

/// §5 Settings theme, human decision 2026-10-29: changing theme leaves the other settings alone.
#[test]
fn theme_theme_does_not_change_other_settings() {
    let mut p = Planner::open_in_memory().unwrap();
    let s = Settings {
        class_reminder_minutes: 30,
        upcoming_window_days: 14,
        notifications_enabled: false,
        theme: ThemeMode::Dark,
    };
    p.save_settings(&s).unwrap();
    assert_eq!(p.settings().unwrap(), s);
}

/// §5 Settings theme, human decision 2026-10-29, R21: the JSON export carries `"theme":"dark"`.
#[test]
fn theme_export_contains_theme_dark() {
    let mut p = Planner::open_in_memory().unwrap();
    save_theme(&mut p, ThemeMode::Dark);
    let doc = parse(&p.export_json(dt(NOW)).unwrap());
    assert_eq!(doc["settings"]["theme"], json!("dark"));
}

/// §5 Settings theme, human decision 2026-10-29, R22: restore applies the backup's theme.
#[test]
fn theme_restore_applies_theme() {
    let mut source = Planner::open_in_memory().unwrap();
    save_theme(&mut source, ThemeMode::Light);
    let backup = source.export_json(dt(NOW)).unwrap();

    let mut target = Planner::open_in_memory().unwrap();
    save_theme(&mut target, ThemeMode::Dark);
    target.restore_json(&backup).unwrap();
    assert_eq!(target.settings().unwrap().theme, ThemeMode::Light);
}

/// §5 Settings theme, human decision 2026-10-29, R22: a backup without a `theme` key restores
/// fine and the theme reads System (a missing row reads as system).
#[test]
fn theme_restore_without_theme_key_reads_system() {
    let mut p = Planner::open_in_memory().unwrap();
    save_theme(&mut p, ThemeMode::Dark);
    let backup = backup_with_settings(|settings| {
        settings.remove("theme");
    });
    p.restore_json(&backup).unwrap();
    assert_eq!(p.settings().unwrap().theme, ThemeMode::System);
}

/// §5 Settings theme, human decision 2026-10-29, R22: a `theme` value other than
/// system|light|dark is rejected with `Error::Validation` and nothing changes (all-or-nothing).
#[test]
fn theme_restore_rejects_unknown_theme() {
    for bad in ["purple", "", "Dark", "SYSTEM"] {
        let mut p = Planner::open_in_memory().unwrap();
        save_theme(&mut p, ThemeMode::Dark);
        let before = p.export_json(dt(NOW)).unwrap();
        let backup = backup_with_settings(|settings| {
            settings.insert("theme".into(), json!(bad));
        });
        let err = p
            .restore_json(&backup)
            .expect_err(&format!("theme {bad:?} should be rejected"));
        assert!(
            matches!(err, Error::Validation(_)),
            "theme {bad:?}: expected Error::Validation, got {err:?}"
        );
        assert_eq!(p.export_json(dt(NOW)).unwrap(), before, "theme {bad:?}");
        assert_eq!(p.settings().unwrap().theme, ThemeMode::Dark);
    }
}
