//! Course marks, needed-in-final, and term GPA (M9, §13.3). Pure helpers take all inputs
//! as arguments; nothing here reads the clock.

use crate::db::Planner;
use crate::error::Result;
use crate::model::{AttendanceBand, CourseMarks, GpaSummary, GradeRow};

impl Planner {
    /// R35-R38: marks, projected grade, and needed-in-final of one course.
    pub fn course_marks(&self, _course_id: i64) -> Result<CourseMarks> {
        todo!("M9: R35-R38")
    }

    /// R40: GPA over the non-archived graded courses of the active term.
    pub fn term_gpa(&self) -> Result<GpaSummary> {
        todo!("M9: R40")
    }
}

/// R35: share of the component's marks earned at attendance `pct`, as a percentage: the
/// `credit_pct` of the band with the highest `min_pct <= pct`.
#[allow(dead_code)]
pub(crate) fn band_credit(_bands: &[AttendanceBand], _pct: f64) -> f64 {
    todo!("M9: R35")
}

/// R36: `(earned, assessed fraction)` of an items component from the entered `scores`:
/// earned = best `best_count` summed / (`best_count` x `item_max`) x `marks`; fraction =
/// min(entered, K) / K.
#[allow(dead_code)]
pub(crate) fn items_earned(
    _scores: &[f64],
    _best_count: u32,
    _item_max: f64,
    _marks: f64,
) -> (f64, f64) {
    todo!("M9: R36")
}

/// R37: round to 1 decimal.
#[allow(dead_code)]
pub(crate) fn round1(_x: f64) -> f64 {
    todo!("M9: R37")
}

/// R40: round to 2 decimals.
#[allow(dead_code)]
pub(crate) fn round2(_x: f64) -> f64 {
    todo!("M9: R40")
}

/// R38: round up to 1 decimal.
#[allow(dead_code)]
pub(crate) fn ceil1(_x: f64) -> f64 {
    todo!("M9: R38")
}

/// R37: letter of the grade-scale row with the highest `min_pct <= pct`.
#[allow(dead_code)]
pub(crate) fn letter_for(_scale: &[GradeRow], _pct: f64) -> Option<String> {
    todo!("M9: R37")
}

#[cfg(test)]
mod tests;
