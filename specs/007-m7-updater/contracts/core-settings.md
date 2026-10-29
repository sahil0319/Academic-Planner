# Contract: `check_updates_on_start` setting (§3, §13.1 UI, R29)

## `Settings` (public, `model.rs`)

```rust
pub struct Settings {
    pub class_reminder_minutes: u32,
    pub upcoming_window_days: u32,
    pub notifications_enabled: bool,
    pub theme: ThemeMode,
    /// §3, R29: check GitHub for an update once per launch. Default on.
    pub check_updates_on_start: bool,
}
```

`Settings::default().check_updates_on_start == true`.

## Storage (`db.rs`)

- Key constant `SETTING_CHECK_UPDATES = "check_updates_on_start"` (`pub(crate)`, used by
  `backup.rs`).
- `settings()`: `"1"` → true, `"0"` → false, missing → `true`, other → `Error::Corrupt`.
- `save_settings()`: upserts `"1"`/`"0"` in the same transaction as the other keys.

## Backup (`backup.rs`, R21/R22)

- Export: unchanged code; the row is exported with the other `setting` rows.
- Restore validation: if `settings.check_updates_on_start` is present it must be exactly `"0"` or
  `"1"`; otherwise `Error::Validation` and nothing is written. Missing is valid and reads as on.

## Tests (`crates/core/tests/settings_updates.rs`)

- `check_updates_fresh_db_is_on`
- `check_updates_saved_off_reads_back_and_keeps_other_settings`
- `check_updates_export_contains_key`
- `check_updates_backup_without_key_restores_as_on`
- `check_updates_restore_rejects_bad_value` (e.g. `"yes"`, `"true"`) and leaves data unchanged

## Existing code to update

- Every full `Settings { … }` literal in `crates/core/tests/*.rs` and
  `crates/app/src/viewmodel/tests.rs` adds the field (or uses `..Settings::default()`).
- `m3_backup_export.rs` expected `settings` objects add `"check_updates_on_start": "1"`/`"0"` where
  settings were saved.
- App: `viewmodel::parse_settings_form` keeps returning a `Settings` (default for the new field);
  `pages/settings.rs::save_settings` keeps the current `theme` **and** `check_updates_on_start`.
