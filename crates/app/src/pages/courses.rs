//! Courses page: fills `CoursesState` (contracts/ui-bridge.md "CoursesState"). Course list with
//! the show-archived toggle (R3), create / edit form (R2), archive / unarchive (R3), and the
//! course detail: attendance summary (R13–R15), every session with status actions (R11, R12), and
//! "Add makeup class" (R10). Course changes sync sessions (R6); status and makeup changes do not.

use std::rc::{Rc, Weak};

use slint::{ComponentHandle, SharedString};

use crate::controller::App;
use crate::viewmodel::{self as vm, FormErrors};
use crate::{bridge, CoursesState, StatusAction};

/// `editing-id` when the form creates a new course.
const NEW_COURSE: i32 = -1;

/// Reloads the course list and, when open, the course detail (R3, R13–R15).
pub fn refresh(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let ui = window.global::<CoursesState>();
    let (show_archived, detail) = {
        let state = app.state();
        (state.show_archived, state.detail_course)
    };
    ui.set_show_archived(show_archived);
    if let Err(text) = fill(app, &ui, show_archived, detail) {
        ui.set_message(text.into());
    }
}

/// R3: the list honours `show_archived`; the detail of an archived course stays reachable
/// (history is kept). A detail course that no longer exists closes the panel.
fn fill(
    app: &App,
    ui: &CoursesState<'_>,
    show_archived: bool,
    detail: Option<i64>,
) -> Result<(), String> {
    let planner = app.planner();
    let courses = planner.list_courses(show_archived).map_err(err)?;
    let rows: Vec<_> = courses.iter().map(vm::course_row).collect();
    ui.set_courses(bridge::rows(&rows, bridge::course_row)?);

    let course = match detail {
        Some(id) => planner
            .list_courses(true)
            .map_err(err)?
            .into_iter()
            .find(|c| c.id == id),
        None => None,
    };
    let Some(course) = course else {
        ui.set_detail_open(false);
        drop(planner);
        app.state().detail_course = None;
        return Ok(());
    };

    let summary = planner.attendance_summary(course.id).map_err(err)?;
    let sessions = planner
        .sessions_for_course(course.id, app.now())
        .map_err(err)?;
    let session_rows: Vec<_> = sessions.iter().map(vm::session_row).collect();
    ui.set_detail_title(vm::course_option_label(&course).into());
    ui.set_summary(bridge::summary_view(&vm::summary_view(&summary)));
    ui.set_sessions(bridge::rows(&session_rows, bridge::session_row)?);
    ui.set_detail_open(true);
    Ok(())
}

/// Connects the page callbacks.
pub fn wire(app: &Rc<App>) {
    let Some(window) = app.window() else {
        return;
    };
    let ui = window.global::<CoursesState>();
    ui.set_below_target_label(vm::BELOW_TARGET_LABEL.into());

    let weak = Rc::downgrade(app);
    ui.on_toggle_archived(move |show| {
        with(&weak, |app, _| {
            app.state().show_archived = show;
            refresh(app);
        })
    });

    let weak = Rc::downgrade(app);
    ui.on_begin_create(move || with(&weak, begin_create));

    let weak = Rc::downgrade(app);
    ui.on_begin_edit(move |id| with(&weak, |app, ui| begin_edit(app, ui, id)));

    let weak = Rc::downgrade(app);
    ui.on_save_course(move || with(&weak, save_course));

    let weak = Rc::downgrade(app);
    ui.on_set_archived(move |id, archived| {
        with(&weak, |app, ui| {
            // R3: archiving keeps sessions and history; R6: sync afterwards.
            let result = app.planner_mut().set_archived(i64::from(id), archived);
            after(app, ui, result, true);
        })
    });

    let weak = Rc::downgrade(app);
    ui.on_open_detail(move |id| {
        with(&weak, |app, ui| {
            app.state().detail_course = Some(i64::from(id));
            clear_makeup_form(ui);
            ui.set_message(SharedString::new());
            refresh(app);
        })
    });

    let weak = Rc::downgrade(app);
    ui.on_close_detail(move || {
        with(&weak, |app, _| {
            app.state().detail_course = None;
            refresh(app);
        })
    });

    let weak = Rc::downgrade(app);
    ui.on_set_status(move |id, action: StatusAction| {
        with(&weak, |app, ui| {
            // R11, R12: core refuses Attended / Absent before the session starts. No sync.
            let now = app.now();
            let result = app
                .planner_mut()
                .set_status(i64::from(id), bridge::status(action), now);
            after(app, ui, result, false);
        })
    });

    let weak = Rc::downgrade(app);
    ui.on_add_makeup(move || with(&weak, add_makeup));
}

