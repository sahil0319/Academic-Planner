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
    // R40: term GPA in the page header.
    ui.set_gpa_text(vm::gpa_header_text(&planner.term_gpa().map_err(err)?).into());
    // R33: the rule set choice offers "Not graded" and the current rule sets.
    let names: Vec<String> = planner
        .list_rule_sets()
        .map_err(err)?
        .into_iter()
        .map(|s| s.name)
        .collect();
    let labels = vm::choice_labels(vm::NOT_GRADED_LABEL, &names);
    if usize::try_from(ui.get_rule_set()).map_or(true, |i| i >= labels.len()) {
        ui.set_rule_set(0);
    }
    ui.set_rule_set_options(bridge::strings(&labels));

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

    // R37, R38: marks card and needed-in-final table of the open course.
    let marks = vm::marks_view(&planner.course_marks(course.id).map_err(err)?);
    ui.set_graded(marks.graded);
    ui.set_not_graded_line(vm::NOT_GRADED_LINE.into());
    ui.set_mark_rows(bridge::model(
        marks.rows.iter().map(bridge::mark_row).collect(),
    ));
    ui.set_marks_secured(marks.secured.into());
    ui.set_marks_projected(marks.projected.into());
    ui.set_needed_caption(marks.needed_caption.into());
    ui.set_needed_rows(bridge::model(
        marks.needed.iter().map(bridge::needed_row).collect(),
    ));
    ui.set_needed_assumptions(bridge::strings(&marks.assumptions));
    ui.set_actual_grade(marks.actual.into());

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
    ui.set_credits_error(SharedString::new());
    ui.set_course_form_error(SharedString::new());
}

/// R33: ids of the rule sets in choice order (choice `i + 1` is `ids[i]`; 0 is "Not graded").
fn rule_set_ids(app: &App) -> Result<Vec<i64>, String> {
    let sets = app.planner().list_rule_sets().map_err(err)?;
    Ok(sets.into_iter().map(|s| s.id).collect())
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
        CoursesState::set_credits,
    ] {
        set(ui, SharedString::new());
    }
    ui.set_rule_set(0);
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
    ui.set_credits(vm::number_input(course.credits).into());
    match rule_set_ids(app) {
        Ok(ids) => ui.set_rule_set(vm::choice_index(course.rule_set_id, &ids)),
        Err(text) => return ui.set_message(text.into()),
    }
    clear_course_errors(ui);
    ui.set_editing_id(id);
    ui.set_form_open(true);
}

/// R2, R33: parses the form (credits and rule set included), then creates or updates the course; core validation errors are shown
/// under the form. R6: sync after a successful save.
fn save_course(app: &App, ui: &CoursesState<'_>) {
    let rule_set_id = match rule_set_ids(app) {
        Ok(ids) => vm::choice_id(ui.get_rule_set(), &ids),
        Err(text) => return ui.set_course_form_error(text.into()),
    };
    let parsed = vm::parse_course_form(
        &ui.get_code(),
        &ui.get_name(),
        &ui.get_instructor(),
        &ui.get_room(),
        &ui.get_notes(),
        &ui.get_target(),
        &ui.get_credits(),
        rule_set_id,
    );
    let input = match parsed {
        Ok(input) => input,
        Err(errors) => {
            ui.set_target_error(errors.get("target").into());
            ui.set_credits_error(errors.get("credits").into());
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
            ui.set_credits_error(SharedString::new());
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
