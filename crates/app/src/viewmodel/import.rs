//! View model for the Courses-page import panel (M10, R42-R50): pure formatting and the
//! apply-gate, no Slint types, no clock, no database. Contract:
//! `specs/010-m10-course-import/contracts/ui-import.md`.

use chrono::NaiveDate;
use planner_core::{
    CourseAdd, FieldChange, FieldValue, ImportField, ImportIssue, ImportPlan, ImportReport,
    ImportSlot, ImportTerm, SlotChange,
};

use super::{format_date, format_time_range, or_dash, weekday_name};

#[cfg(test)]
mod lines_tests;
#[cfg(test)]
mod tests;

/// R48, R49: the preview the panel last produced, kept to decide whether Apply is allowed.
/// `text` and `date_text` are the exact inputs the preview ran on.
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
pub fn issue_line(issue: &ImportIssue) -> String {
    format!("{}: {}", issue.path, issue.message)
}

/// R42: heading above the error list, e.g. `3 errors — nothing will be imported`.
pub fn errors_heading(n: usize) -> String {
    let noun = if n == 1 { "error" } else { "errors" };
    format!("{n} {noun} — nothing will be imported")
}

/// `1 course`, `2 courses`.
fn courses_count(n: usize) -> String {
    if n == 1 {
        "1 course".to_string()
    } else {
        format!("{n} courses")
    }
}

/// R48: one-line plan summary, or `No changes. Everything already matches.` when empty.
pub fn plan_summary(plan: &ImportPlan) -> String {
    if plan.is_empty() {
        return "No changes. Everything already matches.".to_string();
    }
    let mut parts = Vec::new();
    if let Some(term) = &plan.term {
        parts.push(format!("Creates term \"{}\"", term.name));
    }
    if !plan.add.is_empty() {
        parts.push(format!("adds {}", courses_count(plan.add.len())));
    }
    if !plan.update.is_empty() {
        parts.push(format!("updates {}", courses_count(plan.update.len())));
    }
    parts.join(" · ")
}

/// R47: the term to create, e.g. `Semester 7, January 5, 2026 – May 1, 2026`.
pub fn term_line(term: &ImportTerm) -> String {
    format!(
        "{}, {} – {}",
        term.name,
        format_date(term.start_date),
        format_date(term.end_date)
    )
}

/// Credits without a trailing `.0` for whole numbers.
fn credits_text(credits: f64) -> String {
    format!("{credits}")
}

/// R43: a course to add, e.g. `CSE 4711 Artificial Intelligence · 3 credits · Theory`.
pub fn course_add_line(course: &CourseAdd) -> String {
    let mut parts = vec![format!("{} {}", course.code, course.name)];
    if let Some(credits) = course.credits {
        parts.push(format!("{} credits", credits_text(credits)));
    }
    if let Some(rule_set) = &course.rule_set {
        parts.push(rule_set.name.clone());
    }
    parts.join(" · ")
}

/// R44: label of a changed field, aligned with the course form.
fn field_label(field: ImportField) -> &'static str {
    match field {
        ImportField::Name => "Name",
        ImportField::Credits => "Credits",
        ImportField::RuleSet => "Rule set",
        ImportField::Instructor => "Instructor",
        ImportField::Room => "Room",
        ImportField::MinAttendancePct => "Target",
    }
}

/// R44: a field value as text, `—` when cleared.
fn field_value_text(value: &FieldValue) -> String {
    match value {
        FieldValue::Text(v) => or_dash(v.as_deref()),
        FieldValue::Integer(Some(n)) => format!("{n}%"),
        FieldValue::Number(Some(n)) => credits_text(*n),
        FieldValue::RuleSet(Some(r)) => r.name.clone(),
        FieldValue::Integer(None) | FieldValue::Number(None) | FieldValue::RuleSet(None) => {
            or_dash(None)
        }
    }
}

/// R44: one field change, e.g. `Instructor: — → Dr. Rahman`.
pub fn field_change_line(change: &FieldChange) -> String {
    format!(
        "{}: {} → {}",
        field_label(change.field),
        field_value_text(&change.old),
        field_value_text(&change.new)
    )
}

/// Weekday, 12-hour range, and optional room, shared by `slot_line` and the change lines.
fn slot_text(
    weekday: chrono::Weekday,
    start: chrono::NaiveTime,
    end: chrono::NaiveTime,
    room: Option<&str>,
) -> String {
    let base = format!(
        "{} {}",
        weekday_name(weekday),
        format_time_range(start, end)
    );
    match room.map(str::trim).filter(|r| !r.is_empty()) {
        Some(room) => format!("{base} · Room {room}"),
        None => base,
    }
}

