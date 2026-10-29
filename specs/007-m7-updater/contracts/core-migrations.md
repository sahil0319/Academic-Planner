# Contract: Core migrations and pre-migration backup (R25, R26, R30)

## Public surface (unchanged except one error variant)

- `Planner::open(path: &Path) -> Result<Planner>`: same signature. Now applies R26.
- `Planner::open_in_memory() -> Result<Planner>`: same; never writes a backup.
- `SCHEMA_VERSION`: unchanged (1).
- New: `Error::MigrationFailed { version, at, backup, source }` (see data-model.md).

## Private module `crates/core/src/migrate.rs`

```rust
/// R25, R26: applies `migrations[found..]` to `conn`; writes a pre-migration copy first when
/// an existing database (1 <= found < migrations.len()) is upgraded.
pub(crate) fn run(conn: &mut Connection, db_path: Option<&Path>, migrations: &[&str]) -> Result<()>;

/// R26: base name, or first free `-2`, `-3`, … suffix among `existing`. Pure; no clock.
fn backup_file_name(from: i64, to: i64, existing: &[String]) -> String;
```

`Planner::init` keeps the `foreign_keys` pragma, the newer-schema refusal (before anything is
written, including the WAL switch), and the WAL switch, then calls `migrate::run` with
`MIGRATIONS`. The refusal may move into `run` as long as it still happens before any write.

### Behavior

1. `found > len` → `Error::SchemaTooNew { found, supported: len }`; nothing written, no copy.
2. `found == len` → no-op (no copy).
3. `found == 0` → migrate, no copy.
4. `1 ≤ found < len` and `db_path` is `Some`:
   1. `create_dir_all({parent}/backups)`; list existing names; choose `backup_file_name(found, len, …)`.
   2. `VACUUM INTO '<path>'`. On error return it (`Io`/`Db`); no migration runs.
5. For each pending `version` (found+1 ..= len): one transaction runs the SQL and sets
   `user_version = version`, then commits. On error: roll back that transaction and return
   `Error::MigrationFailed { version, at: version - 1, backup, source }`.

## Tests

Unit tests in `migrate.rs` (`#[cfg(test)]`, `tempfile::TempDir`). Test list:
`[include_str!("migrations/001_init.sql"), TEST_V2]`.

- `t31_upgrade_writes_pre_migrate_copy_equal_to_old_db`: make a v1 DB file with the real list,
  add rows (a term, a course, a setting), snapshot (`user_version`, `sqlite_master`, all rows);
  open with the 2-list → `backups/pre-migrate-v1-to-v2.sqlite` exists and its snapshot equals the
  old one; the DB is v2.
- `t31_reopen_current_db_makes_no_new_copy`: open again with the 2-list → still exactly one file
  in `backups`.
- `t31_fresh_db_makes_no_copy`: a new file opened with the 2-list → no `backups` entries.
- `t31_existing_copy_name_gets_suffix` (R26 suffix): pre-create `pre-migrate-v1-to-v2.sqlite`
  → copy is `pre-migrate-v1-to-v2-2.sqlite`; the pre-existing file is unchanged.
- `t32_failed_second_migration_leaves_db_at_v1`: list `[001, "CREATE TABLE t32_partial(x); CREATE TABLE term(x);"]`
  (second statement fails) → `Err(MigrationFailed { version: 2, at: 1, backup: Some(p), .. })`;
  `user_version` still 1; snapshot equals the pre-open snapshot (no `t32_partial`); the error's
  Display contains `p`'s display string.
- `backup_file_name` unit tests: base free; base taken → `-2`; base and `-2` taken → `-3`;
  unrelated names ignored.

Integration tests in `crates/core/tests/migrations.rs` (real list, temp folder, public API):

- `t31_fresh_file_db_makes_no_backup`, `t31_current_file_db_reopen_makes_no_backup`.
- Existing `t20_*` and `t26_*` keep passing (t26: refusal still writes no copy).
