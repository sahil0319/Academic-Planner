//! JSON backup and restore (M3) and reminders (M5).

use chrono::NaiveDateTime;

use crate::db::Planner;
use crate::error::Result;
use crate::model::Reminder;

impl Planner {
    /// R21: exports all data as a versioned JSON backup; IDs are preserved.
    pub fn export_json(&self, now: NaiveDateTime) -> Result<String> {
        let _ = now;
        todo!("M3: R21")
    }

    /// R22: validates a backup and replaces all data in one transaction; on any error the
    /// database is unchanged.
    pub fn restore_json(&mut self, json: &str) -> Result<()> {
        let _ = json;
        todo!("M3: R22")
    }

    /// Reminders due at `now`, logged in `reminder_log` so none is returned twice.
    pub fn due_reminders(&mut self, now: NaiveDateTime) -> Result<Vec<Reminder>> {
        let _ = now;
        todo!("M5: reminders")
    }
}
