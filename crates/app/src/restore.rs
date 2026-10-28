//! R22 pre-restore copy: before a restore replaces all data, the current database is copied to
//! `{data_dir}/backups/pre-restore-{YYYYMMDD-HHMM}.sqlite` (research R-D3, R-D4;
//! contracts/ui-bridge.md "Restore flow").
#![allow(dead_code)] // removed in T026 once the restore flow uses every item

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Context;
use chrono::NaiveDateTime;
use rusqlite::{Connection, OpenFlags};

use crate::viewmodel;

/// R22: copies the database at `db_path` into the `backups` folder next to it, named for `now`
/// (with a `-2`, `-3`, … suffix when that name is taken, research R-D4), and returns the copy's
/// path. The restore must not be attempted unless this returns `Ok`.
///
/// The copy is made with `VACUUM INTO` on a second connection, so commits still in the WAL file
/// are included and the result is one self-contained file (research R-D3).
pub fn pre_restore_copy(db_path: &Path, now: NaiveDateTime) -> anyhow::Result<PathBuf> {
    let data_dir = db_path
        .parent()
        .context("the database path has no parent folder")?;
    let backups = data_dir.join("backups");
    fs::create_dir_all(&backups)
        .with_context(|| format!("could not create {}", backups.display()))?;

    let existing = existing_names(&backups)?;
    let target = backups.join(viewmodel::free_pre_restore_name(now, &existing));
    let target_text = target
        .to_str()
        .with_context(|| format!("{} is not valid UTF-8", target.display()))?;

    // Read-write without CREATE: a missing database is an error, never a new empty file.
    let conn = Connection::open_with_flags(
        db_path,
        OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| format!("could not open {}", db_path.display()))?;
    conn.execute("VACUUM INTO ?1", [target_text])
        .with_context(|| format!("could not write {}", target.display()))?;
    Ok(target)
}

/// File names already in the backups folder (R-D4: input to the free-name choice).
fn existing_names(backups: &Path) -> anyhow::Result<Vec<String>> {
    let entries =
        fs::read_dir(backups).with_context(|| format!("could not list {}", backups.display()))?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.with_context(|| format!("could not list {}", backups.display()))?;
        if let Ok(name) = entry.file_name().into_string() {
            names.push(name);
        }
    }
    Ok(names)
}
