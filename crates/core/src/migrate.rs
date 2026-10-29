//! Schema migrations (R25) and the pre-migration copy of an existing database (R26).

use std::fs;
use std::path::{Path, PathBuf};

use rusqlite::Connection;

use crate::error::{Error, Result};

/// R25, R26: applies `migrations[found..]` to `conn`, where `found` is the stored
/// `user_version`; entry `i` upgrades the schema from version `i` to `i + 1`.
///
/// R25: a database newer than `migrations.len()` is refused before anything is written,
/// including the WAL switch (switching to WAL rewrites the file header). Each migration runs
/// in its own transaction together with its `user_version` bump, so a failure leaves the
/// database at the previous version.
///
/// R26: when an existing file database (`1 <= found < migrations.len()`) is upgraded, a copy
/// is first written to `{parent}/backups/pre-migrate-v{found}-to-v{len}.sqlite`; if the copy
/// cannot be written no migration runs. New and in-memory databases get no copy.
pub(crate) fn run(
    conn: &mut Connection,
    db_path: Option<&Path>,
    migrations: &[&str],
) -> Result<()> {
    let supported = migrations.len() as i64;
    let found: i64 = conn.pragma_query_value(None, "user_version", |r| r.get(0))?;
    if found > supported {
        return Err(Error::SchemaTooNew { found, supported });
    }
    // In-memory databases report "memory"; either answer is fine.
    let _mode: String = conn.pragma_update_and_check(None, "journal_mode", "WAL", |r| r.get(0))?;
    let backup = match db_path {
        Some(path) if (1..supported).contains(&found) => {
            Some(write_copy(conn, path, found, supported)?)
        }
        _ => None,
    };
    for (version, sql) in (1_i64..).zip(migrations).skip(found as usize) {
        apply(conn, version, sql).map_err(|source| Error::MigrationFailed {
            version,
            at: version - 1,
            backup: backup.clone(),
            source,
        })?;
    }
    Ok(())
}

/// R25: runs one migration and its `user_version` bump in a single transaction; on error the
/// transaction is rolled back when it is dropped.
fn apply(conn: &mut Connection, version: i64, sql: &str) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    tx.execute_batch(sql)?;
    tx.pragma_update(None, "user_version", version)?;
    tx.commit()
}

/// R26: writes the pre-migration copy of the database at `db_path` with `VACUUM INTO` and
/// returns its path. Existing files in the `backups` folder are never overwritten.
fn write_copy(conn: &Connection, db_path: &Path, from: i64, to: i64) -> Result<PathBuf> {
    let dir = db_path
        .parent()
        .map_or_else(|| PathBuf::from("backups"), |p| p.join("backups"));
    fs::create_dir_all(&dir)?;
    let existing = fs::read_dir(&dir)?
        .map(|entry| entry.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect::<std::io::Result<Vec<String>>>()?;
    let copy = dir.join(backup_file_name(from, to, &existing));
    let target = copy.to_str().ok_or_else(|| {
        Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("backup path is not valid UTF-8: {}", copy.display()),
        ))
    })?;
    conn.execute("VACUUM INTO ?1", [target])?;
    Ok(copy)
}

/// R26: the file name of the pre-migration copy, `pre-migrate-v{from}-to-v{to}.sqlite`, or
/// the first free `-2`, `-3`, … suffix among `existing`. Pure; no clock.
fn backup_file_name(from: i64, to: i64, existing: &[String]) -> String {
    let stem = format!("pre-migrate-v{from}-to-v{to}");
    let taken = |name: &str| existing.iter().any(|e| e == name);
    let base = format!("{stem}.sqlite");
    if !taken(&base) {
        return base;
    }
    (2_u64..)
        .map(|n| format!("{stem}-{n}.sqlite"))
        .find(|name| !taken(name))
        .expect("an unbounded range always has a free name")
}

#[cfg(test)]
mod tests;
