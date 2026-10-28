//! Dashboard page: fills `DashboardState` (contracts/ui-bridge.md "DashboardState").

use std::rc::Rc;

use slint::{ComponentHandle, SharedString};

use crate::bridge;
use crate::controller::App;
use crate::viewmodel::{self, DashboardView};
use crate::{DashboardState, SessionRow};

/// R1, R14, R16–R20: reloads the dashboard relative to a fresh `now`. A core error or an id that
/// does not fit a Slint `int` is shown in the page's `message`.
pub fn refresh(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let state = window.global::<DashboardState>();
    let now = app.now();
    // T028: without an active term the page shows one "No term yet" card instead of the sections.
    match app.planner().active_term() {
        Ok(term) => state.set_has_term(term.is_some()),
        Err(e) => {
            state.set_message(viewmodel::error_text(&e).into());
            return;
        }
    }
    let loaded = app.planner().dashboard(now);
    let view = match loaded {
        Ok(d) => viewmodel::dashboard_view(&d),
        Err(e) => {
            state.set_message(viewmodel::error_text(&e).into());
            return;
        }
    };
    if let Err(text) = fill(&state, &view) {
        state.set_message(text.into());
    }
}

/// Copies the view into the Slint global; stops at the first id that does not fit.
fn fill(state: &DashboardState<'_>, view: &DashboardView) -> Result<(), String> {
    state.set_banner(bridge::strings(&view.banner));
    state.set_today(bridge::rows(&view.today, bridge::session_row)?);
    // R16, R17: hide Next class when that session is already in Today's classes.
    state.set_show_next(viewmodel::show_next_card(
        view.next_class.as_ref(),
        &view.today,
    ));
    // R17: no next class hides the row and shows the empty state.
    match &view.next_class {
        Some(row) => {
            state.set_next_class(bridge::session_row(row)?);
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

/// Connects the awaiting-attendance row buttons.
pub fn wire(app: &Rc<App>) {
    let Some(window) = app.window() else {
        return;
    };
    let weak = Rc::downgrade(app);
    window
        .global::<DashboardState>()
        .on_set_status(move |id, action| {
            if let Some(app) = weak.upgrade() {
                set_status(&app, id, bridge::status(action));
            }
        });
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

/// Sets the dashboard `message`.
fn show(app: &App, text: &str) {
    if let Some(window) = app.window() {
        window
            .global::<DashboardState>()
            .set_message(SharedString::from(text));
    }
}