/// Runs `f` with the app and the page global if both are still alive.
fn with(weak: &Weak<App>, f: impl FnOnce(&App, &CoursesState<'_>)) {
    let Some(app) = weak.upgrade() else {
        return;
    };
    let Some(window) = app.window() else {
        return;
    };
    f(&app, &window.global::<CoursesState>());
}

/// Core error as display text.
fn err(e: planner_core::Error) -> String {
    vm::error_text(&e)
}

/// Row action result: an error goes to `message`; success clears it and refreshes, syncing first
/// when `sync` is set (R6).
fn after<T>(app: &App, ui: &CoursesState<'_>, result: planner_core::Result<T>, sync: bool) {
    match result {
        Ok(_) => {
            ui.set_message(SharedString::new());
            app.changed(sync);
        }
        Err(e) => ui.set_message(err(e).into()),
    }
}

fn clear_course_errors(ui: &CoursesState<'_>) {
    ui.set_target_error(SharedString::new());
    ui.set_course_form_error(SharedString::new());
}

fn clear_makeup_form(ui: &CoursesState<'_>) {
    for set in [
        CoursesState::set_mk_date,
        CoursesState::set_mk_start,
        CoursesState::set_mk_end,
        CoursesState::set_mk_room,
        CoursesState::set_mk_note,
        CoursesState::set_mk_date_error,
        CoursesState::set_mk_start_error,
        CoursesState::set_mk_end_error,
        CoursesState::set_mk_form_error,
    ] {
        set(ui, SharedString::new());
    }
    ui.set_mk_start_pm(false);
    ui.set_mk_end_pm(false);
}

/// Opens an empty form for a new course.
fn begin_create(_app: &App, ui: &CoursesState<'_>) {
    for set in [
        CoursesState::set_code,
        CoursesState::set_name,
        CoursesState::set_instructor,
        CoursesState::set_room,
        CoursesState::set_notes,
        CoursesState::set_target,
    ] {
        set(ui, SharedString::new());
    }
    clear_course_errors(ui);
    ui.set_editing_id(NEW_COURSE);
    ui.set_form_open(true);
}

/// Opens the form filled with the stored values of course `id`.
fn begin_edit(app: &App, ui: &CoursesState<'_>, id: i32) {
    let found = app
        .planner()
        .list_courses(true)
        .map(|all| all.into_iter().find(|c| c.id == i64::from(id)));
    let course = match found {
        Ok(Some(course)) => course,
        Ok(None) => return refresh(app),
        Err(e) => return ui.set_message(err(e).into()),
    };
    ui.set_code(course.code.into());
    ui.set_name(course.name.into());
    ui.set_instructor(course.instructor.unwrap_or_default().into());
    ui.set_room(course.room.unwrap_or_default().into());
    ui.set_notes(course.notes.unwrap_or_default().into());
    ui.set_target(vm::target_input(course.min_attendance_pct).into());
    clear_course_errors(ui);
    ui.set_editing_id(id);
    ui.set_form_open(true);
}

/// R2: parses the form, then creates or updates the course; core validation errors are shown
/// under the form. R6: sync after a successful save.
fn save_course(app: &App, ui: &CoursesState<'_>) {
    let parsed = vm::parse_course_form(
        &ui.get_code(),
        &ui.get_name(),
        &ui.get_instructor(),
        &ui.get_room(),
        &ui.get_notes(),
        &ui.get_target(),
    );
    let input = match parsed {
        Ok(input) => input,
        Err(errors) => {
            ui.set_target_error(errors.get("target").into());
            ui.set_course_form_error(SharedString::new());
            return;
        }
    };
    let editing = ui.get_editing_id();
    let result = if editing == NEW_COURSE {
        app.planner_mut().create_course(input)
    } else {
        app.planner_mut().update_course(i64::from(editing), input)
    };
    match result {
        Ok(_) => {
            clear_course_errors(ui);
            ui.set_form_open(false);
            ui.set_message(SharedString::new());
            app.changed(true);
        }
        Err(e) => {
            ui.set_target_error(SharedString::new());
            ui.set_course_form_error(err(e).into());
        }
    }
}

fn set_makeup_errors(ui: &CoursesState<'_>, errors: &FormErrors) {
    ui.set_mk_date_error(errors.get("date").into());
    ui.set_mk_start_error(errors.get("start").into());
    ui.set_mk_end_error(errors.get("end").into());
}

/// R10: adds a one-off makeup class to the open course. Core errors (e.g. end not after start)
/// are shown under the form. No sync: makeups are not generated sessions.
fn add_makeup(app: &App, ui: &CoursesState<'_>) {
    let Some(course_id) = app.state().detail_course else {
        return;
    };
    let parsed = vm::parse_makeup_form(
        course_id,
        &ui.get_mk_date(),
        vm::TimeText {
            digits: &ui.get_mk_start(),
            pm: ui.get_mk_start_pm(),
        },
        vm::TimeText {
            digits: &ui.get_mk_end(),
            pm: ui.get_mk_end_pm(),
        },
        &ui.get_mk_room(),
        &ui.get_mk_note(),
    );
    let input = match parsed {
        Ok(input) => input,
        Err(errors) => {
            set_makeup_errors(ui, &errors);
            ui.set_mk_form_error(SharedString::new());
            return;
        }
    };
    set_makeup_errors(ui, &FormErrors::default());
    let result = app.planner_mut().add_makeup(input);
    match result {
        Ok(_) => {
            clear_makeup_form(ui);
            ui.set_message(SharedString::new());
            app.changed(false);
        }
        Err(e) => ui.set_mk_form_error(err(e).into()),
    }
}
