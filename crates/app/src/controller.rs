//! Controller: the one `App` on the UI thread that owns the database, the clock function, and the
//! UI state, and refreshes every page after a change (research R-D5, R-D14;
//! contracts/ui-bridge.md "Controller").

use std::cell::{Ref, RefCell, RefMut};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};

use chrono::{NaiveDate, NaiveDateTime};
use planner_core::Planner;
use slint::{ComponentHandle, Model, SharedString, VecModel};

use crate::toast::ToastQueue;
use crate::viewmodel::{self, FilterChoice};
use crate::{
    bridge, notify, pages, AboutState, AppState, AppWindow, AssessmentsState, CoursesState,
    DashboardState, RulesState, SettingsState, TimetableState, ToastState, ToastView,
};

/// R88: how often the toast timer expires toasts and moves the timer bars.
const TOAST_TICK: Duration = Duration::from_millis(50);

/// Page-level UI state that is not stored in the database (data-model.md "UI state").
#[derive(Debug, Clone)]
pub struct UiState {
    /// R6: date of the last successful sync; the rollover timer compares against it.
    pub last_sync: NaiveDate,
    /// Monday of the week shown in the timetable week view.
    pub week_start: NaiveDate,
    /// R3: the course list includes archived courses.
    pub show_archived: bool,
    /// Course whose detail panel is open.
    pub detail_course: Option<i64>,
    /// R52: courses ticked for deletion on the Courses page.
    pub selected_courses: BTreeSet<i64>,
    /// Assessments page filter.
    pub filter: FilterChoice,
    /// Session selected in the week view.
    pub selected_session: Option<i64>,
    /// Combo box index to course id, rebuilt on each refresh.
    pub course_ids: Vec<i64>,
    /// Assessments form combo index to course id, rebuilt on each assessments refresh.
    pub assessment_course_ids: Vec<i64>,
    /// R36: assessments form "Counts toward" choices after "Not counted": component id and its
    /// effective item max, rebuilt with the course choice.
    pub assessment_components: Vec<(i64, Option<f64>)>,
    /// R22: backup file picked for restore, waiting for confirmation.
    pub pending_restore: Option<PathBuf>,
    /// First day of the month shown in the dashboard mini calendar (§6 mini calendar change).
    pub calendar_month: NaiveDate,
    /// Day shown in the dashboard day panel; today on start (§6 mini calendar change).
    pub calendar_selected: NaiveDate,
}

impl UiState {
    /// Initial state for a session that starts on `today`.
    pub fn new(today: NaiveDate) -> Self {
        UiState {
            last_sync: today,
            week_start: viewmodel::week_start(today),
            show_archived: false,
            detail_course: None,
            selected_courses: BTreeSet::new(),
            filter: FilterChoice::Upcoming,
            selected_session: None,
            course_ids: Vec::new(),
            assessment_course_ids: Vec::new(),
            assessment_components: Vec::new(),
            pending_restore: None,
            calendar_month: viewmodel::month_start(today),
            calendar_selected: today,
        }
    }

    /// R6, §6 mini calendar change: on a date rollover from `old_today` to `new_today`, a day
    /// panel that was showing the old today follows to the new today, and so does the month.
    /// Any other selection is kept.
    pub fn roll_calendar(&mut self, old_today: NaiveDate, new_today: NaiveDate) {
        if self.calendar_selected == old_today {
            self.calendar_selected = new_today;
            self.calendar_month = viewmodel::month_start(new_today);
        }
    }
}

/// The running app: database, clock, window, and UI state (research R-D5).
pub struct App {
    planner: RefCell<Planner>,
    db_path: PathBuf,
    clock: fn() -> NaiveDateTime,
    window: slint::Weak<AppWindow>,
    state: RefCell<UiState>,
    /// R88: the toasts on screen.
    toasts: RefCell<ToastQueue>,
    /// R88: monotonic start of the app; the queue gets the time elapsed since it. Not the wall
    /// clock, so toast times never depend on the date.
    started: Instant,
    /// R88: the model behind `ToastState.toasts`; rows are updated in place while the set of
    /// toasts stays the same, so the close button keeps keyboard focus.
    toast_model: Rc<VecModel<ToastView>>,
    /// R88: expires toasts and moves the timer bars; stopped when the app is dropped.
    toast_timer: slint::Timer,
}

