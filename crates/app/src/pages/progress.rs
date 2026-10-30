//! Progress page: fills `ProgressState` (R61–R65). Read-only, so it has `refresh` and no `wire`.

use slint::ComponentHandle;

use crate::bridge;
use crate::controller::App;
use crate::viewmodel::{self, ProgressView};
use crate::ProgressState;

/// R61–R65: reloads the cards of the active term's non-archived courses (by code, R3, R15) and
/// the header line. A core error is shown in the page's `message`. The planner borrow ends in
/// [`load`], before any Slint setter runs.
pub fn refresh(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let state = window.global::<ProgressState>();
    match load(app) {
        Ok(Some(view)) => {
            state.set_has_term(true);
            state.set_message("".into());
            state.set_empty_text(viewmodel::progress_empty_text(true).into());
            state.set_gpa_text(view.gpa_text.as_str().into());
            state.set_counts_text(view.counts_text.as_str().into());
            state.set_cards(bridge::model(
                view.cards.iter().map(bridge::progress_card).collect(),
            ));
        }
        Ok(None) => {
            state.set_has_term(false);
            state.set_message("".into());
            state.set_empty_text(viewmodel::progress_empty_text(false).into());
            state.set_cards(bridge::model(Vec::new()));
        }
        Err(e) => state.set_message(viewmodel::error_text(&e).into()),
    }
}

/// R61: the page data, or `None` when there is no active term.
fn load(app: &App) -> planner_core::Result<Option<ProgressView>> {
    let planner = app.planner();
    if planner.active_term()?.is_none() {
        return Ok(None);
    }
    let gpa = planner.term_gpa()?;
    let scale = planner.grade_scale()?;
    let mut cards = Vec::new();
    for summary in planner.all_summaries()? {
        let marks = planner.course_marks(summary.course_id)?;
        cards.push(viewmodel::progress_card(&summary, &marks, &scale));
    }
    Ok(Some(viewmodel::progress_view(&gpa, cards)))
}