/// R45: one slot, e.g. `Monday 10:30 AM–11:45 AM · Room 301`.
pub fn slot_line(slot: &ImportSlot) -> String {
    slot_text(slot.weekday, slot.start, slot.end, slot.room.as_deref())
}

/// R45, R7: the slot changes of a course as `Ends after …` and `From …` lines for effective
/// date `effective`; ended slots come first.
pub fn slot_change_lines(change: &SlotChange, effective: NaiveDate) -> Vec<String> {
    let last_day = effective.pred_opt().unwrap_or(effective);
    let ends = change.end.iter().map(|s| {
        format!(
            "Ends after {}: {}",
            format_date(last_day),
            slot_text(s.weekday, s.start_time, s.end_time, s.room.as_deref())
        )
    });
    let adds = change
        .add
        .iter()
        .map(|s| format!("From {}: {}", format_date(effective), slot_line(s)));
    ends.chain(adds).collect()
}

/// R48: result line after apply, e.g. `Imported: 5 added, 1 updated. 72 sessions created, 3 removed.`
pub fn import_report_text(report: &ImportReport) -> String {
    format!(
        "Imported: {} added, {} updated. {} sessions created, {} removed.",
        report.plan.add.len(),
        report.plan.update.len(),
        report.sync.created,
        report.sync.deleted
    )
}

/// R48: true only if the preview is for this exact text and date, has no errors, and is
/// not empty.
pub fn can_apply(preview: Option<&PreviewState>, text: &str, date_text: &str) -> bool {
    preview
        .is_some_and(|p| p.text == text && p.date_text == date_text && !p.has_errors && !p.is_empty)
}

/// How the panel styles one result line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    /// R42: the errors heading.
    Heading,
    /// R42: one validation error.
    Error,
    /// R48: the plan summary or the apply result.
    Summary,
    /// A section title such as `Courses to add`.
    Section,
    /// A term, a course to add, or a course to update.
    Item,
    /// A slot or a field change under an item.
    Detail,
    /// R42, R44, R47: a non-fatal finding.
    Warning,
}

/// One line of the preview result area.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreviewLine {
    /// Display text, formatted by the functions above.
    pub text: String,
    /// Styling hint for the panel.
    pub kind: LineKind,
}

fn line(kind: LineKind, text: impl Into<String>) -> PreviewLine {
    PreviewLine {
        text: text.into(),
        kind,
    }
}

/// R42, R43-R48: the result area for a preview with effective date `effective`: errors (then
/// warnings) when the plan has errors, otherwise the summary and one section per kind of change.
pub fn preview_lines(plan: &ImportPlan, effective: NaiveDate) -> Vec<PreviewLine> {
    let mut lines = Vec::new();
    if plan.errors.is_empty() {
        lines.push(line(LineKind::Summary, plan_summary(plan)));
        if let Some(term) = &plan.term {
            lines.push(line(LineKind::Section, "Term to create"));
            lines.push(line(LineKind::Item, term_line(term)));
        }
        if !plan.add.is_empty() {
            lines.push(line(LineKind::Section, "Courses to add"));
            for course in &plan.add {
                lines.push(line(LineKind::Item, course_add_line(course)));
                lines.extend(
                    course
                        .slots
                        .iter()
                        .map(|s| line(LineKind::Detail, slot_line(s))),
                );
            }
        }
        if !plan.update.is_empty() {
            lines.push(line(LineKind::Section, "Courses to update"));
            for course in &plan.update {
                let title = if course.archived {
                    format!("{} (archived)", course.code)
                } else {
                    course.code.clone()
                };
                lines.push(line(LineKind::Item, title));
                lines.extend(
                    course
                        .changes
                        .iter()
                        .map(|c| line(LineKind::Detail, field_change_line(c))),
                );
                if let Some(slots) = &course.slots {
                    lines.extend(
                        slot_change_lines(slots, effective)
                            .into_iter()
                            .map(|t| line(LineKind::Detail, t)),
                    );
                }
            }
        }
    } else {
        lines.push(line(LineKind::Heading, errors_heading(plan.errors.len())));
        lines.extend(
            plan.errors
                .iter()
                .map(|e| line(LineKind::Error, issue_line(e))),
        );
    }
    if !plan.warnings.is_empty() {
        lines.push(line(LineKind::Section, "Warnings"));
        lines.extend(
            plan.warnings
                .iter()
                .map(|w| line(LineKind::Warning, issue_line(w))),
        );
    }
    lines
}
