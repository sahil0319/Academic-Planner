//! Bridge: view-model rows to the Slint structs of `ui/types.slint` (contracts/ui-bridge.md,
//! research R-D18). Field copies only; all formatting happens in `viewmodel.rs`.

use planner_core::Status;
use slint::{ModelRc, SharedString, VecModel};

use crate::viewmodel::{self as vm, ChipKind};
use crate::{
    AssessmentRow, Chip, CourseRow, DayGroup, HolidayRow, SessionRow, SlotRow, StatusAction,
    SummaryView,
};

/// R-D18: Slint `int` is `i32`; an id that does not fit is reported, never truncated.
pub fn id(value: i64) -> Result<i32, String> {
    i32::try_from(value).map_err(|_| format!("Record id {value} is too large to show"))
}

/// R11: chip colors of a status.
pub fn chip(kind: ChipKind) -> Chip {
    match kind {
        ChipKind::Attended => Chip::Attended,
        ChipKind::Absent => Chip::Absent,
        ChipKind::Cancelled => Chip::Cancelled,
        ChipKind::Pending => Chip::Pending,
    }
}

/// R11: the core status a row action asks for.
pub fn status(action: StatusAction) -> Status {
    match action {
        StatusAction::Attended => Status::Attended,
        StatusAction::Absent => Status::Absent,
        StatusAction::Cancelled => Status::Cancelled,
        StatusAction::Pending => Status::Pending,
    }
}

/// A Slint list model from a vector.
pub fn model<T: Clone + 'static>(items: Vec<T>) -> ModelRc<T> {
    ModelRc::new(VecModel::from(items))
}

/// A Slint string list, e.g. combo box options or banner lines.
pub fn strings(items: &[String]) -> ModelRc<SharedString> {
    model(items.iter().map(SharedString::from).collect())
}

/// Converts every item, stopping at the first id that does not fit.
pub fn rows<T, U>(
    items: &[T],
    convert: impl Fn(&T) -> Result<U, String>,
) -> Result<ModelRc<U>, String>
where
    U: Clone + 'static,
{
    items
        .iter()
        .map(convert)
        .collect::<Result<Vec<_>, _>>()
        .map(model)
}

/// R11, R12: session row.
pub fn session_row(r: &vm::SessionRow) -> Result<SessionRow, String> {
    Ok(SessionRow {
        id: id(r.id)?,
        date: r.date.as_str().into(),
        weekday: r.weekday.as_str().into(),
        time: r.time.as_str().into(),
        course: r.course.as_str().into(),
        room: r.room.as_str().into(),
        status_label: r.status_label.as_str().into(),
        chip: chip(r.chip),
        can_mark: r.can_mark,
        note: r.note.as_str().into(),
    })
}

/// Week view day group.
pub fn day_group(g: &vm::DayGroup) -> Result<DayGroup, String> {
    Ok(DayGroup {
        title: g.title.as_str().into(),
        rows: rows(&g.rows, session_row)?,
    })
}

/// Assessment row.
pub fn assessment_row(r: &vm::AssessmentRow) -> Result<AssessmentRow, String> {
    Ok(AssessmentRow {
        id: id(r.id)?,
        course_code: r.course_code.as_str().into(),
        kind: r.kind.as_str().into(),
        title: r.title.as_str().into(),
        due: r.due.as_str().into(),
        completed: r.completed,
        notes: r.notes.as_str().into(),
    })
}

/// R3: course row.
pub fn course_row(r: &vm::CourseRow) -> Result<CourseRow, String> {
    Ok(CourseRow {
        id: id(r.id)?,
        code: r.code.as_str().into(),
        name: r.name.as_str().into(),
        instructor: r.instructor.as_str().into(),
        room: r.room.as_str().into(),
        target: r.target.as_str().into(),
        archived: r.archived,
    })
}

/// Timetable rule row.
pub fn slot_row(r: &vm::SlotRow) -> Result<SlotRow, String> {
    Ok(SlotRow {
        id: id(r.id)?,
        weekday: r.weekday.as_str().into(),
        time: r.time.as_str().into(),
        course: r.course.as_str().into(),
        room: r.room.as_str().into(),
        valid: r.valid.as_str().into(),
    })
}

/// Holiday row.
pub fn holiday_row(r: &vm::HolidayRow) -> HolidayRow {
    HolidayRow {
        key: r.key.as_str().into(),
        date: r.date.as_str().into(),
        label: r.label.as_str().into(),
    }
}

/// R13–R15: attendance summary.
pub fn summary_view(s: &vm::SummaryView) -> SummaryView {
    SummaryView {
        attended: s.attended.as_str().into(),
        absent: s.absent.as_str().into(),
        cancelled: s.cancelled.as_str().into(),
        pending: s.pending.as_str().into(),
        pct: s.pct.as_str().into(),
        target: s.target.as_str().into(),
        below_target: s.below_target,
    }
}
