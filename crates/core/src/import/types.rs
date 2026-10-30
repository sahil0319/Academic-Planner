//! Public types of the course and timetable import (M10, R42-R48).

use std::fmt;

use chrono::{NaiveDate, NaiveTime, Weekday};

use crate::model::{Slot, SyncReport, WeekParity};

/// R48: the plan `preview_import` returns and `apply_import` applies. When `errors` is
/// non-empty (R42), `term`, `add`, and `update` are empty and nothing may be written.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportPlan {
    /// R47: the term to create; `None` when an active term exists or the file has no term.
    pub term: Option<ImportTerm>,
    /// R43: courses whose code matches no existing course.
    pub add: Vec<CourseAdd>,
    /// R43, R44, R45: matched courses with field or slot changes.
    pub update: Vec<CourseUpdate>,
    /// R42, R44, R47: non-fatal findings (unknown keys, archived match, differing term).
    pub warnings: Vec<ImportIssue>,
    /// R42: every validation error with its JSON path.
    pub errors: Vec<ImportIssue>,
}

impl ImportPlan {
    /// R48: true when the plan has no term, no course to add, and no course to update, so
    /// importing the same file twice yields an empty second plan.
    pub fn is_empty(&self) -> bool {
        self.term.is_none() && self.add.is_empty() && self.update.is_empty()
    }
}

/// R42: one finding with the JSON path of the offending value, e.g. `courses[2].slots[0].end`.
/// `Display` renders `path: message`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportIssue {
    /// JSON path (`document` for text that is not JSON).
    pub path: String,
    /// What is wrong.
    pub message: String,
}

impl fmt::Display for ImportIssue {
    /// R42: `path: message`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path, self.message)
    }
}

/// R47: the term an import creates when no active term exists.
#[derive(Debug, Clone, PartialEq)]
pub struct ImportTerm {
    pub name: String,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

/// R43: a new course; the code is kept as written (trimmed).
#[derive(Debug, Clone, PartialEq)]
pub struct CourseAdd {
    pub code: String,
    pub name: String,
    pub instructor: Option<String>,
    pub room: Option<String>,
    pub min_attendance_pct: Option<i64>,
    pub credits: Option<f64>,
    pub rule_set: Option<RuleSetRef>,
    /// R45: slots start at the effective date.
    pub slots: Vec<ImportSlot>,
}

/// R42: a rule set resolved by exact name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSetRef {
    pub id: i64,
    pub name: String,
}

/// R43-R45: a matched course with its changes. Listed only when `changes` is non-empty or
/// `slots` is `Some`.
#[derive(Debug, Clone, PartialEq)]
pub struct CourseUpdate {
    pub course_id: i64,
    pub code: String,
    /// R44: an archived match stays archived.
    pub archived: bool,
    /// R44: present-and-different fields only.
    pub changes: Vec<FieldChange>,
    /// R45: `Some` only when the file's slot set differs from the current one.
    pub slots: Option<SlotChange>,
}

/// R44: a course field an import can change. `code`, `notes`, and `archived` are never
/// changed by an import.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportField {
    Name,
    Credits,
    RuleSet,
    Instructor,
    Room,
    MinAttendancePct,
}

/// R44: a field value, `None` meaning cleared.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    /// name, instructor, room.
    Text(Option<String>),
    /// min_attendance_pct.
    Integer(Option<i64>),
    /// credits.
    Number(Option<f64>),
    /// rule_set.
    RuleSet(Option<RuleSetRef>),
}

/// R44: one field change, old to new.
#[derive(Debug, Clone, PartialEq)]
pub struct FieldChange {
    pub field: ImportField,
    pub old: FieldValue,
    pub new: FieldValue,
}

/// R45: slot replacement for a course (R7 semantics): `end` slots get `valid_to = E - 1`,
/// `add` slots start at E. Never both empty.
#[derive(Debug, Clone, PartialEq)]
pub struct SlotChange {
    pub end: Vec<Slot>,
    pub add: Vec<ImportSlot>,
}

/// R45, R59: a slot as the file states it. `week_parity` is the file's `biweekly` value
/// already converted through R54 with the effective date (R49); `None` = every week.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportSlot {
    pub weekday: Weekday,
    pub start: NaiveTime,
    pub end: NaiveTime,
    pub room: Option<String>,
    pub week_parity: Option<WeekParity>,
}

/// R48: what `apply_import` did: the applied plan and the sync that followed (R6).
#[derive(Debug, Clone, PartialEq)]
pub struct ImportReport {
    pub plan: ImportPlan,
    pub sync: SyncReport,
}