impl App {
    /// Builds the app around an open database. `clock` is `main.rs::local_now`, the only clock
    /// read (§1).
    pub fn new(
        planner: Planner,
        db_path: PathBuf,
        clock: fn() -> NaiveDateTime,
        window: &AppWindow,
    ) -> Rc<Self> {
        let today = clock().date();
        let toast_model = Rc::new(VecModel::default());
        window
            .global::<ToastState>()
            .set_toasts(toast_model.clone().into());
        Rc::new(App {
            planner: RefCell::new(planner),
            db_path,
            clock,
            window: window.as_weak(),
            state: RefCell::new(UiState::new(today)),
            toasts: RefCell::new(ToastQueue::new()),
            started: Instant::now(),
            toast_model,
            toast_timer: slint::Timer::default(),
        })
    }

    /// R88: connects the cross button and starts the toast timer (about 50 ms).
    pub fn start_toasts(self: &Rc<Self>) {
        if let Some(window) = self.window() {
            let weak = Rc::downgrade(self);
            window.global::<ToastState>().on_dismiss(move |id| {
                if let (Some(app), Ok(id)) = (weak.upgrade(), u64::try_from(id)) {
                    app.toasts.borrow_mut().close(id);
                    app.show_toasts();
                }
            });
        }
        let weak = Rc::downgrade(self);
        self.toast_timer
            .start(slint::TimerMode::Repeated, TOAST_TICK, move || {
                if let Some(app) = weak.upgrade() {
                    app.show_toasts();
                }
            });
    }

    /// R88, R89: shows `text` as a toast. The same text on screen restarts its timer; errors never
    /// go through here, they stay inline (`show_message`).
    pub fn toast(&self, text: &str) {
        let now = self.started.elapsed();
        self.toasts.borrow_mut().add(text, now);
        self.show_toasts();
    }

    /// R88: expires finished toasts and pushes the visible ones, with their bar fractions, to the
    /// window.
    fn show_toasts(&self) {
        let now = self.started.elapsed();
        let mut queue = self.toasts.borrow_mut();
        queue.expire(now);
        let views: Vec<ToastView> = queue
            .visible(now)
            .into_iter()
            .map(|toast| ToastView {
                id: i32::try_from(toast.id).unwrap_or(i32::MAX),
                text: toast.text.as_str().into(),
                remaining: queue.remaining(toast, now),
            })
            .collect();
        drop(queue);
        let same_set = views.len() == self.toast_model.row_count()
            && views.iter().enumerate().all(|(i, view)| {
                self.toast_model
                    .row_data(i)
                    .is_some_and(|row| row.id == view.id && row.text == view.text)
            });
        if same_set {
            for (i, view) in views.into_iter().enumerate() {
                self.toast_model.set_row_data(i, view);
            }
        } else {
            self.toast_model.set_vec(views);
        }
    }

    /// Current local date and time, read fresh on every call.
    pub fn now(&self) -> NaiveDateTime {
        (self.clock)()
    }

    /// Current local date.
    pub fn today(&self) -> NaiveDate {
        self.now().date()
    }

