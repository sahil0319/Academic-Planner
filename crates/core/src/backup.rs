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

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::db::Planner;
    use crate::model::TermInput;

    const LOG_ROW: &str =
        "INSERT INTO reminder_log (kind, ref_id, fire_at) VALUES ('session', 1, '2026-01-05T08:45')";

    fn log_rows(p: &Planner) -> Vec<(String, i64, String)> {
        let mut stmt = p
            .conn
            .prepare("SELECT kind, ref_id, fire_at FROM reminder_log ORDER BY kind, ref_id")
            .unwrap();
        let rows = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .collect::<rusqlite::Result<_>>()
            .unwrap();
        rows
    }

    /// R22: restore, whether it succeeds or fails, leaves the reminder log as it was.
    #[test]
    fn r22_restore_leaves_reminder_log_untouched() {
        let day = |d: u32| NaiveDate::from_ymd_opt(2026, 1, d).unwrap();
        let now = day(25).and_hms_opt(18, 30, 0).unwrap();

        let mut source = Planner::open_in_memory().unwrap();
        source
            .save_term(TermInput {
                id: None,
                name: "Winter 2026".into(),
                start_date: day(5),
                end_date: day(25),
            })
            .unwrap();
        let backup = source.export_json(now).unwrap();

        let mut p = Planner::open_in_memory().unwrap();
        p.conn.execute(LOG_ROW, []).unwrap();
        let before = log_rows(&p);
        assert_eq!(before.len(), 1);

        p.restore_json(&backup).unwrap();
        assert_eq!(log_rows(&p), before, "successful restore touched the log");

        assert!(p.restore_json("this is not a backup").is_err());
        assert_eq!(log_rows(&p), before, "failed restore touched the log");
    }
}
