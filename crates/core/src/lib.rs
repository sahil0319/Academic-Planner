//! Academic Planner core: domain model, SQLite storage, scheduling, attendance,
//! backup, and export. No UI, no system clock, no file dialogs.

mod attendance;
mod backup;
mod dashboard;
mod db;
mod error;
mod export;
mod grading;
mod import;
mod marks;
mod migrate;
mod model;
mod reminders;
mod schedule;
mod terms;

pub use db::{default_db_path, resolve_db_path, Planner, DB_ENV_VAR, DB_FILE_NAME, SCHEMA_VERSION};
pub use error::{Error, Result};
pub use import::*;
pub use model::*;
