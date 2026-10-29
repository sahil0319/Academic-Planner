//! R25, R26, T31, T32: the pre-migration copy, atomic migrations, and the copy's file name.
//! Contract: specs/007-m7-updater/contracts/core-migrations.md. Every test uses its own
//! temporary folder; nothing touches a real user database.

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::types::Value;
use tempfile::TempDir;

use super::*;

const V1: &str = include_str!("../migrations/001_init.sql");
const TEST_V2: &str = "CREATE TABLE t31_probe (x INTEGER);";
const FAILING_V2: &str = "CREATE TABLE t32_partial (x); CREATE TABLE term (x);";
const COPY_NAME: &str = "pre-migrate-v1-to-v2.sqlite";

/// Everything observable in a database file: `user_version`, every `sqlite_master` row, and
/// every table's rows ordered by rowid.
#[derive(Debug, PartialEq)]
struct Snapshot {
    user_version: i64,
    schema: Vec<(String, String, String, Option<String>)>,
    tables: Vec<(String, Vec<Vec<String>>)>,
}

fn snapshot(path: &Path) -> Snapshot {
    let conn = Connection::open(path).unwrap();
    let user_version = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .unwrap();
    let mut stmt = conn
        .prepare("SELECT type, name, tbl_name, sql FROM sqlite_master ORDER BY type, name")
        .unwrap();
    let schema: Vec<(String, String, String, Option<String>)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
        .unwrap()
        .map(|r| r.unwrap())
        .collect();
    let names: Vec<String> = schema
        .iter()
        .filter(|(kind, name, _, _)| kind == "table" && !name.starts_with("sqlite_"))
        .map(|(_, name, _, _)| name.clone())
        .collect();
    let tables = names
        .into_iter()
        .map(|name| {
            let mut stmt = conn
                .prepare(&format!("SELECT * FROM \"{name}\" ORDER BY rowid"))
                .unwrap();
            let columns = stmt.column_count();
            let rows: Vec<Vec<String>> = stmt
                .query_map([], |r| {
                    (0..columns)
                        .map(|i| r.get::<_, Value>(i).map(|v| format!("{v:?}")))
                        .collect()
                })
                .unwrap()
                .map(|r| r.unwrap())
                .collect();
            (name, rows)
        })
        .collect();
    Snapshot {
        user_version,
        schema,
        tables,
    }
}

/// Opens `path` and runs `migrations` on it, as `Planner::open` does.
fn run_on(path: &Path, migrations: &[&str]) -> Result<()> {
    let mut conn = Connection::open(path).unwrap();
    run(&mut conn, Some(path), migrations)
}

/// A version 1 database file with a term, a course, and an extra setting.
fn make_v1(dir: &TempDir) -> PathBuf {
    let path = dir.path().join("planner.sqlite");
    run_on(&path, &[V1]).unwrap();
    let conn = Connection::open(&path).unwrap();
    conn.execute_batch(
        "INSERT INTO term (id, name, start_date, end_date, is_active)
           VALUES (1, 'Spring', '2026-01-12', '2026-05-01', 1);
         INSERT INTO course (id, term_id, code, name) VALUES (1, 1, 'CSE101', 'Intro');
         INSERT INTO setting (key, value) VALUES ('theme', 'dark');",
    )
    .unwrap();
    path
}

fn backup_dir(path: &Path) -> PathBuf {
    path.parent().unwrap().join("backups")
}

/// The names in the `backups` folder next to `path`, sorted; empty when there is no folder.
fn backup_names(path: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(backup_dir(path)) else {
        return Vec::new();
    };
    let mut names: Vec<String> = entries
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn user_version(path: &Path) -> i64 {
    snapshot(path).user_version
}

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

/// R26, T31: upgrading an existing database first writes `backups/pre-migrate-v1-to-v2.sqlite`,
/// a copy equal to the old database, then migrates it to version 2.
#[test]
fn t31_upgrade_writes_pre_migrate_copy_equal_to_old_db() {
    let dir = TempDir::new().unwrap();
    let path = make_v1(&dir);
    let old = snapshot(&path);

    run_on(&path, &[V1, TEST_V2]).unwrap();

    let copy = backup_dir(&path).join(COPY_NAME);
    assert!(copy.exists(), "missing {}", copy.display());
    assert_eq!(snapshot(&copy), old);
    assert_eq!(backup_names(&path), [COPY_NAME]);
    assert_eq!(user_version(&path), 2);
}

/// R26, T31: reopening a database that is already current writes no new copy.
#[test]
fn t31_reopen_current_db_makes_no_new_copy() {
    let dir = TempDir::new().unwrap();
    let path = make_v1(&dir);
    run_on(&path, &[V1, TEST_V2]).unwrap();
    assert_eq!(backup_names(&path), [COPY_NAME]);

    run_on(&path, &[V1, TEST_V2]).unwrap();

    assert_eq!(backup_names(&path), [COPY_NAME]);
    assert_eq!(user_version(&path), 2);
}

/// R26, T31: a new database file is migrated without any copy (nothing to preserve).
#[test]
fn t31_fresh_db_makes_no_copy() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("planner.sqlite");

    run_on(&path, &[V1, TEST_V2]).unwrap();

    assert!(backup_names(&path).is_empty());
    assert_eq!(user_version(&path), 2);
}

