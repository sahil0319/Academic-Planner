//! Reminders (M5): due reminders logged in `reminder_log`.

use chrono::NaiveDateTime;

use crate::db::Planner;
use crate::error::Result;
use crate::model::Reminder;

impl Planner {
    /// §5, FR-001: reminders due at `now`, logged in `reminder_log` so none is returned twice.
    pub fn due_reminders(&mut self, now: NaiveDateTime) -> Result<Vec<Reminder>> {
        let _ = now;
        todo!("M5: reminders")
    }
}
