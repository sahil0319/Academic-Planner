//! Public holidays card of the Settings page (docs/spec.md §13.11, R76, R77, R79; D5, D28, D34).
//!
//! The fetch runs on a worker thread with `Send` data only; the result comes back through
//! `slint::invoke_from_event_loop`, as on the About page. The fetch status, the in-flight flag,
//! the ticked rows, and a weak `App` handle live in a UI-thread `thread_local` (D28), because the
//! event loop closure must be `Send` and cannot hold an `Rc`.

use std::cell::RefCell;
use std::collections::BTreeSet;
use std::rc::{Rc, Weak};

use chrono::NaiveDate;
use planner_core::{Error, PublicHoliday};
use slint::{ComponentHandle, SharedString};

use super::settings::changed_then_toast;
use crate::controller::App;
use crate::holidays::calendars::calendar_for;
use crate::holidays::fetch::HttpFetcher;
use crate::holidays::locale::{os_locale, resolve_country, setting_value, Country};
use crate::holidays::service::{self, FetchOutcome};
use crate::viewmodel::{self as vm, HolidayFetch};
use crate::{bridge, SettingsState};

/// UI-thread state of the card (D28).
struct CardState {
    /// The app, for the worker thread's result (never moved to another thread).
    app: Weak<App>,
    /// R77: where the refresh stands.
    fetch: HolidayFetch,
    /// A request is running; a second one is ignored.
    in_flight: bool,
    /// R79: ticked rows by key (`YYYY-MM-DD|name`).
    ticked: BTreeSet<String>,
}

thread_local! {
    static STATE: RefCell<CardState> = const {
        RefCell::new(CardState {
            app: Weak::new(),
            fetch: HolidayFetch::Idle,
            in_flight: false,
            ticked: BTreeSet::new(),
        })
    };
}

