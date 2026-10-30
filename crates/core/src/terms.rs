//! Terms (M14): archiving a finished term and listing past terms (R68, R70).

use chrono::NaiveDate;

use crate::db::Planner;
use crate::error::Result;
use crate::model::PastTermSummary;

impl Planner {
    /// R68: archives (sets `is_active = 0`) the term `id` when `today > end_date`.
    pub fn archive_term(&mut self, _id: i64, _today: NaiveDate) -> Result<()> {
        todo!("M14 T004: archive_term")
    }

    /// R70: every inactive term, newest `end_date` first (tie: `id` descending).
    pub fn past_terms(&self) -> Result<Vec<PastTermSummary>> {
        todo!("M14 T004: past_terms")
    }
}
