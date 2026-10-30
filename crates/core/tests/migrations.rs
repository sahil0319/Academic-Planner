//! R25: opening a database migrates it, reopening is a no-op, newer schemas are refused.
//!
//! Reopen tests use a shared-cache in-memory database: a "keeper" connection holds it open
//! for the whole test so `Planner::open` can connect to it more than once. No file is created.

use std::path::Path;

use planner_core::{Error, Planner, SCHEMA_VERSION};
use rusqlite::Connection;

const TABLES: [&str; 12] = [
    "assessment",
    "attendance_band",
    "course",
    "grade_scale",
    "holiday",
    "reminder_log",
    "rule_component",
    "rule_set",
    "session",
    "setting",
    "slot",
    "term",
];

fn keeper(uri: &str) -> Connection {
    Connection::open(uri).unwrap()
}

fn user_version(conn: &Connection) -> i64 {
    conn.query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap()
}

/// Every schema object as `(type, name, sql)`, sorted.
fn schema(conn: &Connection) -> Vec<(String, String, Option<String>)> {
    let mut stmt = conn
        .prepare("SELECT type, name, sql FROM sqlite_master ORDER BY type, name")
        .unwrap();
    stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
}

fn settings_rows(conn: &Connection) -> Vec<(String, String)> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM setting ORDER BY key")
        .unwrap();
    stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .map(|r| r.unwrap())
        .collect()
}

#[test]
fn t20_fresh_db_migrated_and_reopen_is_noop() {
    let uri = "file:t20_reopen?mode=memory&cache=shared";
    let keep = keeper(uri);
    assert_eq!(user_version(&keep), 0);

    let planner = Planner::open(Path::new(uri)).unwrap();
    assert_eq!(user_version(&keep), SCHEMA_VERSION);
    assert_eq!(SCHEMA_VERSION, 3);
    let tables: Vec<String> = schema(&keep)
        .into_iter()
        .filter(|(kind, _, _)| kind == "table")
        .map(|(_, name, _)| name)
        .collect();
    assert_eq!(tables, TABLES);
    drop(planner);

    let schema_before = schema(&keep);
    let settings_before = settings_rows(&keep);
    assert_eq!(settings_before.len(), 3);

    let reopened = Planner::open(Path::new(uri)).unwrap();
    assert_eq!(user_version(&keep), SCHEMA_VERSION);
    assert_eq!(schema(&keep), schema_before);
    assert_eq!(settings_rows(&keep), settings_before);
    drop(reopened);
}

#[test]
fn t20_open_in_memory_migrates_each_fresh_db() {
    // Each in-memory planner is its own fresh database; both must migrate cleanly.
    assert!(Planner::open_in_memory().is_ok());
    assert!(Planner::open_in_memory().is_ok());
}

#[test]
fn t26_newer_schema_version_refused() {
    let uri = "file:t26_newer?mode=memory&cache=shared";
    let keep = keeper(uri);
    keep.pragma_update(None, "user_version", SCHEMA_VERSION + 1)
        .unwrap();

    let Err(err) = Planner::open(Path::new(uri)) else {
        panic!("opening a newer database must fail");
    };
    assert!(
        matches!(
            err,
            Error::SchemaTooNew {
                found: 4,
                supported: 3
            }
        ),
        "unexpected error: {err:?}"
    );
    let message = err.to_string();
    assert!(message.contains("newer"), "unclear message: {message}");

    // The database is untouched: same version, no schema created.
    assert_eq!(user_version(&keep), SCHEMA_VERSION + 1);
    assert!(schema(&keep).is_empty());
}

/// The names in `dir/backups`, or none when the folder is absent.
fn backup_names(dir: &Path) -> Vec<String> {
    match std::fs::read_dir(dir.join("backups")) {
        Ok(entries) => entries
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// R26, T31: opening a new database file with the real migration list writes no backup.
#[test]
fn t31_fresh_file_db_makes_no_backup() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("planner.sqlite");

    let planner = Planner::open(&path).unwrap();
    drop(planner);

    assert!(path.exists());
    assert!(backup_names(dir.path()).is_empty());
}

/// R26, T31: reopening a file database that is already current writes no backup.
#[test]
fn t31_current_file_db_reopen_makes_no_backup() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("planner.sqlite");

    drop(Planner::open(&path).unwrap());
    drop(Planner::open(&path).unwrap());

    assert!(backup_names(dir.path()).is_empty());
}
