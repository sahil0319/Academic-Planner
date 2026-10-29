//! Assessments page: fills `AssessmentsState` (contracts/ui-bridge.md, FR-020). Lists assessments
//! with the upcoming / overdue / completed / all filters (R18, R19) and saves the create / edit
//! form through `parse_assessment_form` (FR-007) and core, which owns the R2 checks. Assessment
//! changes do not affect sessions, so they never sync (R6 is only for term, holiday, course, and
//! slot changes).

use std::rc::{Rc, Weak};

use planner_core::{Assessment, AssessmentFilter, Course};
use slint::{ComponentHandle, SharedString};

use crate::controller::App;
use crate::viewmodel::{self as vm, FormErrors};
use crate::{bridge, AssessmentsState};

/// Combo and form id meaning "nothing chosen" / "creating a new assessment".
const NONE: i32 = -1;

/// Reloads the list for the current filter with a fresh `now` (R18, R19) and the course and kind
/// options. Clears the row-action message; the form keeps its contents.
pub fn refresh(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let st = window.global::<AssessmentsState>();
    st.set_message(SharedString::new());

    let kinds: Vec<String> = (0..)
        .map_while(vm::assessment_kind_from_index)
        .map(|k| vm::assessment_kind_label(k).to_string())
        .collect();
    st.set_kind_options(bridge::strings(&kinds));

    let choice = app.state().filter;
    st.set_list_title(vm::filter_title(choice).into());
    st.set_empty_text(vm::filter_empty_text(choice).into());
    let mut problems = Vec::new();

    let listed = app
        .planner()
        .list_assessments(vm::assessment_filter(choice, app.now()));
    let rows: Vec<vm::AssessmentRow> = match listed {
        Ok(list) => list.iter().map(vm::assessment_row).collect(),
        Err(e) => {
            problems.push(vm::error_text(&e));
            Vec::new()
        }
    };
    match bridge::rows(&rows, bridge::assessment_row) {
        Ok(model) => st.set_rows(model),
        Err(e) => {
            st.set_rows(bridge::model(Vec::new()));
            problems.push(e);
        }
    }

    // Keep the chosen course across refreshes by id, not combo index: the course list may have
    // changed (created, archived) while the form was open.
    let chosen = selected_course_id(app, st.get_course());
    let keep = chosen.filter(|_| st.get_editing_id() >= 0);
    match fill_course_options(app, &st, keep) {
        Ok(()) => st.set_course(course_index(app, chosen)),
        Err(e) => problems.push(e),
    }

    if !problems.is_empty() {
        st.set_message(problems.join("\n").into());
    }
}

/// Connects the page callbacks. Each mutation ends in `changed(false)`: assessments do not
/// touch sessions, so no sync is needed (R6).
pub fn wire(app: &Rc<App>) {
    let Some(window) = app.window() else {
        return;
    };
    let st = window.global::<AssessmentsState>();

    let weak = Rc::downgrade(app);
    st.on_set_filter(move |index| {
        with_app(&weak, |app| {
            app.state().filter = vm::filter_from_index(index);
            if let Some(window) = app.window() {
                window.global::<AssessmentsState>().set_filter(index);
            }
            refresh(app);
        });
    });

    let weak = Rc::downgrade(app);
    st.on_begin_create(move || with_app(&weak, clear_form));

    let weak = Rc::downgrade(app);
    st.on_begin_edit(move |id| with_app(&weak, |app| begin_edit(app, id)));

    let weak = Rc::downgrade(app);
    st.on_save(move || with_app(&weak, save));

    let weak = Rc::downgrade(app);
    st.on_set_completed(move |id, done| {
        with_app(&weak, |app| {
            let result = app.planner_mut().set_completed(i64::from(id), done);
            finish_row_action(app, result);
        });
    });

    let weak = Rc::downgrade(app);
    st.on_delete(move |id| {
        with_app(&weak, |app| {
            let result = app.planner_mut().delete_assessment(i64::from(id));
            if result.is_ok() {
                if let Some(window) = app.window() {
                    if window.global::<AssessmentsState>().get_editing_id() == id {
                        clear_form(app);
                    }
                }
            }
            finish_row_action(app, result);
        });
    });
}

/// Runs `f` if the app is still alive.
fn with_app(weak: &Weak<App>, f: impl FnOnce(&App)) {
    if let Some(app) = weak.upgrade() {
        f(&app);
    }
}

/// Courses offered in the form's course combo, in combo index order.
fn active_courses(app: &App) -> Result<Vec<Course>, String> {
    app.planner()
        .list_courses(false)
        .map_err(|e| vm::error_text(&e))
}

/// Sets the course combo to the active courses plus, when `keep` names an archived course, that
/// course at the end, so an assessment of an archived course (R3) can be saved unchanged. Stores
/// the index-to-id map in `UiState::assessment_course_ids`.
fn fill_course_options(
    app: &App,
    st: &AssessmentsState<'_>,
    keep: Option<i64>,
) -> Result<(), String> {
    let mut courses = active_courses(app)?;
    if let Some(id) = keep {
        if !courses.iter().any(|c| c.id == id) {
            let all = app
                .planner()
                .list_courses(true)
                .map_err(|e| vm::error_text(&e))?;
            courses.extend(all.into_iter().filter(|c| c.id == id));
        }
    }
    let labels: Vec<String> = courses.iter().map(vm::course_option_label).collect();
    st.set_course_options(bridge::strings(&labels));
    app.state().assessment_course_ids = courses.iter().map(|c| c.id).collect();
    Ok(())
}

