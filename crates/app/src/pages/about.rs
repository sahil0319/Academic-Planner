//! About page and update banner (docs/spec.md §13.1 UI, R27–R30;
//! specs/007-m7-updater/contracts/ui-about.md, contracts/app-updater.md).
//!
//! Network work runs on worker threads; results come back to the UI thread only through
//! `slint::invoke_from_event_loop` with `Send` data and a `slint::Weak<AppWindow>`. The update
//! state lives in a UI-thread `thread_local` (research D8).

use std::cell::{Cell, RefCell};
use std::process::Command;
use std::rc::{Rc, Weak};

use cargo_packager_updater::Update;
use planner_core::Settings;
use slint::{ComponentHandle, SharedString};

use crate::controller::App;
use crate::updater::{self, InstallKind};
use crate::viewmodel::{self as vm, UpdatePhase};
use crate::{AboutState, AppState, AppWindow};

/// §13.1 UI: the app name shown on the About page.
const APP_NAME: &str = "Academic Planner";
/// §6: page index of About in the navigation.
const ABOUT_PAGE: i32 = 5;

/// UI-thread update state (research D8).
struct UpdateState {
    phase: UpdatePhase,
    /// R28: the update found by the last successful check, installed only after a click.
    pending: Option<Update>,
    /// R29: the version announced by the startup check's banner.
    banner_version: Option<String>,
    /// R29: the banner was dismissed for this launch.
    banner_dismissed: bool,
    /// R28: the confirm panel is open.
    confirm_open: bool,
    /// R28: how to start the new version after the window closes.
    relaunch: Option<Command>,
}

thread_local! {
    static STATE: RefCell<UpdateState> = const {
        RefCell::new(UpdateState {
            phase: UpdatePhase::Idle,
            pending: None,
            banner_version: None,
            banner_dismissed: false,
            confirm_open: false,
            relaunch: None,
        })
    };
}