fn with_state<T>(f: impl FnOnce(&mut CardState) -> T) -> T {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

/// R76, D8: the country in force: the saved choice, else the OS region, else Off.
fn resolved_country(app: &App) -> Result<Country, String> {
    let saved = app
        .planner()
        .public_holiday_country()
        .map_err(|e| vm::error_text(&e))?;
    Ok(resolve_country(saved.as_deref(), os_locale().as_deref()))
}

/// R77, R79, D33: the cached public holidays of the resolved country dated `from..=to`; empty
/// for Off or a code outside the table. The dashboard reads its grid range through this.
pub fn cached_between(
    app: &App,
    from: NaiveDate,
    to: NaiveDate,
) -> Result<Vec<PublicHoliday>, String> {
    let Country::Code(code) = resolved_country(app)? else {
        return Ok(Vec::new());
    };
    if calendar_for(&code).is_none() {
        return Ok(Vec::new());
    }
    app.planner()
        .public_holidays_between(&code, from, to)
        .map_err(|e| vm::error_text(&e))
}

/// Connects the card callbacks and remembers the app for the worker thread's result.
pub fn wire(app: &Rc<App>) {
    with_state(|s| s.app = Rc::downgrade(app));
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();
    let weak = Rc::downgrade(app);
    page.on_ph_country_selected(move || {
        if let Some(app) = weak.upgrade() {
            country_selected(&app);
        }
    });
    let weak = Rc::downgrade(app);
    page.on_ph_retry(move || {
        if let Some(app) = weak.upgrade() {
            request_fetch(&app);
        }
    });
    page.on_ph_toggle(|key, checked| {
        with_state(|s| {
            if checked {
                s.ticked.insert(key.to_string());
            } else {
                s.ticked.remove(key.as_str());
            }
        });
    });
    let weak = Rc::downgrade(app);
    page.on_ph_add_selected(move || {
        if let Some(app) = weak.upgrade() {
            add_selected(&app);
        }
    });
}

/// R77, D5: the once-per-launch fetch, called after the first full refresh.
pub fn start(app: &Rc<App>) {
    request_fetch(app);
}

/// Fills the card: country choice, status, Retry, and the cached rows of the active term with
/// their ticks. Ticks of rows no longer listed are dropped (D28).
pub fn refresh(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();
    page.set_ph_error(SharedString::new());
    page.set_ph_countries(bridge::strings(&vm::country_options()));
    let country = match resolved_country(app) {
        Ok(country) => country,
        Err(text) => {
            page.set_ph_error(text.into());
            Country::Off
        }
    };
    page.set_ph_country_index(vm::country_index(&country));
    let fetch = with_state(|s| s.fetch);
    page.set_ph_status(vm::public_holiday_status_text(fetch, app.today()).into());
    page.set_ph_can_retry(vm::public_holiday_retry_visible(fetch));

    let term = match app.planner().active_term() {
        Ok(term) => term,
        Err(e) => {
            page.set_ph_error(vm::error_text(&e).into());
            None
        }
    };
    // R79: the rows are the cached holidays inside the active term (D33).
    let days = match &term {
        Some(t) => cached_between(app, t.start_date, t.end_date).unwrap_or_else(|text| {
            page.set_ph_error(text.into());
            Vec::new()
        }),
        None => Vec::new(),
    };
    let rows = vm::public_holiday_rows(&days);
    let ticked = with_state(|s| {
        s.ticked
            .retain(|key| rows.iter().any(|row| &row.key == key));
        s.ticked.clone()
    });
    page.set_ph_rows(bridge::model(
        rows.iter()
            .map(|row| bridge::public_holiday_row(row, ticked.contains(&row.key)))
            .collect(),
    ));
    let empty = if term.is_some() {
        "No public holidays in this term."
    } else {
        "Create a term to see its public holidays."
    };
    page.set_ph_empty_text(empty.into());
}

/// R77, D5, D35: starts one background fetch for the resolved country. Does nothing without an
/// active term, with the country Off or outside the table, or while a fetch runs.
pub fn request_fetch(app: &App) {
    let Ok(Some(term)) = app.planner().active_term() else {
        return;
    };
    let Ok(Country::Code(code)) = resolved_country(app) else {
        return;
    };
    if calendar_for(&code).is_none() {
        return;
    }
    let started = with_state(|s| {
        if s.in_flight {
            return false;
        }
        s.in_flight = true;
        s.fetch = HolidayFetch::Loading;
        true
    });
    if !started {
        return;
    }
    refresh(app);
    let years = service::term_years(term.start_date, term.end_date);
    std::thread::spawn(move || {
        let result = service::fetch_events(&HttpFetcher, &code, years);
        // The event loop is gone only when the app is closing; the result is then moot.
        let _ = slint::invoke_from_event_loop(move || finish_fetch(&code, result));
    });
}

/// R77, D28: records a finished fetch on the UI thread. A result for a country that is no longer
/// the current one is dropped, then the current country is fetched. Success replaces that
/// country's rows; failure keeps them. No sync and no toast follow (R79).
fn finish_fetch(code: &str, result: Result<Vec<PublicHoliday>, String>) {
    let Some(app) = with_state(|s| {
        s.in_flight = false;
        s.app.upgrade()
    }) else {
        return;
    };
    if resolved_country(&app) != Ok(Country::Code(code.to_string())) {
        with_state(|s| s.fetch = HolidayFetch::Idle);
        app.refresh_all();
        request_fetch(&app);
        return;
    }
    if let Err(text) = &result {
        eprintln!("public holiday fetch failed: {text}");
    }
    let outcome = service::apply_fetch(&mut app.planner_mut(), code, result);
    let state = match outcome {
        FetchOutcome::Updated => HolidayFetch::Updated(app.today()),
        FetchOutcome::Failed => HolidayFetch::Failed,
    };
    with_state(|s| s.fetch = state);
    app.refresh_all();
}

/// R76, R77: saves the chosen country (an error shows in the card), drops the ticks, and fetches
/// that country's holidays. A selection shows its result in the status line, so it has no toast.
fn country_selected(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();
    let country = vm::country_from_index(page.get_ph_country_index());
    let saved = app
        .planner_mut()
        .set_public_holiday_country(&setting_value(&country));
    if let Err(e) = saved {
        page.set_ph_error(vm::error_text(&e).into());
        return;
    }
    with_state(|s| {
        s.ticked.clear();
        s.fetch = HolidayFetch::Idle;
    });
    app.refresh_all();
    request_fetch(app);
}

/// R71, R79, D34: turns the ticked public holidays that are not yet a day off into holidays
/// labelled with the public name, then syncs (R6) and toasts. Nothing ticked is an inline error;
/// when every tick is skipped nothing is written or synced and the toast still tells why.
fn add_selected(app: &App) {
    let Some(window) = app.window() else {
        return;
    };
    let page = window.global::<SettingsState>();
    let keys = with_state(|s| s.ticked.clone());
    if keys.is_empty() {
        page.set_ph_error(vm::public_holiday_tick_text().into());
        return;
    }
    let term = match app.planner().active_term() {
        Ok(Some(term)) => term,
        Ok(None) => {
            page.set_ph_error(vm::error_text(&Error::NoActiveTerm).into());
            return;
        }
        Err(e) => {
            page.set_ph_error(vm::error_text(&e).into());
            return;
        }
    };
    // Cached rows come ordered by date, then name, which is the order the plan expects.
    let cached = match cached_between(app, term.start_date, term.end_date) {
        Ok(days) => days,
        Err(text) => {
            page.set_ph_error(text.into());
            return;
        }
    };
    let ticked: Vec<PublicHoliday> = vm::public_holiday_rows(&cached)
        .iter()
        .zip(cached)
        .filter(|(row, _)| keys.contains(&row.key))
        .map(|(_, day)| day)
        .collect();
    let plan = vm::plan_add_public_holidays(&term.holidays, &ticked);
    let text = vm::public_holidays_added_text(plan.added, plan.skipped);
    if plan.added == 0 {
        with_state(|s| s.ticked.clear());
        refresh(app);
        app.toast(&text);
        return;
    }
    let saved = app.planner_mut().set_holidays(term.id, &plan.merged);
    if let Err(e) = saved {
        page.set_ph_error(vm::error_text(&e).into());
        return;
    }
    with_state(|s| s.ticked.clear());
    // R6: the new holidays delete pending sessions on their dates; every page refreshes.
    changed_then_toast(app, true, &text);
}
