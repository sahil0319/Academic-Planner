//! Bridge: view-model rows to the Slint structs of `ui/types.slint` (contracts/ui-bridge.md,
//! research R-D18). Field copies only; all formatting happens in `viewmodel.rs`.

use planner_core::{Status, ThemeMode};
use std::cell::Cell;

use slint::winit_030::{winit::window::Theme as NativeTheme, WinitWindowAccessor};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::viewmodel::{self as vm, ChipKind};
use crate::{
    AppWindow, AssessmentRow, BandRow, CalendarDay, CalendarWeek, Chip, ComponentRow, CourseRow,
    DayGroup, GradeRowView, HolidayRow, ImportLine, ImportLineKind, MarkRow, NeededRowView,
    Palette, ProgressCardRow, RuleSetRow, SessionRow, SlotRow, StatusAction, SummaryView, Theme,
};

/// R-D18: Slint `int` is `i32`; an id that does not fit is reported, never truncated.
pub fn id(value: i64) -> Result<i32, String> {
    i32::try_from(value).map_err(|_| format!("Record id {value} is too large to show"))
}

thread_local! {
    /// Native window appearance last chosen by `apply_theme` (see `remembered_native_theme`).
    static NATIVE_THEME: Cell<Option<NativeTheme>> = const { Cell::new(None) };
}

/// §6 "Appearance:": the native window appearance for a mode; `None` follows the system.
fn native_theme(mode: ThemeMode) -> Option<NativeTheme> {
    match mode {
        ThemeMode::System => None,
        ThemeMode::Light => Some(NativeTheme::Light),
        ThemeMode::Dark => Some(NativeTheme::Dark),
    }
}

/// The appearance last chosen by `apply_theme`, for the native window once it exists (its title
/// bar); `apply_theme` may run before the event loop has created that window.
pub fn remembered_native_theme() -> Option<NativeTheme> {
    NATIVE_THEME.with(Cell::get)
}

/// §5 Settings theme, §6 "Appearance:": shows `mode` at once. Sets `Theme.mode` (the switch
/// segment and every color token), the std-widgets color scheme (System sets `unknown`, so the
/// std widgets follow the operating system), and the native window appearance (title bar).
pub fn apply_theme(window: &AppWindow, mode: ThemeMode) {
    // The native title bar follows the window's own appearance, not the Slint palette: set it
    // now when the native window exists, and remember it for when the window is created.
    let native = native_theme(mode);
    NATIVE_THEME.with(|t| t.set(native));
    window.window().with_winit_window(|w| w.set_theme(native));
    window.global::<Theme>().set_mode(vm::theme_index(mode));
    window.global::<Palette>().set_color_scheme(match mode {
        ThemeMode::System => slint::language::ColorScheme::Unknown,
        ThemeMode::Light => slint::language::ColorScheme::Light,
        ThemeMode::Dark => slint::language::ColorScheme::Dark,
    });
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

/// R11, R12: session row. `can_shift` is false: only the dashboard offers "Move to next week".
pub fn session_row(r: &vm::SessionRow) -> Result<SessionRow, String> {
    session_row_with_shift(r, false)
}

/// R58: session row that also says whether "Move to next week" is offered.
pub fn session_row_with_shift(r: &vm::SessionRow, can_shift: bool) -> Result<SessionRow, String> {
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
        biweekly: r.biweekly,
        biweekly_label: vm::biweekly_label(r.biweekly).into(),
        can_shift,
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

/// One mini calendar cell (§6 mini calendar change).
pub fn calendar_day(c: &vm::CalendarDay) -> CalendarDay {
    CalendarDay {
        key: c.key.as_str().into(),
        day: c.day.as_str().into(),
        in_month: c.in_month,
        is_today: c.is_today,
        is_selected: c.is_selected,
        has_class: c.has_class,
        has_due: c.has_due,
        is_weekend: c.is_weekend,
        holiday: c.holiday.as_str().into(),
        label: c.label.as_str().into(),
    }
}

/// The 42-cell grid as six Monday-to-Sunday weeks.
pub fn calendar_weeks(cells: &[vm::CalendarDay]) -> ModelRc<CalendarWeek> {
    model(
        cells
            .chunks(7)
            .map(|week| CalendarWeek {
                days: model(week.iter().map(calendar_day).collect()),
            })
            .collect(),
    )
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
        selected: r.selected,
    })
}

