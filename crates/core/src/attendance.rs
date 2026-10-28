//! Attendance percentages and per-course summaries (M2).

use crate::db::Planner;
use crate::error::Result;
use crate::model::AttendanceSummary;

impl Planner {
    /// R13–R15: counts, percentage, and below-target flag for one course.
    pub fn attendance_summary(&self, course_id: i64) -> Result<AttendanceSummary> {
        let _ = course_id;
        todo!("M2: R13, R14, R15")
    }

    /// R13–R15: summaries for every non-archived course of the active term.
    pub fn all_summaries(&self) -> Result<Vec<AttendanceSummary>> {
        todo!("M2: R13, R14, R15")
    }
}
