//! Dashboard page: fills `DashboardState` (contracts/ui-bridge.md "DashboardState").

use std::rc::Rc;

use chrono::{NaiveDate, NaiveDateTime};
use planner_core::{Assessment, AssessmentFilter, PublicHoliday, SessionView};
use slint::{ComponentHandle, SharedString};

use crate::bridge;
use crate::controller::App;
use crate::viewmodel::{self, DashboardView};
use crate::{DashboardState, SessionRow};

/// R1, R14, R16–R20: reloads the dashboard relative to a fresh `now`, then the mini calendar and
/// day panel. A core error or an id that does not fit a Slint `int` is shown in the page's
/// `message`.
pub fn refresh(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let state = window.global::<DashboardState>();
    let now = app.now();
    // R66: the term line shows with and without an active term.
    // T028: without an active term the page shows one "No term yet" card instead of the sections.
    match app.planner().active_term() {
        Ok(term) => {
            let line = viewmodel::dashboard_term_line(term.as_ref(), app.today());
            state.set_term_line(line.text.into());
            state.set_term_link(line.link.into());
            state.set_has_term(term.is_some());
        }
        Err(e) => {
            state.set_message(viewmodel::error_text(&e).into());
            return;
        }
    }
    let loaded = app.planner().dashboard(now);
    let (view, next_shift) = match loaded {
        Ok(d) => {
            // R58: the next class offers "Move to next week" when it qualifies.
            let target = d
                .next_class
                .as_ref()
                .filter(|v| viewmodel::can_shift_biweekly(v, now.date()))
                .map(|v| (v.session.id, v.session.course_id, v.session.date));
            (viewmodel::dashboard_view(&d), target)
        }
        Err(e) => {
            state.set_message(viewmodel::error_text(&e).into());
            return;
        }
    };
    app.state().shiftable = next_shift.into_iter().collect();
    if let Err(text) = fill(&state, &view, next_shift.is_some()) {
        state.set_message(text.into());
        return;
    }
    if let Err(text) = fill_calendar(app, &state, now) {
        state.set_message(text.into());
        return;
    }
    if let Err(text) = fill_extra_class(app, &state) {
        state.set_message(text.into());
    }
}

/// Copies the view into the Slint global; stops at the first id that does not fit.
fn fill(
    state: &DashboardState<'_>,
    view: &DashboardView,
    next_can_shift: bool,
) -> Result<(), String> {
    state.set_banner(bridge::strings(&view.banner));
    // R16, R17: hide Next class when that session is already in today's day panel. The panel
    // shows today on start and after a rollover; `view.today` holds exactly today's rows.
    state.set_show_next(viewmodel::show_next_card(
        view.next_class.as_ref(),
        &view.today,
    ));
    // R17: no next class hides the row and shows the empty state.
    match &view.next_class {
        Some(row) => {
            state.set_next_class(bridge::session_row_with_shift(row, next_can_shift)?);
            state.set_has_next(true);
        }
        None => {
            state.set_next_class(SessionRow::default());
            state.set_has_next(false);
        }
    }
    state.set_upcoming(bridge::rows(&view.upcoming, bridge::assessment_row)?);
    state.set_overdue(bridge::rows(&view.overdue, bridge::assessment_row)?);
    state.set_awaiting(bridge::rows(&view.awaiting, bridge::session_row)?);
    Ok(())
}

