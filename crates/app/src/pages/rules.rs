//! Rules page: fills `RulesState` (contracts/ui-grading.md "Rules page"). Rule sets with their
//! components and attendance bands (R34, R35) and the grade scale (R39). The row text is parsed by
//! the view model (`parse_component_row`, `parse_band_row`, `parse_grade_row`); the R34 and R39
//! range checks are core's, and its `Error::Validation` text is shown inline. A save refreshes
//! every page, so the Courses page shows changed marks and GPA.

use std::rc::{Rc, Weak};

use planner_core::{ComponentKind, RuleSet, RuleSetInput};
use slint::{ComponentHandle, Model, ModelRc, SharedString};

use crate::controller::App;
use crate::viewmodel::{self as vm};
use crate::{bridge, ComponentRow, GradeRowView, RulesState};

/// `selected-id` when the form creates a new rule set.
const NEW_SET: i32 = -1;

/// Reloads the rule set list and, while no edits are pending, the grade scale. An open form is
/// kept (it holds unsaved edits) unless its rule set no longer exists.
pub fn refresh(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let ui = window.global::<RulesState>();
    if let Err(text) = fill(app, &ui) {
        ui.set_message(text.into());
    }
}

fn fill(app: &App, ui: &RulesState<'_>) -> Result<(), String> {
    let sets = app.planner().list_rule_sets().map_err(err)?;
    let rows: Vec<_> = sets.iter().map(vm::rule_set_row).collect();
    ui.set_rule_sets(bridge::rows(&rows, bridge::rule_set_row)?);
    let selected = ui.get_selected_id();
    if ui.get_form_open()
        && selected != NEW_SET
        && !sets.iter().any(|s| i64::from(selected) == s.id)
    {
        ui.set_form_open(false);
    }
    if !ui.get_scale_dirty() {
        load_scale(app, ui)?;
    }
    Ok(())
}

/// Connects the page callbacks.
pub fn wire(app: &Rc<App>) {
    let Some(window) = app.window() else {
        return;
    };
    let ui = window.global::<RulesState>();

    let weak = Rc::downgrade(app);
    ui.on_begin_new(move || with(&weak, begin_new));

    let weak = Rc::downgrade(app);
    ui.on_edit_set(move |id| with(&weak, |app, ui| edit_set(app, ui, id)));

    let weak = Rc::downgrade(app);
    ui.on_delete_set(move |id| with(&weak, |app, ui| delete_set(app, ui, id)));

    let weak = Rc::downgrade(app);
    ui.on_save_set(move || with(&weak, save_set));

    let weak = Rc::downgrade(app);
    ui.on_cancel_form(move || with(&weak, |_, ui| ui.set_form_open(false)));

    let weak = Rc::downgrade(app);
    ui.on_add_component(move |is_items| {
        with(&weak, |_, ui| {
            let blank = vm::ComponentFormRow {
                id: None,
                name: String::new(),
                is_items,
                weight: String::new(),
                item_count: String::new(),
                best_count: String::new(),
                item_max: String::new(),
                is_final: false,
            };
            edit_components(ui, |rows| {
                if let Ok(row) = bridge::component_row(&blank) {
                    rows.push(row);
                }
            });
        })
    });

    let weak = Rc::downgrade(app);
    ui.on_remove_component(move |i| {
        with(&weak, |_, ui| {
            edit_components(ui, |rows| {
                if let Some(i) = index(i, rows.len()) {
                    rows.remove(i);
                }
            });
        })
    });

    let weak = Rc::downgrade(app);
    ui.on_move_component(move |i, delta| {
        with(&weak, |_, ui| {
            edit_components(ui, |rows| {
                let target = i.checked_add(delta).and_then(|t| index(t, rows.len()));
                if let (Some(from), Some(to)) = (index(i, rows.len()), target) {
                    rows.swap(from, to);
                }
            });
        })
    });

    let weak = Rc::downgrade(app);
    ui.on_toggle_kind(move |i, is_items| {
        with(&weak, |_, ui| {
            edit_components(ui, |rows| {
                if let Some(row) = index(i, rows.len()).map(|i| &mut rows[i]) {
                    row.is_items = is_items;
                    // R34: only an items component can be the final.
                    row.is_final = row.is_final && is_items;
                }
            });
        })
    });

    let weak = Rc::downgrade(app);
    ui.on_set_final(move |i, on| {
        with(&weak, |_, ui| {
            edit_components(ui, |rows| {
                // R34: exactly one final; choosing one clears the others.
                let chosen = index(i, rows.len());
                for (j, row) in rows.iter_mut().enumerate() {
                    row.is_final = on && chosen == Some(j);
                }
            });
        })
    });

    let weak = Rc::downgrade(app);
    ui.on_add_band(move || {
        with(&weak, |_, ui| {
            let mut rows = rows_of(&ui.get_bands());
            rows.push(bridge::band_row(&(String::new(), String::new())));
            ui.set_bands(bridge::model(rows));
        })
    });

    let weak = Rc::downgrade(app);
    ui.on_remove_band(move |i| {
        with(&weak, |_, ui| {
            let mut rows = rows_of(&ui.get_bands());
            if let Some(i) = index(i, rows.len()) {
                rows.remove(i);
            }
            ui.set_bands(bridge::model(rows));
        })
    });

    let weak = Rc::downgrade(app);
    ui.on_add_scale_row(move || {
        with(&weak, |_, ui| {
            let mut rows = rows_of(&ui.get_scale());
            rows.push(bridge::grade_row(&(
                String::new(),
                String::new(),
                String::new(),
            )));
            ui.set_scale(bridge::model(rows));
            ui.set_scale_dirty(true);
        })
    });

    let weak = Rc::downgrade(app);
    ui.on_remove_scale_row(move |i| {
        with(&weak, |_, ui| {
            let mut rows = rows_of(&ui.get_scale());
            if let Some(i) = index(i, rows.len()) {
                rows.remove(i);
            }
            ui.set_scale(bridge::model(rows));
            ui.set_scale_dirty(true);
        })
    });

    let weak = Rc::downgrade(app);
    ui.on_save_scale(move || with(&weak, save_scale));
}

