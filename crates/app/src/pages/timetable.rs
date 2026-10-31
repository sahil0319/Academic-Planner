//! Timetable page: fills `TimetableState` (contracts/ui-bridge.md "TimetableState"). The Week tab
//! shows one Monday–Sunday week grouped by day (research R-D17) with status actions and reschedule
//! for the selected session; the Rules tab lists the slots valid today with add, edit (R7), and
//! end (R5) forms. All display text comes from `viewmodel.rs`.

use std::rc::{Rc, Weak};

use chrono::NaiveDate;
use planner_core::{SessionView, Slot};
use slint::{ComponentHandle, SharedString};

use crate::controller::App;
use crate::viewmodel as vm;
use crate::{bridge, AppWindow, StatusAction, TimetableState};

/// Reloads the page from the database: the week from `UiState.week_start`, the selected session,
/// the course options, and the slots valid today.
pub fn refresh(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let g = window.global::<TimetableState>();
    let now = app.now();
    let today = now.date();
    let (week_start, selected_id) = {
        let s = app.state();
        (s.week_start, s.selected_session)
    };
    g.set_week_title(vm::week_title(week_start).into());

    let mut errors: Vec<String> = Vec::new();
    let mut still_selected = false;
    let mut course_ids = Vec::new();
    {
        let planner = app.planner();

        match planner.sessions_between(week_start, vm::week_end(week_start), now) {
            Ok(views) => {
                match bridge::rows(&vm::group_week(&views), bridge::day_group) {
                    Ok(days) => g.set_days(days),
                    Err(e) => errors.push(e),
                }
                if let Some(view) = selected_id.and_then(|id| find(&views, id)) {
                    match bridge::session_row(&vm::session_row(view)) {
                        Ok(row) => {
                            g.set_selected(row);
                            still_selected = true;
                        }
                        Err(e) => errors.push(e),
                    }
                }
            }
            Err(e) => errors.push(vm::error_text(&e)),
        }

        match planner.list_courses(false) {
            Ok(courses) => {
                let labels: Vec<String> = courses.iter().map(vm::course_option_label).collect();
                g.set_course_options(bridge::strings(&labels));
                course_ids = courses.iter().map(|c| c.id).collect();
            }
            Err(e) => errors.push(vm::error_text(&e)),
        }

        // Slots of archived courses still show their course code.
        let slots = planner
            .list_slots(today)
            .and_then(|slots| planner.list_courses(true).map(|courses| (slots, courses)));
        match slots {
            Ok((slots, courses)) => {
                let rows: Vec<vm::SlotRow> =
                    slots.iter().map(|s| vm::slot_row(s, &courses)).collect();
                match bridge::rows(&rows, bridge::slot_row) {
                    Ok(model) => g.set_slots(model),
                    Err(e) => errors.push(e),
                }
            }
            Err(e) => errors.push(vm::error_text(&e)),
        }
    }

    g.set_has_selected(still_selected);
    {
        let mut s = app.state();
        s.course_ids = course_ids;
        if !still_selected {
            s.selected_session = None;
        }
    }
    if let Some(first) = errors.first() {
        g.set_message(first.as_str().into());
    }
}

/// Connects the page callbacks.
pub fn wire(app: &Rc<App>) {
    let Some(window) = app.window() else {
        return;
    };
    let g = window.global::<TimetableState>();

    g.on_prev_week(with(app, |app, _| move_week(app, Some(-1))));
    g.on_next_week(with(app, |app, _| move_week(app, Some(1))));
    g.on_this_week(with(app, |app, _| move_week(app, None)));

    let weak = Rc::downgrade(app);
    g.on_select_session(move |id| {
        if let Some(app) = weak.upgrade() {
            select_session(&app, id);
        }
    });
    let weak = Rc::downgrade(app);
    g.on_set_status(move |id, action| {
        if let Some(app) = weak.upgrade() {
            set_status(&app, id, action);
        }
    });
    let weak = Rc::downgrade(app);
    g.on_reschedule(move |id| {
        if let Some(app) = weak.upgrade() {
            reschedule(&app, id);
        }
    });

    g.on_begin_add_slot(with(app, begin_add_slot));
    let weak = Rc::downgrade(app);
    g.on_begin_edit_slot(move |id| {
        if let Some(app) = weak.upgrade() {
            begin_edit_slot(&app, id);
        }
    });
    g.on_save_slot(with(app, save_slot));
    let weak = Rc::downgrade(app);
    g.on_begin_end_slot(move |id| {
        if let Some(app) = weak.upgrade() {
            begin_end_slot(&app, id);
        }
    });
    g.on_end_slot(with(app, end_slot));
}

