//! View model for the Courses-page import panel (M10, R42-R50): pure formatting and the
//! apply-gate, no Slint types, no clock, no database. Contract:
//! `specs/010-m10-course-import/contracts/ui-import.md`.
//!
//! T022 stubs: bodies are `todo!("M10")` until T024.

use chrono::NaiveDate;
use planner_core::{
    CourseAdd, FieldChange, ImportIssue, ImportPlan, ImportReport, ImportSlot, ImportTerm,
    SlotChange,
};

#[cfg(test)]
mod tests;

/// R48, R49: the preview the panel last produced, kept to decide whether Apply is allowed.
/// `text` and `date_text` are the exact inputs the preview ran on.
#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewState {
    /// The pasted import text the preview ran on.
    pub text: String,
    /// The effective-date field text the preview ran on.
    pub date_text: String,
    /// R42: the plan has validation errors.
    pub has_errors: bool,
    /// R48: the plan has nothing to apply.
    pub is_empty: bool,
}

/// R42: one issue as `path: message`.
#[allow(dead_code)]
pub fn issue_line(_issue: &ImportIssue) -> String {
    todo!("M10")
}

/// R42: heading above the error list, e.g. `3 errors — nothing will be imported`.
#[allow(dead_code)]
pub fn errors_heading(_n: usize) -> String {
    todo!("M10")
}

/// R48: one-line plan summary, or `No changes. Everything already matches.` when empty.
#[allow(dead_code)]
pub fn plan_summary(_plan: &ImportPlan) -> String {
    todo!("M10")
}

/// R47: the term to create, e.g. `Semester 7, 5 Jan 2026 – 1 May 2026`.
#[allow(dead_code)]
pub fn term_line(_term: &ImportTerm) -> String {
    todo!("M10")
}

/// R43: a course to add, e.g. `CSE 4711 Artificial Intelligence · 3 credits · Theory`.
#[allow(dead_code)]
pub fn course_add_line(_course: &CourseAdd) -> String {
    todo!("M10")
}

/// R44: one field change, e.g. `Instructor: — → Dr. Rahman`.
#[allow(dead_code)]
pub fn field_change_line(_change: &FieldChange) -> String {
    todo!("M10")
}

/// R45: one slot, e.g. `Mon 10:30 AM–11:45 AM · Room 301`.
#[allow(dead_code)]
pub fn slot_line(_slot: &ImportSlot) -> String {
    todo!("M10")
}

/// R45: the slot changes of a course as `Ends after …` and `From …` lines for effective
/// date `effective`.
#[allow(dead_code)]
pub fn slot_change_lines(_change: &SlotChange, _effective: NaiveDate) -> Vec<String> {
    todo!("M10")
}

/// R48: result line after apply, e.g. `Imported: 5 added, 1 updated. 72 sessions created, 3 removed.`
#[allow(dead_code)]
pub fn import_report_text(_report: &ImportReport) -> String {
    todo!("M10")
}

/// R48: true only if the preview is for this exact text and date, has no errors, and is
/// not empty.
#[allow(dead_code)]
pub fn can_apply(_preview: Option<&PreviewState>, _text: &str, _date_text: &str) -> bool {
    todo!("M10")
}
