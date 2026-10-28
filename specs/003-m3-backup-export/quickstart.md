# Quickstart: validate M3

## Prerequisites

- Rust stable (MSRV 1.90). No other tool is needed; tests use in-memory SQLite and never touch a
  real user database.
- The M1 and M2 tests are green before M3 starts.

## Run

```bash
# Focused: the M3 tests only
cargo test -p planner-core --test m3_backup_export
cargo test -p planner-core --test m3_backup_restore
cargo test -p planner-core --lib backup
cargo test -p planner-core --test m3_csv

# Milestone gate (docs/spec.md §7): paste the summary when claiming done
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

## What each test file proves

| File | Proves | Spec |
| --- | --- | --- |
| `crates/core/tests/m3_backup_export.rs` | document shape and version; ids and values preserved; ordering; empty-database export | R21 |
| `crates/core/tests/m3_backup_restore.rs` | export → restore into empty → export equal except `exported_at` (T30); restore replaces, not merges; ids and links preserved; bad format, newer version, invalid JSON, missing collection, dangling reference, bad date all fail with the database unchanged | R22, T30 |
| `crates/core/src/backup.rs` (unit test) | restore leaves `reminder_log` untouched | R22 |
| `crates/core/tests/m3_csv.rs` | exact headers, rows, order, `replaces_date`, quoting of comma / quote / line break, empty planner is header only | R23, T25 |

## Manual check (optional, no UI in M3)

There is no screen for these features until M4. In M4 the human runs the §9 smoke checklist, which
covers the backup, restore, and CSV buttons and the pre-restore file copy (FR-014).

## Expected outcome

All M3 tests pass; `api_surface.rs` still passes unchanged (no signature change); no `todo!("M3…")`
remains in `backup.rs` or `export.rs`; the gate is green.
