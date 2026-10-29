//! Rule sets and the grade scale (M9, §13.3).

use crate::db::Planner;
use crate::error::Result;
use crate::model::{GradeRow, RuleSet, RuleSetInput};

impl Planner {
    /// R34: every rule set with its components (by position) and attendance bands.
    pub fn list_rule_sets(&self) -> Result<Vec<RuleSet>> {
        todo!("M9: R34")
    }

    /// R34: create or update a rule set after validating it; components update in place by id.
    pub fn save_rule_set(&mut self, _input: RuleSetInput) -> Result<RuleSet> {
        todo!("M9: R34")
    }

    /// R34: delete an unused rule set; a set that a course uses is an error.
    pub fn delete_rule_set(&mut self, _id: i64) -> Result<()> {
        todo!("M9: R34")
    }

    /// R39: the grade scale, by `min_pct` descending.
    pub fn grade_scale(&self) -> Result<Vec<GradeRow>> {
        todo!("M9: R39")
    }

    /// R39: validate and replace the whole grade scale.
    pub fn save_grade_scale(&mut self, _rows: &[GradeRow]) -> Result<()> {
        todo!("M9: R39")
    }
}
