//! Schema migrations (R25) and the pre-migration copy of an existing database (R26).

use std::path::Path;

use rusqlite::Connection;

use crate::error::{Error, Result};

/// R25, R26: applies `migrations[found..]` to `conn`, where `found` is the stored
/// `user_version`; entry `i` upgrades the schema from version `i` to `i + 1`.
///
/// R25: a database newer than `migrations.len()` is refused before anything is written,
/// including the WAL switch (switching to WAL rewrites the file header). Each migration runs
/// in its own transaction together with its `user_version` bump.
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
    // R26: the pre-migration copy of a file database is written here (M7, T014).
    let _ = db_path;
    for (version, sql) in (1_i64..).zip(migrations).skip(found as usize) {
        let tx = conn.transaction()?;
        tx.execute_batch(sql)?;
        tx.pragma_update(None, "user_version", version)?;
        tx.commit()?;
    }
    Ok(())
}

/// R26: the file name of the pre-migration copy, `pre-migrate-v{from}-to-v{to}.sqlite`, or
/// the first free `-2`, `-3`, … suffix among `existing`. Pure; no clock.
#[allow(dead_code)]
fn backup_file_name(from: i64, to: i64, existing: &[String]) -> String {
    let _ = (from, to, existing);
    todo!("M7: R26")
}

#[cfg(test)]
mod tests;
