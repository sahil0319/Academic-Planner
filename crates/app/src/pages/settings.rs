//! Settings page: term, holidays, reminder settings, backup, restore, CSV export, and the database
//! path (docs/spec.md §6; contracts/ui-bridge.md "SettingsState", "Restore flow").

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::rc::{Rc, Weak};

use planner_core::Error;
use slint::{ComponentHandle, SharedString};

use crate::controller::App;
use crate::viewmodel as vm;
use crate::{bridge, restore, SettingsState};

const TERM_SAVED: &str = "Term saved.";
const HOLIDAYS_SAVED: &str = "Holidays saved.";
const HOLIDAY_REMOVED: &str = "Holiday removed.";
const SETTINGS_SAVED: &str = "Settings saved.";

/// Reloads the holiday list, the term flag, and the database path. Form text is not touched here
/// (see [`load_forms`]) so the minute timer does not wipe what the student is typing.
pub fn refresh(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();
    page.set_message(SharedString::new());
    // R24: database file path as text.
    page.set_db_path(vm::db_path_text(app.db_path()).into());
    match app.planner().active_term() {
        Ok(term) => {
            page.set_has_term(term.is_some());
            let rows = term
                .map(|t| t.holidays.iter().map(vm::holiday_row).collect::<Vec<_>>())
                .unwrap_or_default();
            page.set_holidays(bridge::model(
                rows.iter().map(bridge::holiday_row).collect(),
            ));
        }
        Err(e) => page.set_message(vm::error_text(&e).into()),
    }
}

/// Fills the term and settings forms from the database: at startup and after a save or restore.
fn load_forms(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();
    let planner = app.planner();
    match planner.active_term() {
        Ok(Some(term)) => {
            page.set_term_name(term.name.as_str().into());
            page.set_term_start(vm::format_date_input(term.start_date).into());
            page.set_term_end(vm::format_date_input(term.end_date).into());
        }
        Ok(None) => {
            page.set_term_name(SharedString::new());
            page.set_term_start(SharedString::new());
            page.set_term_end(SharedString::new());
        }
        Err(e) => page.set_term_form_error(vm::error_text(&e).into()),
    }
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
    page.on_add_holiday(with(app, add_holiday));
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

/// R6: runs the controller's changed path, then shows `note` unless the refresh or sync already
/// put an error in the page message.
fn changed_with_note(app: &App, sync: bool, note: &str) {
    app.changed(sync);
    if let Some(window) = app.window() {
        let page = window.global::<SettingsState>();
        if page.get_message().is_empty() {
            page.set_message(note.into());
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
    match saved {
        Err(e) => page.set_term_form_error(vm::error_text(&e).into()),
        Ok(_) => {
            page.set_term_form_error(SharedString::new());
            // R6: a term change re-syncs sessions.
            changed_with_note(app, true, TERM_SAVED);
            load_forms(app);
        }
    }
}

/// Human decision §6: adds one date or an inclusive range (one holiday per day, the range label
/// winning on overlap) to the active term; R4, R6: then syncs.
fn add_holiday(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();
    let term = match app.planner().active_term() {
        Ok(Some(term)) => term,
        Ok(None) => {
            page.set_hol_form_error(vm::error_text(&Error::NoActiveTerm).into());
            return;
        }
        Err(e) => {
            page.set_hol_form_error(vm::error_text(&e).into());
            return;
        }
    };
    let added = vm::parse_holiday_form(
        &page.get_hol_start(),
        &page.get_hol_end(),
        &page.get_hol_label(),
    );
    let added = match added {
        Ok(added) => added,
        Err(errors) => {
            page.set_hol_start_error(errors.get("start").into());
            page.set_hol_end_error(errors.get("end").into());
            page.set_hol_form_error(SharedString::new());
            return;
        }
    };
    page.set_hol_start_error(SharedString::new());
    page.set_hol_end_error(SharedString::new());
    let merged = vm::merge_holidays(&term.holidays, &added);
    let saved = app.planner_mut().set_holidays(term.id, &merged);
    match saved {
        Err(e) => page.set_hol_form_error(vm::error_text(&e).into()),
        Ok(()) => {
            page.set_hol_form_error(SharedString::new());
            page.set_hol_start(SharedString::new());
            page.set_hol_end(SharedString::new());
            page.set_hol_label(SharedString::new());
            changed_with_note(app, true, HOLIDAYS_SAVED);
        }
    }
}

/// Human decision §6: removes one holiday date from the active term; R6: then syncs.
fn remove_holiday(app: &App, date: &str) {
    let term = match app.planner().active_term() {
        Ok(Some(term)) => term,
        Ok(None) => return message(app, &vm::error_text(&Error::NoActiveTerm)),
        Err(e) => return message(app, &vm::error_text(&e)),
    };
    // The row passes its display text (`format_date`, one text per date), so match on it.
    let Some(date) = term
        .holidays
        .iter()
        .map(|h| h.date)
        .find(|d| vm::format_date(*d) == date)
    else {
        return message(app, "This holiday no longer exists");
    };
    let remaining = vm::remove_holiday(&term.holidays, date);
    let saved = app.planner_mut().set_holidays(term.id, &remaining);
    match saved {
        Err(e) => message(app, &vm::error_text(&e)),
        Ok(()) => {
            changed_with_note(app, true, HOLIDAY_REMOVED);
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
    let saved = app.planner_mut().save_settings(&settings);
    match saved {
        Err(e) => page.set_settings_form_error(vm::error_text(&e).into()),
        Ok(()) => {
            page.set_settings_form_error(SharedString::new());
            changed_with_note(app, false, SETTINGS_SAVED);
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
    let text = match json {
        Ok(json) => match std::fs::write(&path, json) {
            Ok(()) => vm::backup_saved_message(&path),
            Err(e) => vm::file_error_message(&path, &e.to_string()),
        },
        Err(e) => vm::error_text(&e),
    };
    message(app, &text);
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
    changed_with_note(app, true, &vm::restored_message(&copy));
    load_forms(app);
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
        Ok(()) => message(app, &vm::csv_exported_message(&folder)),
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
