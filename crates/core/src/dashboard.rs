//! Dashboard queries (M2).

use chrono::NaiveDateTime;

use crate::db::Planner;
use crate::error::Result;
use crate::model::Dashboard;

impl Planner {
    /// R16–R20: today's classes, next class, upcoming and overdue assessments, sessions
    /// awaiting attendance, and below-target courses, all relative to `now`.
    pub fn dashboard(&self, now: NaiveDateTime) -> Result<Dashboard> {
        let _ = now;
        todo!("M2: R16, R17, R18, R19, R20")
    }
}
