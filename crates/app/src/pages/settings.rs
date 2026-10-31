//! Settings page: term, holidays, reminder settings, backup, restore, CSV export, and the database
//! path (docs/spec.md §6; contracts/ui-bridge.md "SettingsState", "Restore flow").

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::rc::{Rc, Weak};

use planner_core::{DayOffKind, Error, Settings};
use slint::{ComponentHandle, SharedString};

use crate::controller::App;
use crate::viewmodel as vm;
use crate::{bridge, restore, SettingsState};

/// Reloads the Term card, the holiday list, the past terms, and the database path. Form text is
/// not touched here (see [`load_forms`], [`edit_term`]) so the minute timer does not wipe what the
/// student is typing, and `term-editing` is kept so it does not close an open form.
pub fn refresh(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();
    page.set_message(SharedString::new());
    // R24: database file path as text.
    page.set_db_path(vm::db_path_text(app.db_path()).into());
    let today = app.today();
    let active = app.planner().active_term();
    match active {
        Ok(term) => {
            let has_term = term.is_some();
            page.set_has_term(has_term);
            if !has_term {
                // Nothing to edit or archive without an active term.
                page.set_term_editing(false);
                page.set_confirm_archive(false);
            }
            let editing = has_term && page.get_term_editing();
            // R67, R69: the form shows only with no active term or after Edit.
            page.set_term_form_open(vm::term_form_open(has_term, editing));
            // R67, R68: the read-only Term card and the Archive state.
            let card = term
                .as_ref()
                .map(|t| vm::term_card(t, today))
                .unwrap_or_default();
            page.set_term_title(card.title.into());
            page.set_term_dates(card.dates.into());
            page.set_term_week(card.week.into());
            let (enabled, hint) = match &term {
                Some(t) => (
                    vm::archive_enabled(t.end_date, today),
                    vm::archive_hint_text(t.end_date, today),
                ),
                None => (false, String::new()),
            };
            page.set_archive_enabled(enabled);
            page.set_archive_hint(hint.into());
            // R73: one list per kind of day off (empty without a term).
            let days = term.map(|t| t.holidays).unwrap_or_default();
            let rows_of = |kind| {
                bridge::model(
                    vm::day_off_rows(&days, kind)
                        .iter()
                        .map(bridge::holiday_row)
                        .collect(),
                )
            };
            page.set_holidays(rows_of(DayOffKind::Holiday));
            page.set_midterms(rows_of(DayOffKind::Midterm));
            page.set_finals(rows_of(DayOffKind::Final));
        }
        Err(e) => page.set_message(vm::error_text(&e).into()),
    }
    // R70: past terms, newest first.
    let past = app.planner().past_terms();
    match past {
        Ok(terms) => {
            let rows = terms
                .iter()
                .map(|t| bridge::past_term_row(&vm::past_term_row(t)))
                .collect();
            page.set_past_terms(bridge::model(rows));
        }
        Err(e) => page.set_message(vm::error_text(&e).into()),
    }
    // R76, R77, R79: the Public holidays card.
    super::public_holidays::refresh(app);
}

/// R67: empties the term form fields and their errors.
fn clear_term_fields(page: &SettingsState<'_>) {
    page.set_term_name(SharedString::new());
    page.set_term_start(SharedString::new());
    page.set_term_end(SharedString::new());
    page.set_term_start_error(SharedString::new());
    page.set_term_end_error(SharedString::new());
    page.set_term_form_error(SharedString::new());
}

/// Fills the settings form from the database: at startup and after a save or restore. The term
/// fields are filled only by Edit (R67).
fn load_forms(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();
    let planner = app.planner();
    match planner.settings() {
        Ok(s) => {
            page.set_reminder(s.class_reminder_minutes.to_string().into());
            page.set_window(s.upcoming_window_days.to_string().into());
            page.set_notifications(s.notifications_enabled);
        }
        Err(e) => page.set_settings_form_error(vm::error_text(&e).into()),
    }
}