/// Runs `f` with the app and the page global if both are still alive.
fn with(weak: &Weak<App>, f: impl FnOnce(&App, &RulesState<'_>)) {
    let Some(app) = weak.upgrade() else {
        return;
    };
    let Some(window) = app.window() else {
        return;
    };
    f(&app, &window.global::<RulesState>());
}

/// Core error as display text.
fn err(e: planner_core::Error) -> String {
    vm::error_text(&e)
}

/// `i` as a valid index into a list of `len` rows.
fn index(i: i32, len: usize) -> Option<usize> {
    usize::try_from(i).ok().filter(|&i| i < len)
}

/// Every row of a Slint model.
fn rows_of<T: Clone + 'static>(model: &ModelRc<T>) -> Vec<T> {
    model.iter().collect()
}

/// Rebuilds the component rows after `change`, then refreshes `has-attendance`. A full rebuild
/// (not a row update) makes every row's widgets show the stored values again.
fn edit_components(ui: &RulesState<'_>, change: impl FnOnce(&mut Vec<ComponentRow>)) {
    let mut rows = rows_of(&ui.get_components());
    change(&mut rows);
    set_components(ui, rows);
}

fn set_components(ui: &RulesState<'_>, rows: Vec<ComponentRow>) {
    ui.set_has_attendance(rows.iter().any(|r| !r.is_items));
    ui.set_components(bridge::model(rows));
}

/// Opens an empty form for a new rule set.
fn begin_new(_app: &App, ui: &RulesState<'_>) {
    ui.set_set_name(SharedString::new());
    set_components(ui, Vec::new());
    ui.set_bands(bridge::model(Vec::new()));
    ui.set_saved_lines(bridge::strings(&[]));
    ui.set_form_error(SharedString::new());
    ui.set_selected_id(NEW_SET);
    ui.set_form_open(true);
}

/// Fills the form from a stored rule set. R35: the bands of its attendance component fill the
/// band rows.
fn load_form(ui: &RulesState<'_>, set: &RuleSet) -> Result<(), String> {
    let rows: Vec<_> = set.components.iter().map(vm::component_form_row).collect();
    let rows = rows
        .iter()
        .map(bridge::component_row)
        .collect::<Result<Vec<_>, _>>()?;
    let bands: Vec<_> = set
        .components
        .iter()
        .find(|c| c.kind == ComponentKind::Attendance)
        .map(|c| c.bands.iter().map(vm::band_form_row).collect())
        .unwrap_or_default();
    // R34, R35, R87: read-only lines of the saved components and attendance bands.
    let lines = vm::rule_set_lines(set);
    ui.set_saved_lines(bridge::strings(&lines));
    ui.set_set_name(set.name.as_str().into());
    set_components(ui, rows);
    ui.set_bands(bridge::model(bands.iter().map(bridge::band_row).collect()));
    ui.set_form_error(SharedString::new());
    ui.set_selected_id(bridge::id(set.id)?);
    ui.set_form_open(true);
    Ok(())
}

/// Opens the form filled with the stored rule set `id`.
fn edit_set(app: &App, ui: &RulesState<'_>, id: i32) {
    let found = app
        .planner()
        .list_rule_sets()
        .map(|sets| sets.into_iter().find(|s| s.id == i64::from(id)));
    match found {
        Ok(Some(set)) => {
            if let Err(text) = load_form(ui, &set) {
                ui.set_message(text.into());
            }
        }
        Ok(None) => refresh(app),
        Err(e) => ui.set_message(err(e).into()),
    }
}