/// M10: one import-preview line.
pub fn import_line(l: &vm::import::PreviewLine) -> ImportLine {
    use vm::import::LineKind;
    ImportLine {
        text: l.text.as_str().into(),
        kind: match l.kind {
            LineKind::Heading => ImportLineKind::Heading,
            LineKind::Error => ImportLineKind::Error,
            LineKind::Summary => ImportLineKind::Summary,
            LineKind::Section => ImportLineKind::Section,
            LineKind::Item => ImportLineKind::Item,
            LineKind::Detail => ImportLineKind::Detail,
            LineKind::Warning => ImportLineKind::Warning,
        },
    }
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
        biweekly: r.biweekly,
        biweekly_label: vm::biweekly_label(r.biweekly).into(),
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

/// R34: rule set list row.
pub fn rule_set_row(r: &vm::RuleSetRow) -> Result<RuleSetRow, String> {
    Ok(RuleSetRow {
        id: id(r.id)?,
        name: r.name.as_str().into(),
        summary: r.summary.as_str().into(),
    })
}

/// R34: component form row; `-1` is a row not saved yet.
pub fn component_row(r: &vm::ComponentFormRow) -> Result<ComponentRow, String> {
    Ok(ComponentRow {
        id: r.id.map(id).transpose()?.unwrap_or(-1),
        name: r.name.as_str().into(),
        is_items: r.is_items,
        weight: r.weight.as_str().into(),
        item_count: r.item_count.as_str().into(),
        best_count: r.best_count.as_str().into(),
        item_max: r.item_max.as_str().into(),
        is_final: r.is_final,
        error: SharedString::new(),
    })
}

/// R35: attendance band form row from `(min, credit)` text.
pub fn band_row((min, credit): &(String, String)) -> BandRow {
    BandRow {
        min: min.as_str().into(),
        credit: credit.as_str().into(),
        error: SharedString::new(),
    }
}

/// R39: grade scale form row from `(letter, min, points)` text.
pub fn grade_row((letter, min, points): &(String, String, String)) -> GradeRowView {
    GradeRowView {
        letter: letter.as_str().into(),
        min: min.as_str().into(),
        points: points.as_str().into(),
        error: SharedString::new(),
    }
}

/// R61–R65: Progress card row; every field is already formatted text or a bar fraction.
pub fn progress_card(c: &vm::ProgressCardView) -> ProgressCardRow {
    ProgressCardRow {
        code: c.code.as_str().into(),
        name: c.name.as_str().into(),
        status_label: c.status_label.as_str().into(),
        chip: chip(c.chip),
        attendance_figure: c.attendance_figure.as_str().into(),
        attendance_line: c.attendance_line.as_str().into(),
        target_text: c.target_text.as_str().into(),
        bar_fill: c.bar_fill,
        has_marker: c.has_marker,
        marker_at: c.marker_at,
        miss_line: c.miss_line.as_str().into(),
        marks_figure: c.marks_figure.as_str().into(),
        marks_line: c.marks_line.as_str().into(),
        best_grade_line: c.best_grade_line.as_str().into(),
    }
}

/// R35/R36: marks card row from `(component name, "earned of possible")`.
pub fn mark_row((name, marks): &(String, String)) -> MarkRow {
    MarkRow {
        name: name.as_str().into(),
        marks: marks.as_str().into(),
    }
}

/// R38: needed-in-final table row.
pub fn needed_row(r: &vm::NeededRowView) -> NeededRowView {
    NeededRowView {
        letter: r.letter.as_str().into(),
        needed: r.text.as_str().into(),
    }
}