/// Connects the page callbacks and fills the forms once.
pub fn wire(app: &Rc<App>) {
    load_forms(app);
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();

    page.on_save_term(with(app, save_term));
    page.on_edit_term(with(app, edit_term));
    page.on_cancel_term(with(app, cancel_term));
    page.on_archive_term(with(app, archive_term));
    page.on_archive_confirm(with(app, archive_confirm));
    page.on_archive_cancel(with(app, archive_cancel));
    page.on_add_holiday(with(app, |app| add_day_off(app, DayOffKind::Holiday)));
    page.on_add_midterm(with(app, |app| add_day_off(app, DayOffKind::Midterm)));
    page.on_add_final(with(app, |app| add_day_off(app, DayOffKind::Final)));
    let weak = Rc::downgrade(app);
    page.on_remove_holiday(move |date| {
        if let Some(app) = weak.upgrade() {
            remove_holiday(&app, &date);
        }
    });
    page.on_save_settings(with(app, save_settings));
    page.on_backup(with(app, backup));
    page.on_export_csv(with(app, export_csv));
    page.on_restore_pick(with(app, restore_pick));
    page.on_restore_confirm(with(app, restore_confirm));
    page.on_restore_cancel(with(app, restore_cancel));
}

/// A callback that runs `f` while the app is alive.
fn with(app: &Rc<App>, f: fn(&App)) -> impl Fn() + 'static {
    let weak: Weak<App> = Rc::downgrade(app);
    move || {
        if let Some(app) = weak.upgrade() {
            f(&app);
        }
    }
}

/// Shows `text` in the page message.
fn message(app: &App, text: &str) {
    if let Some(window) = app.window() {
        window.global::<SettingsState>().set_message(text.into());
    }
}

/// R6, R89: runs the controller's changed path, then shows `text` as a toast unless the refresh
/// or sync already put an error in the page message (errors stay inline, never toasts).
pub(super) fn changed_then_toast(app: &App, sync: bool, text: &str) {
    app.changed(sync);
    if let Some(window) = app.window() {
        if window.global::<SettingsState>().get_message().is_empty() {
            app.toast(text);
        }
    }
}

/// R1, R2, R6: creates the term, or edits the active one, then syncs.
fn save_term(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();
    let active_id = match app.planner().active_term() {
        Ok(term) => term.map(|t| t.id),
        Err(e) => {
            page.set_term_form_error(vm::error_text(&e).into());
            return;
        }
    };
    let input = vm::parse_term_form(
        active_id,
        &page.get_term_name(),
        &page.get_term_start(),
        &page.get_term_end(),
    );
    let input = match input {
        Ok(input) => input,
        Err(errors) => {
            page.set_term_start_error(errors.get("start").into());
            page.set_term_end_error(errors.get("end").into());
            page.set_term_form_error(SharedString::new());
            return;
        }
    };
    page.set_term_start_error(SharedString::new());
    page.set_term_end_error(SharedString::new());
    // R2: core validates the range; its message is the form error.
    let saved = app.planner_mut().save_term(input);
    // R67, R88, T70: success toasts, clears the fields and closes the form; an error stays
    // inline with the fields kept.
    let outcome = vm::term_save_outcome(&saved);
    if outcome.close_form {
        page.set_term_editing(false);
    }
    if outcome.clear_fields {
        clear_term_fields(&page);
    }
    page.set_term_form_error(outcome.form_error.into());
    if let Some(text) = outcome.toast {
        // R6: a term change re-syncs sessions; every page refreshes.
        changed_then_toast(app, true, &text);
    }
}

/// R67: Edit opens the form pre-filled with the active term.
fn edit_term(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();
    let active = app.planner().active_term();
    let term = match active {
        Ok(Some(term)) => term,
        Ok(None) => return,
        Err(e) => return message(app, &vm::error_text(&e)),
    };
    clear_term_fields(&page);
    page.set_term_name(term.name.as_str().into());
    page.set_term_start(vm::format_date_input(term.start_date).into());
    page.set_term_end(vm::format_date_input(term.end_date).into());
    page.set_term_editing(true);
    page.set_confirm_archive(false);
    refresh(app);
}