/// R26, T31: when the copy's name is taken the copy gets the `-2` suffix and the existing
/// file is left unchanged.
#[test]
fn t31_existing_copy_name_gets_suffix() {
    let dir = TempDir::new().unwrap();
    let path = make_v1(&dir);
    let old = snapshot(&path);
    fs::create_dir_all(backup_dir(&path)).unwrap();
    let taken = backup_dir(&path).join(COPY_NAME);
    fs::write(&taken, b"earlier copy").unwrap();

    run_on(&path, &[V1, TEST_V2]).unwrap();

    assert_eq!(fs::read(&taken).unwrap(), b"earlier copy");
    let suffixed = backup_dir(&path).join("pre-migrate-v1-to-v2-2.sqlite");
    assert!(suffixed.exists(), "missing {}", suffixed.display());
    assert_eq!(snapshot(&suffixed), old);
    assert_eq!(user_version(&path), 2);
}

/// R25, R26, T32: a failing second migration rolls back whole, leaves the database at
/// version 1 unchanged, and the error names the copy made before the attempt.
#[test]
fn t32_failed_second_migration_leaves_db_at_v1() {
    let dir = TempDir::new().unwrap();
    let path = make_v1(&dir);
    let before = snapshot(&path);

    let err = run_on(&path, &[V1, FAILING_V2]).expect_err("the second migration must fail");

    let Error::MigrationFailed {
        version,
        at,
        backup: Some(copy),
        ..
    } = &err
    else {
        panic!("expected MigrationFailed with a backup path, got {err:?}");
    };
    assert_eq!((*version, *at), (2, 1));
    assert!(copy.exists(), "missing {}", copy.display());
    assert_eq!(snapshot(copy), before);
    assert!(
        err.to_string().contains(&copy.display().to_string()),
        "message does not name the copy: {err}"
    );
    let after = snapshot(&path);
    assert_eq!(after.user_version, 1);
    assert!(after
        .schema
        .iter()
        .all(|(_, name, _, _)| name != "t32_partial"));
    assert_eq!(after, before);
}

/// R26: the copy is named `pre-migrate-v{from}-to-v{to}.sqlite` when that name is free.
#[test]
fn backup_file_name_base_is_free() {
    assert_eq!(backup_file_name(1, 2, &[]), COPY_NAME);
}

/// R26: when the base name is taken the first free suffix is `-2`.
#[test]
fn backup_file_name_base_taken_gets_suffix_2() {
    assert_eq!(
        backup_file_name(1, 2, &names(&[COPY_NAME])),
        "pre-migrate-v1-to-v2-2.sqlite"
    );
}

/// R26: when the base name and `-2` are taken the next free suffix is `-3`.
#[test]
fn backup_file_name_base_and_2_taken_gets_suffix_3() {
    assert_eq!(
        backup_file_name(1, 2, &names(&[COPY_NAME, "pre-migrate-v1-to-v2-2.sqlite"])),
        "pre-migrate-v1-to-v2-3.sqlite"
    );
}

/// R26: names that are not the base or one of its suffixes do not count as taken.
#[test]
fn backup_file_name_ignores_unrelated_names() {
    let existing = names(&[
        "notes.txt",
        "pre-migrate-v2-to-v3.sqlite",
        "pre-migrate-v1-to-v2.sqlite.bak",
        "planner-2026-01-25.json",
    ]);
    assert_eq!(backup_file_name(1, 2, &existing), COPY_NAME);
}
