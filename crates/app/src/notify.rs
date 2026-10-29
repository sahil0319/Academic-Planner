//! Desktop notifications for due reminders (docs/spec.md §2, §7; FR-006–FR-008;
//! contracts/app-notify.md). The only module in the app that uses `notify_rust`.

use planner_core::Reminder;

/// FR-006: shows one desktop notification for a due reminder, off the UI thread.
///
/// FR-008: the call returns at once; a display error is written to stderr and dropped, with no
/// retry, no dialog, and no panic.
///
/// R31: any `HH:MM` in the reminder text is shown in 12-hour form (the stored text stays 24-hour).
pub fn show(reminder: &Reminder) {
    let title = crate::viewmodel::times_to_12h(&reminder.title);
    std::thread::spawn(move || {
        if let Err(e) = notify_rust::Notification::new()
            .summary("Academic Planner")
            .body(&title)
            .show()
        {
            eprintln!("notification failed: {e}");
        }
    });
}
