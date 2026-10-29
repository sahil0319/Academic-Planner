//! Course and timetable JSON import (M10, docs/spec.md §13.4).

mod apply;
mod parse;
mod plan;
mod types;

use chrono::NaiveDate;

use crate::db::Planner;
use crate::error::Result;

pub use types::{
    CourseAdd, CourseUpdate, FieldChange, FieldValue, ImportField, ImportIssue, ImportPlan,
    ImportReport, ImportSlot, ImportTerm, RuleSetRef, SlotChange,
};

impl Planner {
    /// R42-R48: parses and validates the whole document and returns the plan. Writes nothing.
    pub fn preview_import(&self, json: &str, effective_from: NaiveDate) -> Result<ImportPlan> {
        plan::build_plan(&self.conn, json, effective_from)
    }

    /// R48: applies the plan `preview_import` returns for the same input, in one transaction,
    /// then syncs (R6) with min(effective_from, today). Errors in the plan (R42) give
    /// `Error::Validation` listing every "path: message"; nothing is written.
    pub fn apply_import(
        &mut self,
        json: &str,
        effective_from: NaiveDate,
        today: NaiveDate,
    ) -> Result<ImportReport> {
        apply::apply(&mut self.conn, json, effective_from, today)
    }
}