/// R67: Cancel closes the form and drops what was typed; no data changes.
fn cancel_term(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();
    page.set_term_editing(false);
    clear_term_fields(&page);
    refresh(app);
}

/// R68: Archive opens the inline confirm panel for the active term.
fn archive_term(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();
    let active = app.planner().active_term();
    let term = match active {
        Ok(Some(term)) => term,
        Ok(None) => return,
        Err(e) => return message(app, &vm::error_text(&e)),
    };
    page.set_archive_confirm_text(vm::archive_confirm_text(&term.name).into());
    page.set_confirm_archive(true);
}

/// R68, R88: archives the active term (core checks the end date), then toasts and refreshes
/// every page; an error stays inline in the page message.
fn archive_confirm(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();
    page.set_confirm_archive(false);
    let active = app.planner().active_term();
    let term = match active {
        Ok(Some(term)) => term,
        Ok(None) => return,
        Err(e) => return message(app, &vm::error_text(&e)),
    };
    let today = app.today();
    let archived = app.planner_mut().archive_term(term.id, today);
    match archived {
        Err(e) => message(app, &vm::error_text(&e)),
        Ok(()) => {
            page.set_term_editing(false);
            clear_term_fields(&page);
            changed_then_toast(app, true, &vm::term_archived_text());
        }
    }
}

/// R68: Cancel hides the confirm panel; nothing changes.
fn archive_cancel(app: &App) {
    if let Some(window) = app.window() {
        window.global::<SettingsState>().set_confirm_archive(false);
    }
}

/// R73: the form values of one day off card: first day, last day, and label text.
fn day_off_fields(
    page: &SettingsState<'_>,
    kind: DayOffKind,
) -> (SharedString, SharedString, SharedString) {
    match kind {
        DayOffKind::Holiday => (
            page.get_hol_start(),
            page.get_hol_end(),
            page.get_hol_label(),
        ),
        DayOffKind::Midterm => (
            page.get_mid_start(),
            page.get_mid_end(),
            page.get_mid_label(),
        ),
        DayOffKind::Final => (
            page.get_fin_start(),
            page.get_fin_end(),
            page.get_fin_label(),
        ),
    }
}

/// R73: sets the three error lines (first day, last day, form) of one day off card.
fn set_day_off_errors(
    page: &SettingsState<'_>,
    kind: DayOffKind,
    start: SharedString,
    end: SharedString,
    form: SharedString,
) {
    match kind {
        DayOffKind::Holiday => {
            page.set_hol_start_error(start);
            page.set_hol_end_error(end);
            page.set_hol_form_error(form);
        }
        DayOffKind::Midterm => {
            page.set_mid_start_error(start);
            page.set_mid_end_error(end);
            page.set_mid_form_error(form);
        }
        DayOffKind::Final => {
            page.set_fin_start_error(start);
            page.set_fin_end_error(end);
            page.set_fin_form_error(form);
        }
    }
}

/// R73: empties the three form fields of one day off card.
fn clear_day_off_fields(page: &SettingsState<'_>, kind: DayOffKind) {
    let empty = SharedString::new;
    match kind {
        DayOffKind::Holiday => {
            page.set_hol_start(empty());
            page.set_hol_end(empty());
            page.set_hol_label(empty());
        }
        DayOffKind::Midterm => {
            page.set_mid_start(empty());
            page.set_mid_end(empty());
            page.set_mid_label(empty());
        }
        DayOffKind::Final => {
            page.set_fin_start(empty());
            page.set_fin_end(empty());
            page.set_fin_label(empty());
        }
    }
}

