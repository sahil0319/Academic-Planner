# Checkpoint

- Milestone: M3 done (specs/003-m3-backup-export, T001–T013, converge clean). Next milestone: M4
- Last green gate: 2dfaf5c (fmt ok, clippy -D warnings ok, cargo test --workspace: 124 tests pass incl. T25, T30, r21_/r22_/r23_ backup and CSV tests).
- Next: M4 per docs/spec.md §7: ui/theme.slint from DESIGN.md first, then full UI (§6); design skills allowed at M4 start/end only (§12.0).
- M4 carry-over (human decision 2026-10-29, M3 FR-014): the R22 pre-restore copy of the DB file to {data_dir}/backups/pre-restore-{YYYYMMDD-HHMM}.sqlite is built in the app restore flow, before calling restore_json.
- M4 carry-over (memory): exam periods are entered as holidays; raise a date-range holiday input at M4.
- M3 notes:
  - Backup JSON: flags as booleans, nullable keys must be present (null ok), exported_at = YYYY-MM-DDTHH:MM.
  - restore_json validates the whole document before writing; constraint violations -> Error::Validation; reminder_log untouched.
  - CSV: explicit CRLF terminator (csv crate default is LF); completed = true/false.
  - Remaining stub: due_reminders (M5).
- Notes: rustc 1.90; Slint 1.16; rusqlite 0.41; packages planner-core / app.
- Open questions: none
