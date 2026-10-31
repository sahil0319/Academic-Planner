//! Academic Planner desktop app (docs/spec.md §2, §6): opens the database, syncs sessions, and runs
//! the Slint window. The only place that reads the system clock (§1).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

mod bridge;
mod controller;
mod holidays;
mod notify;
mod pages;
mod restore;
mod toast;
mod updater;
mod viewmodel;

use std::rc::Rc;

use chrono::NaiveDateTime;
use planner_core::Planner;
use slint::winit_030::WinitWindowAccessor;
use slint::ComponentHandle;

use crate::controller::App;

/// Current local date and time. §1, research R-D1: the app's single clock read; everything else
/// receives `now` or `today` from here.
#[allow(clippy::disallowed_methods)] // §1: the app is the only place that reads the clock
fn local_now() -> NaiveDateTime {
    chrono::Local::now().naive_local()
}

fn main() -> anyhow::Result<()> {
    // Slint creates winit windows with `transparent(true)`; on macOS that leaves the native title
    // bar see-through because the app paints only the content area. The app never uses window
    // transparency, so ask for an opaque window on every platform.
    slint::BackendSelector::new()
        .with_winit_window_attributes_hook(|attributes| attributes.with_transparent(false))
        .select()?;

    let window = AppWindow::new()?;

    // R24: database path; R25: a newer schema is refused by `open`.
    let opened = planner_core::default_db_path()
        .and_then(|path| Planner::open(&path).map(|planner| (planner, path)));

    // Kept alive until the window closes; callbacks hold weak references to it.
    let _app = match opened {
        Err(e) => {
            // FR-008: no panic; only the error screen is shown.
            window
                .global::<AppState>()
                .set_startup_error(viewmodel::error_text(&e).into());
            // §6 "Appearance:": the switch works on this screen too; with no database the choice
            // is shown but not saved.
            bridge::apply_theme(&window, planner_core::ThemeMode::System);
            let weak = window.as_weak();
            window.global::<AppState>().on_theme_selected(move |index| {
                if let (Some(window), Some(theme)) =
                    (weak.upgrade(), viewmodel::theme_from_index(index))
                {
                    bridge::apply_theme(&window, theme);
                }
            });
            None
        }
        Ok((planner, path)) => {
            let app = App::new(planner, path, local_now, &window);
            pages::dashboard::wire(&app);
            pages::timetable::wire(&app);
            pages::courses::wire(&app);
            pages::assessments::wire(&app);
            pages::settings::wire(&app);
            pages::about::wire(&app);
            pages::rules::wire(&app);
            pages::import::wire(&app);
            app.start_toasts();

            let weak = Rc::downgrade(&app);
            window.global::<AppState>().on_page_selected(move |_| {
                if let Some(app) = weak.upgrade() {
                    app.refresh_all();
                }
            });
            let weak = Rc::downgrade(&app);
            window.global::<AppState>().on_theme_selected(move |index| {
                if let Some(app) = weak.upgrade() {
                    app.select_theme(index);
                }
            });

            // §5 Settings theme: show the saved appearance before the first frame.
            app.load_theme();

            // R6: sync on app start, then fill every page.
            app.changed(true);

            // R27, R29, §1 Network: one background update check per launch when the About
            // switch is on. It never installs anything; a failure shows only on About. An
            // unreadable setting counts as off, so no request is made.
            let check_on_start = app
                .planner()
                .settings()
                .is_ok_and(|s| s.check_updates_on_start);
            if check_on_start {
                pages::about::start_check(&window, true);
            }
            Some(app)
        }
    };

    // R6: every 60 seconds, sync when the date has rolled over and refresh the now-dependent
    // lists, then (FR-006) show desktop notifications for due reminders. The timer stops when it is dropped after the window closes.
    let timer = slint::Timer::default();
    if let Some(app) = &_app {
        let weak = Rc::downgrade(app);
        timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_secs(60),
            move || {
                if let Some(app) = weak.upgrade() {
                    app.tick();
                }
            },
        );
    }

    // §6 "Appearance:": the native window (and its title bar) only exists once the event loop has
    // created it, so the saved appearance is applied as soon as it does.
    let weak = window.as_weak();
    slint::spawn_local(async move {
        let Some(window) = weak.upgrade() else {
            return;
        };
        if let Ok(native) = window.window().winit_window().await {
            native.set_theme(bridge::remembered_native_theme());
        }
    })?;

    window.run()?;

    // R28, R30: an installed update relaunches only after the database is closed (dropping
    // `_app` closes it). The updater replaced only the app; the new version's `open` applies
    // R25/R26 to the untouched database.
    drop(timer);
    drop(_app);
    if let Some(mut command) = pages::about::take_relaunch() {
        if let Err(e) = command.spawn() {
            eprintln!("could not start the updated app: {e}");
        }
    }
    Ok(())
}