/// Human decision §6, R71, R73: adds one date or an inclusive range (one day off of `kind` per
/// day, the range label winning on overlap) to the active term; R4, R6, R72: then syncs. An R71
/// error (the date already has another kind) shows inline in that card and keeps its fields.
fn add_day_off(app: &App, kind: DayOffKind) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();
    let none = SharedString::new;
    let term = match app.planner().active_term() {
        Ok(Some(term)) => term,
        Ok(None) => {
            set_day_off_errors(
                &page,
                kind,
                none(),
                none(),
                vm::error_text(&Error::NoActiveTerm).into(),
            );
            return;
        }
        Err(e) => {
            set_day_off_errors(&page, kind, none(), none(), vm::error_text(&e).into());
            return;
        }
    };
    let (start, end, label) = day_off_fields(&page, kind);
    let added = match vm::parse_holiday_form(&start, &end, &label) {
        Ok(added) => vm::as_day_off(added, kind),
        Err(errors) => {
            set_day_off_errors(
                &page,
                kind,
                errors.get("start").into(),
                errors.get("end").into(),
                none(),
            );
            return;
        }
    };
    let merged = vm::merge_holidays(&term.holidays, &added);
    let saved = app.planner_mut().set_holidays(term.id, &merged);
    match saved {
        Err(e) => set_day_off_errors(&page, kind, none(), none(), vm::error_text(&e).into()),
        Ok(()) => {
            set_day_off_errors(&page, kind, none(), none(), none());
            clear_day_off_fields(&page, kind);
            changed_then_toast(app, true, &vm::day_off_saved_text(kind));
        }
    }
}

/// Human decision §6, R73: removes one day off (of any kind) from the active term; R6: then
/// syncs. The toast names the removed kind (R89a).
fn remove_holiday(app: &App, key: &str) {
    let term = match app.planner().active_term() {
        Ok(Some(term)) => term,
        Ok(None) => return message(app, &vm::error_text(&Error::NoActiveTerm)),
        Err(e) => return message(app, &vm::error_text(&e)),
    };
    // The row passes its stored ISO date (`HolidayRow::key`), independent of the display format.
    let Some(removed) = term
        .holidays
        .iter()
        .find(|h| h.date.format("%Y-%m-%d").to_string() == key)
    else {
        return message(app, "This day off no longer exists");
    };
    let kind = removed.kind;
    let remaining = vm::remove_holiday(&term.holidays, removed.date);
    let saved = app.planner_mut().set_holidays(term.id, &remaining);
    match saved {
        Err(e) => message(app, &vm::error_text(&e)),
        Ok(()) => {
            changed_then_toast(app, true, &vm::day_off_removed_text(kind));
        }
    }
}

/// Saves reminder minutes, the upcoming window, and the notifications switch (no sync).
fn save_settings(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();
    let parsed = vm::parse_settings_form(
        &page.get_reminder(),
        &page.get_window(),
        page.get_notifications(),
    );
    let settings = match parsed {
        Ok(settings) => settings,
        Err(errors) => {
            page.set_reminder_error(errors.get("reminder").into());
            page.set_window_error(errors.get("window").into());
            page.set_settings_form_error(SharedString::new());
            return;
        }
    };
    page.set_reminder_error(SharedString::new());
    page.set_window_error(SharedString::new());
    // §5 Settings theme, R29: the form has neither the theme nor the update-check switch
    // (that one lives on About), so both saved values are kept.
    let current = app.planner().settings();
    let saved = current.and_then(|current| {
        app.planner_mut().save_settings(&Settings {
            theme: current.theme,
            check_updates_on_start: current.check_updates_on_start,
            ..settings
        })
    });
    match saved {
        Err(e) => page.set_settings_form_error(vm::error_text(&e).into()),
        Ok(()) => {
            page.set_settings_form_error(SharedString::new());
            changed_then_toast(app, false, &vm::settings_saved_text());
            load_forms(app);
        }
    }
}

/// R21: writes `export_json` to a file picked in a save dialog. Cancel does nothing.
fn backup(app: &App) {
    let now = app.now();
    let Some(path) = rfd::FileDialog::new()
        .set_file_name(vm::backup_file_name(now))
        .add_filter("JSON", &["json"])
        .save_file()
    else {
        return;
    };
    let json = app.planner().export_json(now);
    let saved = match json {
        Ok(json) => {
            std::fs::write(&path, json).map_err(|e| vm::file_error_message(&path, &e.to_string()))
        }
        Err(e) => Err(vm::error_text(&e)),
    };
    match saved {
        Ok(()) => {
            message(app, "");
            app.toast(&vm::backup_saved_message(&path));
        }
        Err(text) => message(app, &text),
    }
}