/// R34: deletes a rule set; core refuses while a course uses it, and that text is shown inline.
fn delete_set(app: &App, ui: &RulesState<'_>, id: i32) {
    let result = app.planner_mut().delete_rule_set(i64::from(id));
    match result {
        Ok(()) => {
            if ui.get_selected_id() == id {
                ui.set_form_open(false);
            }
            ui.set_message(SharedString::new());
            app.changed(false);
        }
        Err(e) => ui.set_message(err(e).into()),
    }
}

/// R34, R35: parses every row, then saves; core's validation error is shown under the form.
fn save_set(app: &App, ui: &RulesState<'_>) {
    let mut bands = Vec::new();
    let mut band_rows = rows_of(&ui.get_bands());
    let mut ok = true;
    for row in &mut band_rows {
        match vm::parse_band_row(&row.min, &row.credit) {
            Ok(band) => {
                row.error = SharedString::new();
                bands.push(band);
            }
            Err(e) => {
                row.error = vm::field_errors_text(&e).into();
                ok = false;
            }
        }
    }

    let mut components = Vec::new();
    let mut component_rows = rows_of(&ui.get_components());
    for row in &mut component_rows {
        let kind = if row.is_items {
            ComponentKind::Items
        } else {
            ComponentKind::Attendance
        };
        let stored_id = (row.id >= 0).then_some(i64::from(row.id));
        let parsed = vm::parse_component_row(
            stored_id,
            &row.name,
            kind,
            &row.weight,
            &row.item_count,
            &row.best_count,
            &row.item_max,
            row.is_final,
            bands.clone(),
        );
        match parsed {
            Ok(input) => {
                row.error = SharedString::new();
                components.push(input);
            }
            Err(e) => {
                row.error = vm::field_errors_text(&e).into();
                ok = false;
            }
        }
    }

    if !ok {
        // Row updates keep the cursor where it is; only the error text changes.
        write_errors(&ui.get_components(), component_rows);
        write_errors(&ui.get_bands(), band_rows);
        ui.set_form_error(SharedString::new());
        return;
    }
    write_errors(&ui.get_components(), component_rows);
    write_errors(&ui.get_bands(), band_rows);

    let selected = ui.get_selected_id();
    let input = RuleSetInput {
        id: (selected != NEW_SET).then_some(i64::from(selected)),
        name: ui.get_set_name().trim().to_string(),
        components,
    };
    let result = app.planner_mut().save_rule_set(input);
    match result {
        Ok(saved) => {
            if let Err(text) = load_form(ui, &saved) {
                ui.set_message(text.into());
            }
            ui.set_message(SharedString::new());
            app.changed(false);
        }
        Err(e) => ui.set_form_error(err(e).into()),
    }
}

/// Replaces each row of `model` with the matching row of `rows` without rebuilding the list.
fn write_errors<T: Clone + 'static>(model: &ModelRc<T>, rows: Vec<T>) {
    for (i, row) in rows.into_iter().enumerate() {
        model.set_row_data(i, row);
    }
}

/// R39: shows the stored grade scale and clears the pending-edit flag.
fn load_scale(app: &App, ui: &RulesState<'_>) -> Result<(), String> {
    let scale = app.planner().grade_scale().map_err(err)?;
    let rows: Vec<GradeRowView> = scale
        .iter()
        .map(|g| bridge::grade_row(&vm::grade_form_row(g)))
        .collect();
    ui.set_scale(bridge::model(rows));
    ui.set_scale_dirty(false);
    ui.set_scale_error(SharedString::new());
    Ok(())
}

/// R39: parses every row, then saves the scale; core's validation error is shown under the table.
fn save_scale(app: &App, ui: &RulesState<'_>) {
    let mut rows = rows_of(&ui.get_scale());
    let mut scale = Vec::new();
    let mut ok = true;
    for row in &mut rows {
        match vm::parse_grade_row(&row.letter, &row.min, &row.points) {
            Ok(grade) => {
                row.error = SharedString::new();
                scale.push(grade);
            }
            Err(e) => {
                row.error = vm::field_errors_text(&e).into();
                ok = false;
            }
        }
    }
    write_errors(&ui.get_scale(), rows);
    if !ok {
        ui.set_scale_error(SharedString::new());
        return;
    }
    let result = app.planner_mut().save_grade_scale(&scale);
    match result {
        Ok(()) => {
            ui.set_scale_dirty(false);
            ui.set_scale_error(SharedString::new());
            app.changed(false);
        }
        Err(e) => ui.set_scale_error(err(e).into()),
    }
}