/// A no-argument callback that runs `f` while the app is alive.
fn with(app: &Rc<App>, f: impl Fn(&App, &AppWindow) + 'static) -> impl Fn() + 'static {
    let weak: Weak<App> = Rc::downgrade(app);
    move || {
        if let Some(app) = weak.upgrade() {
            if let Some(window) = app.window() {
                f(&app, &window);
            }
        }
    }
}

fn find(views: &[SessionView], id: i64) -> Option<&SessionView> {
    views.iter().find(|v| v.session.id == id)
}

/// Moves the week by `weeks`, or back to the week of today; the selection is cleared.
fn move_week(app: &App, weeks: Option<i64>) {
    let today = app.today();
    {
        let mut s = app.state();
        s.week_start = match weeks {
            Some(n) => vm::shift_week(s.week_start, n),
            None => vm::week_start(today),
        };
        s.selected_session = None;
    }
    if let Some(window) = app.window() {
        window
            .global::<TimetableState>()
            .set_message(SharedString::new());
    }
    refresh(app);
}

/// Opens the inline panel of session `id` and pre-fills the reschedule form with its date, times,
/// and room (research R-D16). Selecting the open session again closes its panel.
fn select_session(app: &App, id: i32) {
    let Some(window) = app.window() else {
        return;
    };
    let g = window.global::<TimetableState>();
    g.set_message(SharedString::new());
    if app.state().selected_session == Some(i64::from(id)) {
        app.state().selected_session = None;
        refresh(app);
        return;
    }
    let Some(view) = week_session(app, i64::from(id), &g) else {
        return;
    };
    let s = &view.session;
    g.set_rs_date(vm::format_date_input(s.date).into());
    let (start_digits, start_pm) = vm::format_time_input(s.start_time);
    let (end_digits, end_pm) = vm::format_time_input(s.end_time);
    g.set_rs_start(start_digits.into());
    g.set_rs_start_pm(start_pm);
    g.set_rs_end(end_digits.into());
    g.set_rs_end_pm(end_pm);
    g.set_rs_room(s.room.clone().unwrap_or_default().into());
    g.set_rs_note(SharedString::new());
    clear_reschedule_errors(&g);
    app.state().selected_session = Some(s.id);
    refresh(app);
}

/// Session `id` of the shown week, or `None` with the error in `message`.
fn week_session(app: &App, id: i64, g: &TimetableState<'_>) -> Option<SessionView> {
    let week_start = app.state().week_start;
    let found = app
        .planner()
        .sessions_between(week_start, vm::week_end(week_start), app.now())
        .map(|views| find(&views, id).cloned());
    match found {
        Ok(Some(view)) => Some(view),
        Ok(None) => {
            refresh(app);
            None
        }
        Err(e) => {
            g.set_message(vm::error_text(&e).into());
            None
        }
    }
}

/// R8, R11, R12: sets the status of a session; core refuses Attended / Absent before the start.
/// No sync.
fn set_status(app: &App, id: i32, action: StatusAction) {
    let Some(window) = app.window() else {
        return;
    };
    let g = window.global::<TimetableState>();
    g.set_message(SharedString::new());
    let result = app
        .planner_mut()
        .set_status(i64::from(id), bridge::status(action), app.now());
    match result {
        Ok(()) => app.changed(false),
        Err(e) => g.set_message(vm::error_text(&e).into()),
    }
}

/// R9: reschedules session `id` with the form values and the session's course; core's error for
/// an attended or absent original is shown in `message`. No sync.
fn reschedule(app: &App, id: i32) {
    let Some(window) = app.window() else {
        return;
    };
    let g = window.global::<TimetableState>();
    g.set_message(SharedString::new());
    let Some(view) = week_session(app, i64::from(id), &g) else {
        return;
    };
    let parsed = vm::parse_makeup_form(
        view.session.course_id,
        &g.get_rs_date(),
        vm::TimeText {
            digits: &g.get_rs_start(),
            pm: g.get_rs_start_pm(),
        },
        vm::TimeText {
            digits: &g.get_rs_end(),
            pm: g.get_rs_end_pm(),
        },
        &g.get_rs_room(),
        &g.get_rs_note(),
    );
    let input = match parsed {
        Ok(input) => {
            clear_reschedule_errors(&g);
            input
        }
        Err(errors) => {
            g.set_rs_date_error(errors.get("date").into());
            g.set_rs_start_error(errors.get("start").into());
            g.set_rs_end_error(errors.get("end").into());
            return;
        }
    };
    let result = app.planner_mut().reschedule(view.session.id, input);
    match result {
        Ok(_) => {
            g.set_rs_note(SharedString::new());
            app.changed(false);
            app.toast_unless_error(&vm::rescheduled_text());
        }
        Err(e) => g.set_message(vm::error_text(&e).into()),
    }
}

fn clear_reschedule_errors(g: &TimetableState<'_>) {
    g.set_rs_date_error(SharedString::new());
    g.set_rs_start_error(SharedString::new());
    g.set_rs_end_error(SharedString::new());
}

fn clear_slot_errors(g: &TimetableState<'_>) {
    for set in [
        TimetableState::set_slot_course_error,
        TimetableState::set_slot_weekday_error,
        TimetableState::set_slot_start_error,
        TimetableState::set_slot_end_error,
        TimetableState::set_slot_from_error,
        TimetableState::set_slot_to_error,
        TimetableState::set_slot_effective_error,
        TimetableState::set_slot_form_error,
    ] {
        set(g, SharedString::new());
    }
}

/// Opens an empty slot form; valid from starts at the active term's start (research R-D16).
fn begin_add_slot(app: &App, window: &AppWindow) {
    let g = window.global::<TimetableState>();
    g.set_message(SharedString::new());
    let term_start = match active_term_start(app) {
        Ok(start) => start,
        Err(text) => {
            g.set_message(text.into());
            return;
        }
    };
    g.set_end_form_open(false);
    g.set_slot_editing(false);
    g.set_slot_editing_id(-1);
    g.set_slot_course(-1);
    g.set_slot_weekday(-1);
    g.set_slot_start(SharedString::new());
    g.set_slot_start_pm(false);
    g.set_slot_end(SharedString::new());
    g.set_slot_end_pm(false);
    g.set_slot_room(SharedString::new());
    g.set_slot_from(
        term_start
            .map(vm::format_date_input)
            .unwrap_or_default()
            .into(),
    );
    g.set_slot_to(SharedString::new());
    g.set_slot_effective(SharedString::new());
    // R56: a new slot is weekly until "Bi-weekly" is ticked; "Class this week" starts ticked.
    g.set_slot_biweekly_available(term_start.is_some());
    g.set_slot_biweekly(false);
    g.set_slot_class_this_week(true);
    clear_slot_errors(&g);
    g.set_slot_form_open(true);
}

/// R56: start date of the active term, or `None` without one (the parity needs it). A core error
/// is returned as display text.
fn active_term_start(app: &App) -> Result<Option<NaiveDate>, String> {
    app.planner()
        .active_term()
        .map(|term| term.map(|t| t.start_date))
        .map_err(|e| vm::error_text(&e))
}

/// Slot `id` among the slots valid today, or `None` with the error in `message`.
fn slot_valid_today(app: &App, id: i64, g: &TimetableState<'_>) -> Option<Slot> {
    let found = app
        .planner()
        .list_slots(app.today())
        .map(|slots| slots.into_iter().find(|s| s.id == id));
    match found {
        Ok(slot) => {
            if slot.is_none() {
                refresh(app);
            }
            slot
        }
        Err(e) => {
            g.set_message(vm::error_text(&e).into());
            None
        }
    }
}

/// Opens the slot form filled with slot `id`; the effective date starts at today (R7).
fn begin_edit_slot(app: &App, id: i32) {
    let Some(window) = app.window() else {
        return;
    };
    let g = window.global::<TimetableState>();
    g.set_message(SharedString::new());
    let Some(slot) = slot_valid_today(app, i64::from(id), &g) else {
        return;
    };
    let course_index = app
        .state()
        .course_ids
        .iter()
        .position(|c| *c == slot.course_id)
        .and_then(|i| i32::try_from(i).ok())
        .unwrap_or(-1);
    g.set_end_form_open(false);
    g.set_slot_editing(true);
    g.set_slot_editing_id(id);
    g.set_slot_course(course_index);
    g.set_slot_weekday(vm::weekday_index(slot.weekday));
    let (start_digits, start_pm) = vm::format_time_input(slot.start_time);
    let (end_digits, end_pm) = vm::format_time_input(slot.end_time);
    g.set_slot_start(start_digits.into());
    g.set_slot_start_pm(start_pm);
    g.set_slot_end(end_digits.into());
    g.set_slot_end_pm(end_pm);
    g.set_slot_room(slot.room.unwrap_or_default().into());
    g.set_slot_from(vm::format_date_input(slot.valid_from).into());
    g.set_slot_to(
        slot.valid_to
            .map(vm::format_date_input)
            .unwrap_or_default()
            .into(),
    );
    g.set_slot_effective(vm::format_date_input(app.today()).into());
    // R56: pre-fill "Bi-weekly" and "Class this week" from the stored parity and today. Without
    // an active term the boxes are hidden and the slot saves as weekly.
    let term_start = match active_term_start(app) {
        Ok(start) => start,
        Err(text) => {
            g.set_message(text.into());
            return;
        }
    };
    let (biweekly, class_this_week) = term_start.map_or((false, true), |start| {
        vm::slot_form_from_parity(start, app.today(), slot.week_parity)
    });
    g.set_slot_biweekly_available(term_start.is_some());
    g.set_slot_biweekly(biweekly);
    g.set_slot_class_this_week(class_this_week);
    clear_slot_errors(&g);
    g.set_slot_form_open(true);
}

/// Adds a slot, or when editing changes it from the effective date (R7); then syncs (R6). R56:
/// the slot's week parity comes from the bi-weekly boxes.
/// Field errors go under the fields, core R2 errors to the form error.
fn save_slot(app: &App, window: &AppWindow) {
    let g = window.global::<TimetableState>();
    g.set_message(SharedString::new());
    clear_slot_errors(&g);
    let course_id = usize::try_from(g.get_slot_course())
        .ok()
        .and_then(|i| app.state().course_ids.get(i).copied());
    // R56: the stored parity comes from the two boxes, today, and the active term's start.
    let term_start = match active_term_start(app) {
        Ok(start) => start,
        Err(text) => {
            g.set_slot_form_error(text.into());
            return;
        }
    };
    let week_parity = term_start.and_then(|start| {
        vm::slot_parity_from_form(
            start,
            app.today(),
            g.get_slot_biweekly(),
            g.get_slot_class_this_week(),
        )
    });
    let parsed = vm::parse_slot_form(
        course_id,
        g.get_slot_weekday(),
        vm::TimeText {
            digits: &g.get_slot_start(),
            pm: g.get_slot_start_pm(),
        },
        vm::TimeText {
            digits: &g.get_slot_end(),
            pm: g.get_slot_end_pm(),
        },
        &g.get_slot_room(),
        &g.get_slot_from(),
        &g.get_slot_to(),
        week_parity,
    );
    let editing = g.get_slot_editing();
    let effective = if editing {
        match vm::parse_date(&g.get_slot_effective()) {
            Ok(date) => Some(date),
            Err(e) => {
                g.set_slot_effective_error(e.into());
                None
            }
        }
    } else {
        None
    };
    let input = match parsed {
        Ok(input) => input,
        Err(errors) => {
            g.set_slot_course_error(errors.get("course").into());
            g.set_slot_weekday_error(errors.get("weekday").into());
            g.set_slot_start_error(errors.get("start").into());
            g.set_slot_end_error(errors.get("end").into());
            g.set_slot_from_error(errors.get("valid_from").into());
            g.set_slot_to_error(errors.get("valid_to").into());
            return;
        }
    };
    let result = match (editing, effective) {
        (false, _) => app.planner_mut().add_slot(input).map(|_| ()),
        (true, Some(effective)) => app
            .planner_mut()
            .change_slot(i64::from(g.get_slot_editing_id()), input, effective)
            .map(|_| ()),
        (true, None) => return,
    };
    match result {
        Ok(()) => {
            g.set_slot_form_open(false);
            app.changed(true);
            app.toast_unless_error(&vm::slot_saved_text(!editing));
        }
        Err(e) => g.set_slot_form_error(vm::error_text(&e).into()),
    }
}

/// Opens the end form for slot `id`; the last date starts at today (research R-D16).
fn begin_end_slot(app: &App, id: i32) {
    let Some(window) = app.window() else {
        return;
    };
    let g = window.global::<TimetableState>();
    g.set_message(SharedString::new());
    g.set_slot_form_open(false);
    g.set_end_slot_id(id);
    g.set_end_date(vm::format_date_input(app.today()).into());
    g.set_end_date_error(SharedString::new());
    g.set_end_form_error(SharedString::new());
    g.set_end_form_open(true);
}

/// R5, R7: makes the date the slot's last valid date; core syncs and removes later pending
/// regular sessions. Then syncs with today (R6).
fn end_slot(app: &App, window: &AppWindow) {
    let g = window.global::<TimetableState>();
    g.set_message(SharedString::new());
    g.set_end_form_error(SharedString::new());
    let last_date: NaiveDate = match vm::parse_date(&g.get_end_date()) {
        Ok(date) => {
            g.set_end_date_error(SharedString::new());
            date
        }
        Err(e) => {
            g.set_end_date_error(e.into());
            return;
        }
    };
    let result = app
        .planner_mut()
        .end_slot(i64::from(g.get_end_slot_id()), last_date);
    match result {
        Ok(()) => {
            g.set_end_form_open(false);
            app.changed(true);
            app.toast_unless_error(&vm::slot_ended_text());
        }
        Err(e) => g.set_end_form_error(vm::error_text(&e).into()),
    }
}