/// Shows or hides the inline restore confirm panel.
fn show_confirm(app: &App, shown: bool) {
    if let Some(window) = app.window() {
        window.global::<SettingsState>().set_confirm_restore(shown);
    }
}

/// R22 step 1: picks a backup file and shows the inline confirm panel warning that all current
/// data will be replaced. Cancel does nothing.
fn restore_pick(app: &App) {
    let Some(path) = rfd::FileDialog::new()
        .add_filter("JSON", &["json"])
        .pick_file()
    else {
        return;
    };
    app.state().pending_restore = Some(path);
    message(app, "");
    show_confirm(app, true);
}

/// R22 step 2: forgets the picked file and hides the panel; nothing is copied.
fn restore_cancel(app: &App) {
    app.state().pending_restore = None;
    show_confirm(app, false);
}

/// R22 step 3: reads the picked file, copies the current database to the backups folder, and only
/// then replaces all data. A failed copy means the restore is not attempted; a failed restore
/// leaves the data unchanged. R6: a successful restore re-syncs sessions.
fn restore_confirm(app: &App) {
    let pending = app.state().pending_restore.take();
    show_confirm(app, false);
    let Some(path) = pending else {
        return;
    };
    let json = match std::fs::read_to_string(&path) {
        Ok(json) => json,
        Err(e) => return message(app, &vm::file_error_message(&path, &e.to_string())),
    };
    let copy = match restore::pre_restore_copy(app.db_path(), app.now()) {
        Ok(copy) => copy,
        Err(e) => return message(app, &vm::pre_restore_failed_message(&format!("{e:#}"))),
    };
    let restored = app.planner_mut().restore_json(&json);
    if let Err(e) = restored {
        return message(app, &vm::error_text(&e));
    }
    {
        // Ids from before the restore may no longer exist.
        let mut state = app.state();
        state.detail_course = None;
        state.selected_session = None;
    }
    // The restored data may hold another term: close the term form and the archive panel.
    if let Some(window) = app.window() {
        let page = window.global::<SettingsState>();
        page.set_term_editing(false);
        page.set_confirm_archive(false);
        clear_term_fields(&page);
    }
    changed_then_toast(app, true, &vm::restored_message(&copy));
    load_forms(app);
    // §5 Settings theme: the restored backup carries its own theme.
    app.load_theme();
}

/// R23: writes `attendance.csv` and `assessments.csv` to a folder picked in a dialog. Cancel does
/// nothing.
fn export_csv(app: &App) {
    let Some(folder) = rfd::FileDialog::new().pick_folder() else {
        return;
    };
    let planner = app.planner();
    let attendance = write_csv(&folder.join("attendance.csv"), |w| {
        planner.export_attendance_csv(w)
    });
    let result = attendance.and_then(|()| {
        write_csv(&folder.join("assessments.csv"), |w| {
            planner.export_assessments_csv(w)
        })
    });
    drop(planner);
    match result {
        Ok(()) => {
            message(app, "");
            app.toast(&vm::csv_exported_message(&folder));
        }
        Err(text) => message(app, &text),
    }
}

/// Creates `path` and fills it with `export`; the error is the text to show.
fn write_csv(
    path: &Path,
    export: impl FnOnce(&mut BufWriter<File>) -> planner_core::Result<()>,
) -> Result<(), String> {
    let file = File::create(path).map_err(|e| vm::file_error_message(path, &e.to_string()))?;
    let mut writer = BufWriter::new(file);
    export(&mut writer).map_err(|e| vm::error_text(&e))?;
    writer
        .flush()
        .map_err(|e| vm::file_error_message(path, &e.to_string()))
}