/// Course id behind combo `index` in the current options, if any.
fn selected_course_id(app: &App, index: i32) -> Option<i64> {
    let i = usize::try_from(index).ok()?;
    app.state().assessment_course_ids.get(i).copied()
}

/// Combo index of course `id` in the current options, or `NONE`.
fn course_index(app: &App, id: Option<i64>) -> i32 {
    id.and_then(|id| {
        app.state()
            .assessment_course_ids
            .iter()
            .position(|&c| c == id)
    })
    .and_then(|i| i32::try_from(i).ok())
    .unwrap_or(NONE)
}

/// After a row action: on success refreshes every page; on error refreshes this page (so a
/// toggled checkbox shows the stored value again) and shows the error.
fn finish_row_action(app: &App, result: planner_core::Result<()>) {
    match result {
        Ok(()) => app.changed(false),
        Err(e) => {
            refresh(app);
            app.show_message(&vm::error_text(&e));
        }
    }
}

/// Empties the form and switches it to "new assessment".
fn clear_form(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let st = window.global::<AssessmentsState>();
    st.set_editing_id(NONE);
    st.set_course(NONE);
    st.set_kind(NONE);
    st.set_title(SharedString::new());
    st.set_due_date(SharedString::new());
    st.set_due_time(SharedString::new());
    st.set_due_time_pm(false);
    st.set_notes(SharedString::new());
    st.set_remind(SharedString::new());
    // A new assessment offers active courses only; drop an archived course kept for editing.
    let problem = fill_course_options(app, &st, None)
        .err()
        .unwrap_or_default();
    show_errors(&st, &FormErrors::default(), &problem);
}

/// Fills the form with the stored assessment `id`. Its course is shown even when archived (R3),
/// so saving unchanged works.
fn begin_edit(app: &App, id: i32) {
    let Some(window) = app.window() else {
        return;
    };
    let st = window.global::<AssessmentsState>();
    let found = find_assessment(app, i64::from(id))
        .and_then(|a| fill_course_options(app, &st, Some(a.course_id)).map(|()| a));
    let assessment = match found {
        Ok(found) => found,
        Err(e) => {
            st.set_message(e.into());
            return;
        }
    };

    st.set_editing_id(id);
    st.set_course(course_index(app, Some(assessment.course_id)));
    st.set_kind(vm::assessment_kind_index(assessment.kind));
    st.set_title(assessment.title.as_str().into());
    st.set_due_date(vm::format_date_input(assessment.due_at.date()).into());
    let (time_digits, time_pm) = vm::format_time_input(assessment.due_at.time());
    st.set_due_time(time_digits.into());
    st.set_due_time_pm(time_pm);
    st.set_notes(assessment.notes.unwrap_or_default().into());
    st.set_remind(
        assessment
            .remind_minutes_before
            .map(|m| m.to_string())
            .unwrap_or_default()
            .into(),
    );
    show_errors(&st, &FormErrors::default(), "");
}

/// The stored assessment `id`. Core has no single-assessment getter, so the full list is
/// searched.
fn find_assessment(app: &App, id: i64) -> Result<Assessment, String> {
    let all = app
        .planner()
        .list_assessments(AssessmentFilter::All)
        .map_err(|e| vm::error_text(&e))?;
    all.into_iter()
        .find(|a| a.id == id)
        .ok_or_else(|| "This assessment no longer exists".to_string())
}

/// FR-007 field checks, then create or update; R2 errors from core are shown as the form error
/// and nothing is saved.
fn save(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let st = window.global::<AssessmentsState>();

    let course_id = selected_course_id(app, st.get_course());

    let input = match vm::parse_assessment_form(
        course_id,
        st.get_kind(),
        &st.get_title(),
        &st.get_due_date(),
        vm::TimeText {
            digits: &st.get_due_time(),
            pm: st.get_due_time_pm(),
        },
        &st.get_notes(),
        &st.get_remind(),
        None,
        "",
    ) {
        Ok(input) => input,
        Err(errors) => {
            show_errors(&st, &errors, "");
            return;
        }
    };

    let editing = st.get_editing_id();
    let result = if editing >= 0 {
        app.planner_mut()
            .update_assessment(i64::from(editing), input)
    } else {
        app.planner_mut().create_assessment(input)
    };
    match result {
        Ok(_) => {
            clear_form(app);
            app.changed(false);
        }
        Err(e) => show_errors(&st, &FormErrors::default(), &vm::error_text(&e)),
    }
}

/// Shows the field errors and the form-level error (FR-007, R2).
fn show_errors(st: &AssessmentsState<'_>, errors: &FormErrors, form_error: &str) {
    st.set_course_error(errors.get("course").into());
    st.set_kind_error(errors.get("kind").into());
    st.set_due_date_error(errors.get("due_date").into());
    st.set_due_time_error(errors.get("due_time").into());
    st.set_remind_error(errors.get("remind").into());
    st.set_form_error(form_error.into());
}