/// §6 mini calendar change, R3, R12, R16: fills the month grid from `UiState.calendar_month` and
/// the day panel from `UiState.calendar_selected`. Sessions come from `sessions_between` over
/// the 42-day grid (plus the selected day when it lies outside the grid) and assessments from
/// `list_assessments(All)`; both exclude archived courses in core.
fn fill_calendar(app: &App, state: &DashboardState<'_>, now: NaiveDateTime) -> Result<(), String> {
    let today = now.date();
    let (month, selected) = {
        let s = app.state();
        (s.calendar_month, s.calendar_selected)
    };
    let first = viewmodel::grid_start(month);
    let last = viewmodel::grid_end(month);
    // R77, R78, D33: the cached public holidays of the visible grid (and the selected day).
    let public_holidays =
        super::public_holidays::cached_between(app, first.min(selected), last.max(selected))?;
    let (grid_views, selected_views, assessments, holidays) = {
        let planner = app.planner();
        // §6 mini calendar change, R14: the active term's holidays; none without a term.
        let holidays = planner
            .active_term()
            .map_err(|e| viewmodel::error_text(&e))?
            .map(|t| t.holidays)
            .unwrap_or_default();
        let grid_views = planner
            .sessions_between(first, last, now)
            .map_err(|e| viewmodel::error_text(&e))?;
        let selected_views = if selected < first || selected > last {
            planner
                .sessions_between(selected, selected, now)
                .map_err(|e| viewmodel::error_text(&e))?
        } else {
            Vec::new()
        };
        let assessments = planner
            .list_assessments(AssessmentFilter::All)
            .map_err(|e| viewmodel::error_text(&e))?;
        (grid_views, selected_views, assessments, holidays)
    };

    let in_grid: Vec<Assessment> = assessments
        .iter()
        .filter(|a| (first..=last).contains(&a.due_at.date()))
        .cloned()
        .collect();
    let mut cells = viewmodel::month_grid(
        month,
        today,
        selected,
        &viewmodel::class_days(&grid_views),
        &viewmodel::due_days(&in_grid),
        &holidays,
    );
    // R78: the public holiday fill, label, and legend entry.
    viewmodel::mark_public_holidays(&mut cells, &public_holidays);
    state.set_month_title(viewmodel::month_title(month).into());
    let initials: Vec<String> = viewmodel::weekday_initials()
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    state.set_weekday_initials(bridge::strings(&initials));
    state.set_calendar(bridge::calendar_weeks(&cells));

    // R16: the selected day's sessions by start time, as core orders today's classes.
    let mut day_views: Vec<&SessionView> = grid_views
        .iter()
        .chain(selected_views.iter())
        .filter(|v| v.session.date == selected)
        .collect();
    day_views.sort_by(|a, b| {
        (a.session.start_time, &a.course_code, a.session.id).cmp(&(
            b.session.start_time,
            &b.course_code,
            b.session.id,
        ))
    });
    // R58: each row says whether "Move to next week" is offered; the targets are remembered.
    let mut day_rows: Vec<(viewmodel::SessionRow, bool)> = Vec::new();
    for view in day_views {
        let can_shift = viewmodel::can_shift_biweekly(view, today);
        if can_shift {
            let s = &view.session;
            app.state().shiftable.push((s.id, s.course_id, s.date));
        }
        day_rows.push((viewmodel::session_row(view), can_shift));
    }
    // Assessments due that day, completed or not, in core's due order; the panel shows the time.
    let due_rows = assessments
        .iter()
        .filter(|a| a.due_at.date() == selected)
        .map(|a| {
            bridge::assessment_row(&viewmodel::AssessmentRow {
                due: viewmodel::format_time(a.due_at.time()),
                ..viewmodel::assessment_row(a)
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    state.set_panel_title(viewmodel::day_panel_title(selected, today).into());
    // R74, R78: the day panel lines name the kind of day off and the public holiday; the legend
    // lists the day off kinds and Public holiday.
    let on_selected: Vec<PublicHoliday> = public_holidays
        .iter()
        .filter(|p| p.date == selected)
        .cloned()
        .collect();
    state.set_panel_public_holiday(viewmodel::day_panel_public_holiday_text(&on_selected).into());
    state.set_panel_holiday(
        viewmodel::day_panel_day_off_text(viewmodel::day_off_on(&holidays, selected)).into(),
    );
    state.set_day_off_legend(bridge::model(
        viewmodel::calendar_legend()
            .iter()
            .map(bridge::legend_item)
            .collect(),
    ));
    state.set_panel_empty_text(viewmodel::day_panel_empty_text(selected, today).into());
    state.set_panel_sessions(bridge::rows(&day_rows, |(row, can_shift)| {
        bridge::session_row_with_shift(row, *can_shift)
    })?);
    state.set_panel_assessments(bridge::model(due_rows));
    Ok(())
}

/// Connects the row status buttons and the mini calendar controls.
pub fn wire(app: &Rc<App>) {
    let Some(window) = app.window() else {
        return;
    };
    let state = window.global::<DashboardState>();
    let weak = Rc::downgrade(app);
    state.on_set_status(move |id, action| {
        if let Some(app) = weak.upgrade() {
            set_status(&app, id, bridge::status(action));
        }
    });
    let weak = Rc::downgrade(app);
    state.on_shift_biweekly(move |id| {
        if let Some(app) = weak.upgrade() {
            shift_biweekly(&app, id);
        }
    });
    let weak = Rc::downgrade(app);
    state.on_prev_month(move || {
        if let Some(app) = weak.upgrade() {
            move_month(&app, Some(-1));
        }
    });
    let weak = Rc::downgrade(app);
    state.on_next_month(move || {
        if let Some(app) = weak.upgrade() {
            move_month(&app, Some(1));
        }
    });
    let weak = Rc::downgrade(app);
    state.on_this_month(move || {
        if let Some(app) = weak.upgrade() {
            move_month(&app, None);
        }
    });
    let weak = Rc::downgrade(app);
    state.on_select_day(move |key| {
        if let Some(app) = weak.upgrade() {
            select_day(&app, key.as_str());
        }
    });
    let weak = Rc::downgrade(app);
    state.on_open_extra_class(move || {
        if let Some(app) = weak.upgrade() {
            open_extra_class(&app);
        }
    });
    let weak = Rc::downgrade(app);
    state.on_add_extra_class(move || {
        if let Some(app) = weak.upgrade() {
            add_extra_class(&app);
        }
    });
    let weak = Rc::downgrade(app);
    state.on_cancel_extra_class(move || {
        if let Some(app) = weak.upgrade() {
            cancel_extra_class(&app);
        }
    });
}

/// §6 mini calendar change: Previous / Next move the shown month and keep the selected day;
/// Today (`None`) shows the current month and selects today. Only the dashboard is refreshed.
fn move_month(app: &App, months: Option<i32>) {
    let today = app.today();
    {
        let mut s = app.state();
        match months {
            Some(n) => s.calendar_month = viewmodel::shift_month(s.calendar_month, n),
            None => {
                s.calendar_month = viewmodel::month_start(today);
                s.calendar_selected = today;
            }
        }
    }
    refresh(app);
}

/// §6 mini calendar change: selects the clicked day by its ISO key; a day of another month also
/// moves the calendar to that month. An unknown key is ignored.
fn select_day(app: &App, key: &str) {
    let Some(date) = viewmodel::parse_date_key(key) else {
        return;
    };
    {
        let mut s = app.state();
        s.calendar_selected = date;
        s.calendar_month = viewmodel::month_start(date);
    }
    refresh(app);
}

/// R139, R140: fills the "Add extra class" course dropdown from the non-archived courses of the
/// active term (by code), keeps the chosen course when it is still listed, and keeps an open
/// form's Date on the selected day until the user has edited it. With no course the form shows
/// "Add a course first." and cannot be submitted.
fn fill_extra_class(app: &App, state: &DashboardState<'_>) -> Result<(), String> {
    let courses = app
        .planner()
        .list_courses(false)
        .map_err(|e| viewmodel::error_text(&e))?;
    let ids: Vec<i64> = courses.iter().map(|c| c.id).collect();
    let labels: Vec<String> = courses.iter().map(viewmodel::course_option_label).collect();
    let selected = app.state().calendar_selected;
    let chosen = usize::try_from(state.get_xc_course_index())
        .ok()
        .and_then(|i| app.state().extra_class_courses.get(i).copied());
    let index = chosen
        .and_then(|id| ids.iter().position(|&c| c == id))
        .and_then(|i| i32::try_from(i).ok())
        .unwrap_or(0);
    let defaults = viewmodel::extra_class_defaults(selected, ids.len());
    app.state().extra_class_courses = ids;
    state.set_xc_courses(bridge::strings(&labels));
    state.set_xc_course_index(index);
    state.set_xc_notice(defaults.notice.into());
    state.set_xc_can_submit(defaults.can_submit);
    if state.get_xc_open() {
        state.set_xc_date(
            viewmodel::extra_class_date_text(
                selected,
                &state.get_xc_date(),
                state.get_xc_date_edited(),
            )
            .into(),
        );
    }
    Ok(())
}

/// R140: opens the form under the day panel title with the selected day in the Date field.
fn open_extra_class(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let state = window.global::<DashboardState>();
    if state.get_xc_open() {
        return;
    }
    reset_extra_class_form(app, &state);
    state.set_xc_open(true);
}

/// R140: Cancel closes the form and clears it.
fn cancel_extra_class(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let state = window.global::<DashboardState>();
    reset_extra_class_form(app, &state);
    state.set_xc_open(false);
}

/// R140: puts the defaults for the selected day into every form field and clears all errors.
fn reset_extra_class_form(app: &App, state: &DashboardState<'_>) {
    let count = app.state().extra_class_courses.len();
    let defaults = viewmodel::extra_class_defaults(app.state().calendar_selected, count);
    state.set_xc_date(defaults.date.into());
    state.set_xc_date_edited(false);
    state.set_xc_course_index(defaults.course_index);
    state.set_xc_start(defaults.start.into());
    state.set_xc_start_pm(false);
    state.set_xc_end(defaults.end.into());
    state.set_xc_end_pm(false);
    state.set_xc_room(defaults.room.into());
    state.set_xc_note(defaults.note.into());
    state.set_xc_notice(defaults.notice.into());
    state.set_xc_can_submit(defaults.can_submit);
    set_xc_errors(state, &viewmodel::FormErrors::default());
    state.set_xc_form_error(SharedString::new());
}

/// R88: shows the form's field errors under the fields.
fn set_xc_errors(state: &DashboardState<'_>, errors: &viewmodel::FormErrors) {
    state.set_xc_course_error(errors.get("course").into());
    state.set_xc_date_error(errors.get("date").into());
    state.set_xc_start_error(errors.get("start").into());
    state.set_xc_end_error(errors.get("end").into());
}

/// R141: adds the extra class (a `makeup` session) through `add_makeup`. Field errors and core
/// errors show inline under the form; no sync runs because makeups are not generated sessions.
/// R142: on success the form closes, the calendar shows the added day's month with the day
/// selected, every page refreshes, and a toast names the date.
fn add_extra_class(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let state = window.global::<DashboardState>();
    let ids = app.state().extra_class_courses.clone();
    let parsed = viewmodel::parse_extra_class_form(
        &ids,
        state.get_xc_course_index(),
        &state.get_xc_date(),
        viewmodel::TimeText {
            digits: &state.get_xc_start(),
            pm: state.get_xc_start_pm(),
        },
        viewmodel::TimeText {
            digits: &state.get_xc_end(),
            pm: state.get_xc_end_pm(),
        },
        &state.get_xc_room(),
        &state.get_xc_note(),
    );
    let input = match parsed {
        Ok(input) => input,
        Err(errors) => {
            set_xc_errors(&state, &errors);
            state.set_xc_form_error(SharedString::new());
            return;
        }
    };
    set_xc_errors(&state, &viewmodel::FormErrors::default());
    let date = input.date;
    let result = app.planner_mut().add_makeup(input);
    match result {
        Ok(_) => {
            focus_calendar_on(app, date);
            reset_extra_class_form(app, &state);
            state.set_xc_open(false);
            show(app, "");
            app.changed(false);
            app.toast_unless_error(&viewmodel::extra_class_added_text(date));
        }
        Err(e) => state.set_xc_form_error(viewmodel::error_text(&e).into()),
    }
}

/// R142: shows the month of `added` and selects that day.
fn focus_calendar_on(app: &App, added: NaiveDate) {
    let (year, month, selected) = viewmodel::calendar_focus_after_add(added);
    let mut s = app.state();
    s.calendar_month = NaiveDate::from_ymd_opt(year, month, 1).unwrap_or(selected);
    s.calendar_selected = selected;
}

/// R11, R12: marks a session with a fresh `now`; the core refuses attended/absent before the
/// session starts. Status changes need no sync, so only the pages are refreshed (R6).
fn set_status(app: &App, id: i32, status: planner_core::Status) {
    let now = app.now();
    let result = app.planner_mut().set_status(i64::from(id), status, now);
    match result {
        Ok(()) => {
            show(app, "");
            app.changed(false);
        }
        Err(e) => show(app, &viewmodel::error_text(&e)),
    }
}

/// R58: moves the bi-weekly classes of the session's course from its date on to the other week,
/// then refreshes every page and toasts. An unknown session (the list changed) only refreshes;
/// a core error stays inline in the dashboard `message`.
fn shift_biweekly(app: &App, id: i32) {
    let target = app
        .state()
        .shiftable
        .iter()
        .find(|(session, _, _)| *session == i64::from(id))
        .copied();
    let Some((_, course_id, date)) = target else {
        app.refresh_all();
        return;
    };
    let result = app.planner_mut().shift_biweekly(course_id, date);
    match result {
        Ok(()) => {
            show(app, "");
            app.changed(false);
            app.toast(&viewmodel::moved_next_week_text());
        }
        Err(e) => show(app, &viewmodel::error_text(&e)),
    }
}

/// Sets the dashboard `message`.
fn show(app: &App, text: &str) {
    if let Some(window) = app.window() {
        window
            .global::<DashboardState>()
            .set_message(SharedString::from(text));
    }
}
