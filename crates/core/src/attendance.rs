//! Attendance percentages and per-course summaries (M2).

use crate::db::{load_course, Planner};
use crate::error::Result;
use crate::model::{AttendanceSummary, Status};

impl Planner {
    /// R13–R15: counts of every session of one course (regular and makeup, past and
    /// future) by status, the percentage, and the below-target flag. Works for archived
    /// courses so their history stays visible (R3).
    pub fn attendance_summary(&self, course_id: i64) -> Result<AttendanceSummary> {
        let course = load_course(&self.conn, course_id)?;
        let mut summary = AttendanceSummary {
            course_id: course.id,
            course_code: course.code,
            course_name: course.name,
            attended: 0,
            absent: 0,
            cancelled: 0,
            pending: 0,
            pct: None,
            target: course.min_attendance_pct,
            below_target: false,
        };
        let mut stmt = self
            .conn
            .prepare("SELECT status, count(*) FROM session WHERE course_id = ?1 GROUP BY status")?;
        let counts = stmt
            .query_map([course_id], |r| {
                Ok((crate::db::col_status(r, 0)?, r.get::<_, u32>(1)?))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        for (status, n) in counts {
            match status {
                Status::Attended => summary.attended = n,
                Status::Absent => summary.absent = n,
                Status::Cancelled => summary.cancelled = n,
                Status::Pending => summary.pending = n,
            }
        }
        summary.pct = pct(summary.attended, summary.absent);
        summary.below_target = below_target(summary.pct, summary.target);
        Ok(summary)
    }

    /// R3, R13–R15: summaries for every non-archived course of the active term, ordered by
    /// course code. No active term: empty (research R-D2, R-D9).
    pub fn all_summaries(&self) -> Result<Vec<AttendanceSummary>> {
        let mut stmt = self.conn.prepare(
            "SELECT c.id FROM course c
             WHERE c.archived = 0 AND c.term_id = (SELECT id FROM term WHERE is_active = 1)
             ORDER BY c.code",
        )?;
        let ids = stmt
            .query_map([], |r| r.get::<_, i64>(0))?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        ids.into_iter()
            .map(|id| self.attendance_summary(id))
            .collect()
    }
}

/// R13: `attended / (attended + absent) × 100`, rounded to one decimal with ties away from
/// zero (`f64::round`, research R-D9); `None` when the denominator is 0.
pub(crate) fn pct(attended: u32, absent: u32) -> Option<f64> {
    let total = u64::from(attended) + u64::from(absent);
    if total == 0 {
        return None;
    }
    let ratio = attended as f64 / total as f64;
    Some((ratio * 1000.0).round() / 10.0)
}

/// R14: `target.is_some() && pct.is_some() && pct < target`, on the rounded `pct`.
pub(crate) fn below_target(pct: Option<f64>, target: Option<i64>) -> bool {
    match (pct, target) {
        (Some(pct), Some(target)) => pct < target as f64,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn r13_pct_rounds_to_one_decimal() {
        assert_eq!(pct(3, 1), Some(75.0));
        assert_eq!(pct(2, 1), Some(66.7));
        assert_eq!(pct(149, 50), Some(74.9));
        assert_eq!(pct(0, 4), Some(0.0));
        assert_eq!(pct(4, 0), Some(100.0));
    }

    #[test]
    fn r13_pct_ties_round_away_from_zero() {
        // 1 / 16 = 6.25 % rounds to 6.3.
        assert_eq!(pct(1, 15), Some(6.3));
    }

    #[test]
    fn r13_pct_none_without_attended_or_absent() {
        assert_eq!(pct(0, 0), None);
    }

    #[test]
    fn r14_below_target_boundaries() {
        assert!(below_target(Some(74.9), Some(75)));
        assert!(!below_target(Some(75.0), Some(75)));
        assert!(!below_target(Some(10.0), None));
        assert!(!below_target(None, Some(75)));
        assert!(!below_target(None, None));
    }
}
