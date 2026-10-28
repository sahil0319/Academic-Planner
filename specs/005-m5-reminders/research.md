# Research: M5 Reminders

All Technical Context items are resolved. `docs/spec.md` §1 fixes the notification crate
(`notify-rust`), the clock rule, and naive local time; §3 fixes `reminder_log` and the setting
defaults; §5 fixes the `due_reminders` signature. The items below are implementation choices made
inside those limits. None changes a rule, the schema, or the §5 API.

## R-1 notify-rust version and Linux system libraries

- **Decision**: `notify-rust = "4.18.2"` in `crates/app/Cargo.toml` with default features.
- **Rationale**: 4.18.2 is the latest release (checked 2026-10-29 with `cargo info`); its
  `rust-version` is 1.89, under the workspace's 1.90. The default feature `z` uses `zbus`, a
  pure-Rust D-Bus client, so Linux CI needs no `libdbus-1-dev`. Resolver 3 keeps transitive
  versions MSRV-compatible.
- **Check the builder runs**: `cargo build -p app` on Rust 1.90, then
  `cargo tree -p app -i libdbus-sys` must report no match. If `libdbus-sys` appears, add
  `libdbus-1-dev` to the apt line in `.github/workflows/ci.yml`; otherwise leave CI unchanged.
  `Cargo.lock` must be committed because CI runs with `--locked`.
- **Alternatives considered**: feature `d` (libdbus bindings) needs a system library on Linux CI;
  rejected. `z-with-tokio` pulls a tokio runtime the app does not otherwise need; rejected.

## R-2 Due window and missed reminders

- **Decision**: with event time `T` (session start, or assessment `due_at`) and lead `m` minutes,
  `fire_at = T - m`. A reminder is due when `fire_at <= now < max(T, fire_at + 1 minute)`.
- **Rationale**: this is FR-004 in one expression: for `m >= 1` the end is `T`; for `m = 0` the end
  is `T + 1 minute`, so the 60-second timer still catches it once. `now` may carry seconds (the app
  passes local time); `fire_at` is always on a whole minute. Anything outside the window is neither
  returned nor logged, so a reminder missed while the app was closed is skipped (US1.4, US2.4).
- **Alternatives considered**: firing missed reminders late on the next check; rejected by the
  spec (missed reminders are skipped).

## R-3 Dedupe and atomicity

- **Decision**: inside one `rusqlite` transaction, compute the due candidates, run
  `INSERT OR IGNORE INTO reminder_log (kind, ref_id, fire_at)` for each, and keep a candidate only
  when the insert changed one row; commit, then sort and return.
- **Rationale**: the `(kind, ref_id, fire_at)` primary key (§3) is the dedupe key, so a second call
  in the window inserts nothing and returns nothing (FR-001). If the transaction fails, nothing is
  logged and an error is returned, so no reminder is lost silently. `fire_at` is stored with the
  existing `db::fmt_datetime` format (`YYYY-MM-DDTHH:MM`, §1).
- **Consequence (accepted, from the spec's key choice)**: changing an assessment's due time or
  reminder minutes, or `class_reminder_minutes`, produces a new `fire_at`, which may fire once more
  inside the new window (US2.3).
- **Alternatives considered**: `SELECT` the log then `INSERT`; rejected as two statements where one
  suffices.

## R-4 Candidate query

- **Decision**: sessions: `status = 'pending'`, course not archived, course in the active term
  (`term.is_active = 1`), both kinds (regular and makeup). Assessments: `completed = 0`,
  `remind_minutes_before IS NOT NULL`, course not archived, course in the active term. To keep the
  scan small, SQL may pre-filter to events whose `T >= now - 1 minute` (no due window ends later
  than `T + 1 minute`); the exact window check is done in Rust with `chrono`.
- **Rationale**: FR-002, FR-003, R1 (the active term scopes sessions and courses). Session room for
  the title is `coalesce(session.room, course.room)` (the session's stored room is a snapshot, R7;
  sync already copies slot room else course room).
- **Lead time source**: sessions use `settings().class_reminder_minutes` (read in the same call);
  assessments use their own `remind_minutes_before`. Lead arithmetic uses `chrono::Duration` on
  naive datetimes; an out-of-range result (`checked_sub` is `None`) skips that candidate.
- **Alternatives considered**: computing everything in SQL with `datetime()` arithmetic; rejected
  because the stored format `YYYY-MM-DDTHH:MM` and the minute-0 window are clearer in Rust.

## R-5 Titles and order

- **Decision**: session title `"{code} {name} at {HH:MM}"`, plus `" in {room}"` when the room is
  not null or blank. Assessment title `"{code} {Kind}: {title} due {YYYY-MM-DD HH:MM}"` where
  `{Kind}` is `Quiz`, `Exam`, `Assignment`, or `Other` (the same words the UI shows). Sort key:
  `(fire_at, kind rank with Session = 0 and Assessment = 1, ref_id)`.
- **Rationale**: FR-005. The title is a field of the §5 `Reminder` type, so it is built in core,
  which has no dependency on the app's `viewmodel::assessment_kind_label`; core keeps a private
  mapping with the same words. Example on the fixture: `CSE101 Intro to Programming at 09:00 in
  A-101`.
- **Alternatives considered**: lower-case stored kind (`quiz`); rejected as less readable in a
  notification (wording choice).

## R-6 App side: notify.rs, the timer hook, and the settings switch

- **Decision**:
  - `crates/app/src/notify.rs`: `pub fn show(reminder: &Reminder)` clones the title and spawns a
    detached `std::thread` that builds `notify_rust::Notification` with summary
    `"Academic Planner"` and body = the title, calls `.show()`, and ignores (or `eprintln!`s) the
    error. No retry (US3.3).
  - `App::tick()` keeps its R6 work first (rollover sync, or the now-dependent refreshes), then
    calls a new private `App::check_reminders(now)`: read `settings()`; if
    `notifications_enabled` is false, return without calling core (US3.2); else call
    `due_reminders(now)` and `notify::show` each result in order. A core error is shown with the
    existing `show_message(&viewmodel::error_text(&e))` and never panics (FR-008).
  - The check runs only from the timer (not at app start); the first check is at most 60 seconds
    after launch, which is inside every due window (lead `>= 1` gives at least one minute; lead 0
    gives exactly one minute).
- **Rationale**: a D-Bus call can wait for service activation when no notification daemon runs;
  doing it off the UI thread meets "must not block the UI" (FR-008). The reminder is already logged
  before `show`, so a failed display is not retried. The sync runs before the check so sessions of
  a new day exist. The app adds no formatting (core supplies the title, and the summary is a
  constant), so no new viewmodel function or unit test is needed; the settings gate is a single
  `if` in the controller, which needs a window to run and is covered by the manual check.
- **Alternatives considered**: showing on the UI thread (simplest, but can block on a missing
  daemon); a viewmodel helper for the gate (adds a function with no formatting in it); checking
  `notifications_enabled` inside core (would log reminders while the switch is off, which US3.2
  forbids, and core would differ from the §5 description "returns due reminders").

## R-7 Test layout

- **Decision**: one integration file `crates/core/tests/m5_reminders.rs` using `mod common;` and the
  §8 fixture (`synced_fixture`, `assessment_input`, `makeup_input`, `dt`). Test names start with
  `m5_` (no T-IDs exist for M5) and each doc comment cites the FR id and §5. The tests observe the
  log only through `due_reminders` (a second call returns nothing), so no test-only API is added.
- **Rationale**: matches the M2/M3 file layout; §5 stays frozen.