    /// Shared borrow of the database. Drop it before calling [`App::changed`].
    pub fn planner(&self) -> Ref<'_, Planner> {
        self.planner.borrow()
    }

    /// Mutable borrow of the database. Drop it before calling [`App::changed`].
    pub fn planner_mut(&self) -> RefMut<'_, Planner> {
        self.planner.borrow_mut()
    }

    /// R24: path of the open database.
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// The window, or `None` once it has been closed.
    pub fn window(&self) -> Option<AppWindow> {
        self.window.upgrade()
    }

    /// Mutable borrow of the UI state. Drop it before calling [`App::changed`] or
    /// [`App::refresh_all`].
    pub fn state(&self) -> RefMut<'_, UiState> {
        self.state.borrow_mut()
    }

    /// R6: generates sessions up to `today` and records the sync date.
    pub fn sync(&self, today: NaiveDate) -> planner_core::Result<()> {
        self.planner_mut().sync_sessions(today)?;
        self.state().last_sync = today;
        Ok(())
    }

    /// R6: after a committed mutation, syncs with today when `sync` is true, then refreshes every
    /// page. A sync error is shown in the current page's `message`; the mutation stands and the
    /// next trigger retries the sync.
    pub fn changed(&self, sync: bool) {
        let sync_error = if sync {
            self.sync(self.today()).err()
        } else {
            None
        };
        self.refresh_all();
        if let Some(e) = sync_error {
            self.show_message(&viewmodel::error_text(&e));
        }
    }

    /// R6: called by the 60-second timer. When the date has changed since the last sync, syncs
    /// with the new date and refreshes every page; otherwise refreshes only the pages whose lists
    /// depend on `now` (awaiting attendance, Attended/Absent enablement, overdue), leaving the
    /// Settings forms untouched while the student types. Then checks reminders with the same
    /// `now` (FR-006).
    pub fn tick(&self) {
        let now = self.now();
        let last_sync = self.state().last_sync;
        if viewmodel::date_rolled_over(last_sync, now) {
            self.state().roll_calendar(last_sync, now.date());
            let sync_error = self.sync(now.date()).err();
            self.refresh_all();
            if let Some(e) = sync_error {
                self.show_message(&viewmodel::error_text(&e));
            }
        } else {
            pages::dashboard::refresh(self);
            pages::timetable::refresh(self);
            pages::courses::refresh(self);
            pages::assessments::refresh(self);
        }
        self.check_reminders(now);
    }

    /// FR-006, FR-008, §7: after the R6 work of [`App::tick`], shows a desktop notification for
    /// each reminder that is due at `now`, in core's order. When the notifications switch is off,
    /// core is not called (US3.2). An error is shown in the current page's message, never a panic.
    fn check_reminders(&self, now: NaiveDateTime) {
        // Each borrow of the planner ends with its `let` statement, before any message is shown.
        let settings = self.planner().settings();
        let enabled = match settings {
            Ok(settings) => settings.notifications_enabled,
            Err(e) => {
                self.show_message(&viewmodel::error_text(&e));
                return;
            }
        };
        if !enabled {
            return;
        }
        let due = self.planner_mut().due_reminders(now);
        match due {
            Ok(reminders) => reminders.iter().for_each(notify::show),
            Err(e) => self.show_message(&viewmodel::error_text(&e)),
        }
    }

    /// §5 Settings theme: shows the saved theme (on start and after a restore). A read error is
    /// shown in the current page's message and the theme stays as it is.
    pub fn load_theme(&self) {
        let Some(window) = self.window() else {
            return;
        };
        let settings = self.planner().settings();
        match settings {
            Ok(settings) => bridge::apply_theme(&window, settings.theme),
            Err(e) => self.show_message(&viewmodel::error_text(&e)),
        }
    }

    /// §6 "Appearance:": the switch segment `index` was clicked. Applies the theme at once, then
    /// saves it with the other settings unchanged. A save error is shown in the current page's
    /// message; the theme stays applied for this session.
    pub fn select_theme(&self, index: i32) {
        let (Some(window), Some(theme)) = (self.window(), viewmodel::theme_from_index(index))
        else {
            return;
        };
        bridge::apply_theme(&window, theme);
        let current = self.planner().settings();
        let saved = current.and_then(|settings| {
            self.planner_mut()
                .save_settings(&planner_core::Settings { theme, ..settings })
        });
        if let Err(e) = saved {
            self.show_message(&viewmodel::error_text(&e));
        }
    }

    /// Reloads every page from the database with a fresh `now` (research R-D14).
    pub fn refresh_all(&self) {
        pages::dashboard::refresh(self);
        pages::timetable::refresh(self);
        pages::courses::refresh(self);
        pages::assessments::refresh(self);
        pages::settings::refresh(self);
        pages::about::refresh(self);
        pages::rules::refresh(self);
        pages::import::refresh(self);
    }

    /// Shows `text` in the `message` property of the page that is currently visible.
    pub fn show_message(&self, text: &str) {
        let Some(window) = self.window() else {
            return;
        };
        let text = SharedString::from(text);
        match window.global::<AppState>().get_current_page() {
            1 => window.global::<TimetableState>().set_message(text),
            2 => window.global::<CoursesState>().set_message(text),
            3 => window.global::<AssessmentsState>().set_message(text),
            4 => window.global::<SettingsState>().set_message(text),
            5 => window.global::<AboutState>().set_message(text),
            6 => window.global::<RulesState>().set_message(text),
            _ => window.global::<DashboardState>().set_message(text),
        }
    }
}
