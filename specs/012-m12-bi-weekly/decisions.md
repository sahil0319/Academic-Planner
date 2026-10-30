# M12 decisions (overnight, one line each)

- T001/T002: `SessionView.biweekly` is read for real from the slot join (not a constant); existing "latest version" test asserts (migrations, m9_backup, m3_backup_export, m9_migration incl. copy name v1-to-v3) bumped to 3. Backup `SlotRow` does not carry `week_parity` yet (T009).
- T009: backup `SlotRow.week_parity` is `Option<WeekParity>` with serde default (absent = weekly); `SCHEMA_VERSION` follows the migration count (3), so no extra constant was needed. Slot CSV is not part of R23, so nothing changed there.
