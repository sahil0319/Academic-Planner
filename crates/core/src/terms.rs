//! Terms (M14): archiving a finished term and listing past terms (R68, R70).

use chrono::NaiveDate;
use rusqlite::params;

use crate::attendance;
use crate::db::{courses_of_term, load_term, Planner};
use crate::error::{Error, Result};
use crate::model::PastTermSummary;

impl Planner {
    /// R68: archives (sets `is_active = 0`) the term `id` when `today > end_date`.
    ///
    /// An unknown `id` is `Error::NotFound`; an already inactive term is a no-op. `today` on
    /// or before the end date is `Error::Validation` and the term stays active. Courses,
    /// slots, sessions, assessments, and holidays are untouched (R68 "all data stays", R3).
    pub fn archive_term(&mut self, id: i64, today: NaiveDate) -> Result<()> {
        let term = load_term(&self.conn, id)?;
        if !term.is_active {
            return Ok(());
        }
        if today <= term.end_date {
            return Err(Error::Validation(format!(
                "This term ends on {}; archive it after that",
                term.end_date.format("%B %-d, %Y")
            )));
        }
        self.conn
            .execute("UPDATE term SET is_active = 0 WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// R70: every inactive term, newest `end_date` first (tie: `id` descending).
    ///
    /// `courses` counts every course of the term, archived ones included (R3).
    /// `attendance_pct` is R13 over the attended and absent sessions of all those courses.
    /// `gpa` is the R40 GPA of that term. The active term (R1) is never listed.
    pub fn past_terms(&self) -> Result<Vec<PastTermSummary>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM term WHERE is_active = 0 ORDER BY end_date DESC, id DESC")?;
        let ids = stmt
            .query_map([], |r| r.get::<_, i64>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        ids.into_iter()
            .map(|id| {
                let term = load_term(&self.conn, id)?;
                let courses = courses_of_term(&self.conn, id, true)?.len() as u32;
                let (attended, absent): (u32, u32) = self.conn.query_row(
                    "SELECT coalesce(sum(s.status = 'attended'), 0),
                            coalesce(sum(s.status = 'absent'), 0)
                     FROM session s JOIN course c ON c.id = s.course_id
                     WHERE c.term_id = ?1",
                    [id],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )?;
                Ok(PastTermSummary {
                    term,
                    courses,
                    attendance_pct: attendance::pct(attended, absent),
                    gpa: self.term_gpa_for(id)?.gpa,
                })
            })
            .collect()
    }
}
