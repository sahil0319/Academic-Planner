//! R22 pre-restore copy: before a restore replaces all data, the current database is copied to
//! `{data_dir}/backups/pre-restore-{YYYYMMDD-HHMM}.sqlite` (research R-D3, R-D4;
//! contracts/ui-bridge.md "Restore flow").
#![allow(dead_code)] // removed in T026 once the restore flow uses every item

use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;

/// R22: copies the database at `db_path` into the `backups` folder next to it, named for `now`
/// (with a `-2`, `-3`, … suffix when that name is taken), and returns the copy's path. The restore
/// must not be attempted unless this returns `Ok`.
pub fn pre_restore_copy(db_path: &Path, now: NaiveDateTime) -> anyhow::Result<PathBuf> {
    let _ = (db_path, now);
    todo!("T021")
}
