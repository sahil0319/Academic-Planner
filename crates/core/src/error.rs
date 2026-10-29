//! Error type returned by every core operation.

use std::path::{Path, PathBuf};

use thiserror::Error;

/// Every failure a core operation can report. User input never causes a panic.
#[derive(Debug, Error)]
pub enum Error {
    /// R2: the input breaks a validation rule; the message says which.
    #[error("invalid input: {0}")]
    Validation(String),

    /// The row addressed by `id` does not exist.
    #[error("{entity} {id} not found")]
    NotFound { entity: &'static str, id: i64 },

    /// R1: the operation needs an active term and there is none.
    #[error("no active term; save a term first")]
    NoActiveTerm,

    /// R25: the database was written by a newer app version and is left untouched.
    #[error(
        "database schema version {found} is newer than this app supports ({supported}); update the app"
    )]
    SchemaTooNew { found: i64, supported: i64 },

    /// R26: a migration failed; the database stays at version `at`.
    #[error(
        "upgrading the database to schema version {version} failed; it was left at version {at}{}",
        backup_sentence(.backup.as_deref())
    )]
    MigrationFailed {
        version: i64,
        at: i64,
        backup: Option<PathBuf>,
        #[source]
        source: rusqlite::Error,
    },

    /// R24: no `ACADEMIC_PLANNER_DB` override and the OS reports no user data directory.
    #[error("no user data directory found; set ACADEMIC_PLANNER_DB to choose a database path")]
    NoDataDir,

    /// A stored value could not be parsed (for example a malformed date).
    #[error("corrupt stored value: {0}")]
    Corrupt(String),

    /// Any other SQLite error.
    #[error("database error: {0}")]
    Db(#[source] rusqlite::Error),

    /// File system error, e.g. while creating the data directory.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<rusqlite::Error> for Error {
    fn from(e: rusqlite::Error) -> Self {
        match e {
            // Raised by the column decoders in `db.rs` when stored text does not parse.
            rusqlite::Error::FromSqlConversionFailure(_, _, inner) => {
                Error::Corrupt(inner.to_string())
            }
            other => Error::Db(other),
        }
    }
}

/// R26: the optional last sentence of `MigrationFailed`, naming the pre-migration copy.
fn backup_sentence(backup: Option<&Path>) -> String {
    backup
        .map(|path| format!(". A copy from before the upgrade is at {}", path.display()))
        .unwrap_or_default()
}

/// Result alias used by every core operation.
pub type Result<T> = std::result::Result<T, Error>;