fn with_state<T>(f: impl FnOnce(&mut UpdateState) -> T) -> T {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

/// Fills the static About items, the startup switch, the update status, and the banner.
pub fn refresh(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<AboutState>();
    page.set_message(SharedString::new());
    page.set_app_name(APP_NAME.into());
    page.set_version_text(vm::version_text(updater::RUNNING_VERSION).into());
    page.set_license_text(vm::license_text().into());
    page.set_db_path(vm::db_path_text(app.db_path()).into());
    load_switch(app, &window);
    show_update(&window);
}

/// R29: shows the saved `check_updates_on_start` in the switch.
fn load_switch(app: &App, window: &AppWindow) {
    let page = window.global::<AboutState>();
    match app.planner().settings() {
        Ok(s) => page.set_check_on_start(s.check_updates_on_start),
        Err(e) => page.set_message(vm::error_text(&e).into()),
    }
}

/// R27, R28, R29: copies the update state into `AboutState` and the banner into `AppState`.
fn show_update(window: &AppWindow) {
    let page = window.global::<AboutState>();
    let (view, downloading, confirm_open, banner) = with_state(|s| {
        let banner = match (&s.banner_version, s.banner_dismissed) {
            (Some(version), false) => vm::banner_text(version),
            _ => String::new(),
        };
        (
            vm::update_view(&s.phase),
            matches!(s.phase, UpdatePhase::Downloading { .. }),
            s.confirm_open,
            banner,
        )
    });
    page.set_status_text(view.status.into());
    page.set_action_label(view.action.into());
    page.set_action_enabled(view.action_enabled);
    page.set_downloading(downloading);
    page.set_progress(view.progress);
    page.set_progress_text(view.progress_text.into());
    page.set_is_error(view.is_error);
    page.set_confirm_open(confirm_open);
    window.global::<AppState>().set_update_banner(banner.into());
}

/// Connects the About callbacks and the banner buttons.
pub fn wire(app: &Rc<App>) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<AboutState>();

    let weak = window.as_weak();
    page.on_action(move || {
        if let Some(window) = weak.upgrade() {
            action(&window);
        }
    });
    let weak = window.as_weak();
    page.on_cancel(move || {
        if let Some(window) = weak.upgrade() {
            with_state(|s| s.confirm_open = false);
            show_update(&window);
        }
    });
    page.on_confirm(with(app, confirm));
    page.on_check_on_start_toggled(with(app, toggle_check_on_start));
    page.on_open_repo(with(app, |app| open_link(app, updater::REPO_URL)));
    page.on_open_notices(with(app, |app| {
        open_link(app, &updater::notices_url(updater::RUNNING_VERSION));
    }));

    let state = window.global::<AppState>();
    let weak = Rc::downgrade(app);
    state.on_update_banner_view(move || {
        if let Some(app) = weak.upgrade() {
            if let Some(window) = app.window() {
                window.global::<AppState>().set_current_page(ABOUT_PAGE);
            }
            app.refresh_all();
        }
    });
    let weak = window.as_weak();
    state.on_update_banner_dismiss(move || {
        if let Some(window) = weak.upgrade() {
            with_state(|s| s.banner_dismissed = true);
            show_update(&window);
        }
    });
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

/// R27, R28: "Check for updates" / "Retry" start a check; "Install" opens the confirm panel.
/// Nothing happens while a check, download, or install runs (the button is disabled then).
fn action(window: &AppWindow) {
    let phase = with_state(|s| s.phase.clone());
    match phase {
        UpdatePhase::Idle
        | UpdatePhase::UpToDate
        | UpdatePhase::CheckFailed
        | UpdatePhase::InstallFailed => start_check(window, false),
        UpdatePhase::Available { version } => open_confirm(window, &version),
        UpdatePhase::Checking | UpdatePhase::Downloading { .. } | UpdatePhase::Installing => {}
    }
}

/// R27, §1 Network: runs one update check on a worker thread. A `startup` check (R29) that
/// finds an update also shows the banner; its failure is silent apart from the About status.
pub fn start_check(window: &AppWindow, startup: bool) {
    with_state(|s| {
        s.phase = UpdatePhase::Checking;
        s.pending = None;
        s.confirm_open = false;
    });
    show_update(window);
    let weak = window.as_weak();
    std::thread::spawn(move || {
        let result = updater::check().map_err(|e| e.to_string());
        // The event loop is gone only when the app is closing; the result is then moot.
        let _ = slint::invoke_from_event_loop(move || finish_check(&weak, result, startup));
    });
}

/// R27, R29: records a finished check on the UI thread.
fn finish_check(
    weak: &slint::Weak<AppWindow>,
    result: Result<Option<Update>, String>,
    startup: bool,
) {
    with_state(|s| match result {
        Ok(Some(update)) => {
            s.phase = UpdatePhase::Available {
                version: update.version.clone(),
            };
            if startup {
                s.banner_version = Some(update.version.clone());
            }
            s.pending = Some(update);
        }
        Ok(None) => s.phase = UpdatePhase::UpToDate,
        Err(e) => {
            eprintln!("update check failed: {e}");
            s.phase = UpdatePhase::CheckFailed;
        }
    });
    if let Some(window) = weak.upgrade() {
        show_update(&window);
    }
}

/// R28: opens the confirm panel for `version`; nothing is installed before its button.
fn open_confirm(window: &AppWindow, version: &str) {
    let notes = with_state(|s| {
        s.confirm_open = true;
        s.pending
            .as_ref()
            .and_then(|u| u.body.clone())
            .unwrap_or_default()
    });
    let page = window.global::<AboutState>();
    page.set_confirm_title(vm::confirm_title(version).into());
    page.set_confirm_notes(notes.trim().into());
    page.set_confirm_label(vm::confirm_label(updater::current_install_kind()).into());
    show_update(window);
}

/// R28: the confirm click. `.deb` and dev builds open the release page; installed copies
/// download, verify, and install the update on a worker thread, then quit to relaunch.
fn confirm(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let pending = with_state(|s| {
        s.confirm_open = false;
        s.pending.clone()
    });
    let Some(update) = pending else {
        show_update(&window);
        return;
    };
    match updater::current_install_kind() {
        InstallKind::OpenReleasePage => {
            show_update(&window);
            open_link(app, &updater::release_page_url(&update.version));
        }
        InstallKind::SelfInstall => start_install(&window, update),
    }
}

/// R28: downloads and installs `update` on a worker thread. Progress is posted to the UI
/// thread only when the whole percentage changes.
fn start_install(window: &AppWindow, update: Update) {
    with_state(|s| {
        s.phase = UpdatePhase::Downloading {
            received: 0,
            total: None,
        }
    });
    show_update(window);
    let weak = window.as_weak();
    std::thread::spawn(move || {
        let last: Cell<Option<Option<u64>>> = Cell::new(None);
        let progress_weak = weak.clone();
        let result = updater::download_and_install(&update, |received, total| {
            let percent = vm::download_percent(received, total);
            if last.get() == Some(percent) {
                return;
            }
            last.set(Some(percent));
            let weak = progress_weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                let downloading = with_state(|s| {
                    let downloading = matches!(s.phase, UpdatePhase::Downloading { .. });
                    if downloading {
                        s.phase = UpdatePhase::Downloading { received, total };
                    }
                    downloading
                });
                if let (true, Some(window)) = (downloading, weak.upgrade()) {
                    show_update(&window);
                }
            });
        });
        let result = result.map_err(|e| e.to_string());
        let _ = slint::invoke_from_event_loop(move || finish_install(&weak, result));
    });
}

/// R28: on success stores the relaunch command and closes the window (main.rs relaunches
/// after the database is closed); on failure nothing was installed and Retry checks again.
fn finish_install(weak: &slint::Weak<AppWindow>, result: Result<Option<Command>, String>) {
    let installed = with_state(|s| match result {
        Ok(command) => {
            s.phase = UpdatePhase::Installing;
            s.relaunch = command;
            true
        }
        Err(e) => {
            eprintln!("update install failed: {e}");
            s.phase = UpdatePhase::InstallFailed;
            s.pending = None;
            false
        }
    });
    if let Some(window) = weak.upgrade() {
        show_update(&window);
    }
    if installed {
        let _ = slint::quit_event_loop();
    }
}

/// R28, R30: the command that starts the new version, taken once after the window closed.
pub fn take_relaunch() -> Option<Command> {
    with_state(|s| s.relaunch.take())
}

/// R29, §1 Saving: saves the switch at once with only `check_updates_on_start` changed; on
/// error shows the message and reloads the switch.
fn toggle_check_on_start(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<AboutState>();
    let on = page.get_check_on_start();
    let current = app.planner().settings();
    let saved = current.and_then(|current| {
        app.planner_mut().save_settings(&Settings {
            check_updates_on_start: on,
            ..current
        })
    });
    match saved {
        Ok(()) => refresh(app),
        Err(e) => {
            page.set_message(vm::error_text(&e).into());
            load_switch(app, &window);
        }
    }
}

/// §13.1 UI, R28: opens `url` in the browser; a failure is shown on the current page.
fn open_link(app: &App, url: &str) {
    if let Err(e) = updater::open_url(url) {
        app.show_message(&format!("{e:#}"));
    }
}
