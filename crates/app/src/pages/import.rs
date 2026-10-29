//! Import panel of the Courses page: fills `ImportState` (contracts/ui-import.md). Preview calls
//! core's `preview_import` and shows the plan or the errors with their paths (R42-R48); Apply is
//! enabled only by `can_apply` (R48, D19) and runs `apply_import`, then the usual changed/sync/
//! refresh path so Courses, Timetable, and Dashboard update. The effective date is `DD-MM-YYYY`,
//! pre-filled with today (R49). All text comes from `viewmodel::import`.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

use slint::{ComponentHandle, SharedString};

use crate::controller::App;
use crate::viewmodel::import::{self as imp, PreviewState};
use crate::viewmodel::{self as vm};
use crate::{bridge, ImportState};

/// R50: the template shipped in `docs/import`, put on the clipboard by Copy template.
const TEMPLATE: &str = include_str!("../../../../docs/import/template.json");
/// R50: the AI prompt shipped in `docs/import`, put on the clipboard by Copy AI prompt.
const AI_PROMPT: &str = include_str!("../../../../docs/import/ai-prompt.md");

/// What the panel remembers between callbacks: the last preview (D19).
type Shared = Rc<RefCell<Option<PreviewState>>>;

/// Connects the panel callbacks and loads the embedded texts.
pub fn wire(app: &Rc<App>) {
    let Some(window) = app.window() else {
        return;
    };
    let ui = window.global::<ImportState>();
    ui.set_template_text(TEMPLATE.into());
    ui.set_prompt_text(AI_PROMPT.into());
    reset(app, &ui);

    let preview: Shared = Rc::default();

    let weak = Rc::downgrade(app);
    let shared = preview.clone();
    ui.on_close(move || {
        with(&weak, |app, ui| {
            *shared.borrow_mut() = None;
            ui.set_open(false);
            reset(app, ui);
        })
    });

    let weak = Rc::downgrade(app);
    let shared = preview.clone();
    ui.on_text_edited(move |_| with(&weak, |_, ui| edited(ui, &shared)));

    let weak = Rc::downgrade(app);
    let shared = preview.clone();
    ui.on_date_edited(move |_| with(&weak, |_, ui| edited(ui, &shared)));

    let weak = Rc::downgrade(app);
    let shared = preview.clone();
    ui.on_preview(move || with(&weak, |app, ui| run_preview(app, ui, &shared)));

    let weak = Rc::downgrade(app);
    let shared = preview;
    ui.on_apply(move || with(&weak, |app, ui| run_apply(app, ui, &shared)));

    let weak = Rc::downgrade(app);
    ui.on_template_copied(move || with(&weak, |_, ui| ui.set_copy_status(TEMPLATE_COPIED.into())));
    let weak = Rc::downgrade(app);
    ui.on_prompt_copied(move || with(&weak, |_, ui| ui.set_copy_status(PROMPT_COPIED.into())));
}

/// R49: while the panel is closed the date field follows today (also after a date rollover).
pub fn refresh(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let ui = window.global::<ImportState>();
    if !ui.get_open() {
        reset(app, &ui);
    }
}

/// Empties the panel and pre-fills the effective date with today (R49).
fn reset(app: &App, ui: &ImportState<'_>) {
    ui.set_text(SharedString::new());
    ui.set_date_text(vm::format_date_input(app.today()).into());
    ui.set_date_error(SharedString::new());
    ui.set_lines(bridge::model(Vec::new()));
    ui.set_message(SharedString::new());
    ui.set_copy_status(SharedString::new());
    update_gates(ui, None);
}

/// Status line after Copy template.
const TEMPLATE_COPIED: &str = "Template copied";
/// Status line after Copy AI prompt.
const PROMPT_COPIED: &str = "AI prompt copied";

/// Runs `f` with the app and the page global if both are still alive.
fn with(weak: &Weak<App>, f: impl FnOnce(&App, &ImportState<'_>)) {
    let Some(app) = weak.upgrade() else {
        return;
    };
    let Some(window) = app.window() else {
        return;
    };
    f(&app, &window.global::<ImportState>());
}

/// D19, R49: Preview needs non-blank text and a valid date; Apply needs a matching preview.
fn update_gates(ui: &ImportState<'_>, preview: Option<&PreviewState>) {
    let text = ui.get_text();
    let date_text = ui.get_date_text();
    let date_ok = vm::parse_date(&date_text).is_ok();
    ui.set_can_preview(date_ok && !text.trim().is_empty());
    ui.set_can_apply(date_ok && imp::can_apply(preview, &text, &date_text));
}

/// Editing the text or the date clears the shown preview (D19) and re-checks the date (R49).
fn edited(ui: &ImportState<'_>, preview: &Shared) {
    *preview.borrow_mut() = None;
    ui.set_lines(bridge::model(Vec::new()));
    ui.set_message(SharedString::new());
    ui.set_copy_status(SharedString::new());
    let error = vm::parse_date(&ui.get_date_text())
        .err()
        .unwrap_or_default();
    ui.set_date_error(error.into());
    update_gates(ui, None);
}

fn run_preview(app: &App, ui: &ImportState<'_>, preview: &Shared) {
    let text = ui.get_text().to_string();
    let date_text = ui.get_date_text().to_string();
    let Ok(date) = vm::parse_date(&date_text) else {
        return;
    };
    ui.set_message(SharedString::new());
    ui.set_copy_status(SharedString::new());
    let result = app.planner().preview_import(&text, date);
    match result {
        Ok(plan) => {
            let state = PreviewState {
                text,
                date_text,
                has_errors: !plan.errors.is_empty(),
                is_empty: plan.is_empty(),
            };
            let lines: Vec<_> = imp::preview_lines(&plan, date)
                .iter()
                .map(bridge::import_line)
                .collect();
            ui.set_lines(bridge::model(lines));
            update_gates(ui, Some(&state));
            *preview.borrow_mut() = Some(state);
        }
        Err(e) => {
            *preview.borrow_mut() = None;
            ui.set_lines(bridge::model(Vec::new()));
            ui.set_message(vm::error_text(&e).into());
            update_gates(ui, None);
        }
    }
}

fn run_apply(app: &App, ui: &ImportState<'_>, preview: &Shared) {
    let text = ui.get_text().to_string();
    let date_text = ui.get_date_text().to_string();
    let allowed = imp::can_apply(preview.borrow().as_ref(), &text, &date_text);
    let Ok(date) = vm::parse_date(&date_text) else {
        return;
    };
    if !allowed {
        return;
    }
    let today = app.today();
    let result = app.planner_mut().apply_import(&text, date, today);
    *preview.borrow_mut() = None;
    match result {
        Ok(report) => {
            ui.set_message(SharedString::new());
            let line = imp::PreviewLine {
                text: imp::import_report_text(&report),
                kind: imp::LineKind::Summary,
            };
            ui.set_lines(bridge::model(vec![bridge::import_line(&line)]));
            update_gates(ui, None);
            // R6: sync with today and refresh every page.
            app.changed(true);
        }
        Err(e) => {
            ui.set_message(vm::error_text(&e).into());
            update_gates(ui, None);
        }
    }
}
