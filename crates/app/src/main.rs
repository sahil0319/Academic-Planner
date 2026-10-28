//! Academic Planner desktop app (docs/spec.md §2, §6): opens the database, syncs sessions, and runs
//! the Slint window. The only place that reads the system clock (§1).

slint::include_modules!();

mod bridge;
mod controller;
mod pages;
mod restore;
mod viewmodel;

use std::rc::Rc;

use chrono::NaiveDateTime;
use planner_core::Planner;
use slint::ComponentHandle;

use crate::controller::App;

/// Current local date and time. §1, research R-D1: the app's single clock read; everything else
/// receives `now` or `today` from here.
#[allow(clippy::disallowed_methods)] // §1: the app is the only place that reads the clock
fn local_now() -> NaiveDateTime {
    chrono::Local::now().naive_local()
}

fn main() -> anyhow::Result<()> {
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
            None
        }
        Ok((planner, path)) => {
            let app = App::new(planner, path, local_now, &window);
            pages::dashboard::wire(&app);
            pages::timetable::wire(&app);
            pages::courses::wire(&app);
            pages::assessments::wire(&app);
            pages::settings::wire(&app);

            let weak = Rc::downgrade(&app);
            window.global::<AppState>().on_page_selected(move |_| {
                if let Some(app) = weak.upgrade() {
                    app.refresh_all();
                }
            });

            // R6: sync on app start, then fill every page.
            app.changed(true);
            Some(app)
        }
    };

    // R6: every 60 seconds, sync when the date has rolled over and refresh the now-dependent
    // lists. The timer stops when it is dropped after the window closes.
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

    window.run()?;
    Ok(())
}
