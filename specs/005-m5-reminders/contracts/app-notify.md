# Contract: app notifications (M5)

## `crates/app/src/notify.rs` (§2)

```rust
/// FR-006: shows one desktop notification for a due reminder, off the UI thread.
pub fn show(reminder: &Reminder);
```

- Summary `"Academic Planner"`, body `reminder.title`.
- Runs `notify_rust::Notification::show()` on a detached `std::thread`; the call returns at once.
- A display error is ignored (optionally written to stderr); no retry, no dialog, no panic (US3.3,
  FR-008).
- No other code in the app calls `notify_rust`.

## Timer hook (`crates/app/src/controller.rs`, `main.rs`)

- The existing 60-second `slint::Timer` in `main.rs` keeps calling `App::tick()`; no second timer.
- `App::tick()` order: existing R6 work (rollover sync + refresh, or now-dependent refreshes), then
  the reminder check with the same `now`.
- Reminder check:
  1. `settings()`; on error show the message and stop.
  2. `notifications_enabled == false` → return; `due_reminders` is not called (US3.2).
  3. `due_reminders(now)`; on error show `viewmodel::error_text(&e)` in the current page message.
  4. `notify::show(&r)` for each reminder, in the returned order.
- The planner borrow is released before showing messages (existing `App` borrow rule).
- No notification when the app is closed: there is no background service or autostart (FR-007, §10).

## Dependency

`crates/app/Cargo.toml`: `notify-rust = "4.18.2"` (default features; research R-1). Core gets no
new dependency.
