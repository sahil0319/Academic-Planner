# Contract: M3 behavior of the stubbed §5 methods

The signatures below are exactly the ones in `docs/spec.md` §5 and in the M1 stubs. They are frozen
by `crates/core/tests/api_surface.rs`. M3 adds no public item and changes none. The R22 pre-restore
file copy has no core API (spec FR-014): the M4 app layer copies the database file before calling
`restore_json`.

## `export_json`

```rust
pub fn export_json(&self, now: NaiveDateTime) -> Result<String>;
```

- R21. Returns one JSON document with the keys, order, and value types in
  [data-model.md](../data-model.md). `exported_at` is `fmt_datetime(now)`.
- Pretty printing is allowed but not required; tests parse the text with `serde_json`, they do not
  compare raw text with a fixed layout. Two exports of unchanged data with the same `now` are
  byte-identical.
- Empty database: every collection is `[]`, `settings` holds the default settings rows.
- Errors: `Error::Db` / `Error::Corrupt` on read failure. Never touches the clock or a file.

## `restore_json`

```rust
pub fn restore_json(&mut self, json: &str) -> Result<()>;
```

- R22. Validates, then replaces all data in `term`, `holiday`, `course`, `slot`, `session`,
  `assessment`, `setting` in one transaction; `reminder_log` untouched; ids preserved.
- `Error::Validation` when: text is not valid JSON or the root is not an object; `format` missing or
  not `"academic-planner-backup"`; `schema_version` missing, not an integer, or greater than
  `SCHEMA_VERSION`; a collection is missing or a row is missing a required field or has a wrongly
  typed field; a date, time, or date-time text does not parse; a SQLite constraint (CHECK, UNIQUE,
  NOT NULL, PRIMARY KEY, FOREIGN KEY at commit) rejects the data.
- On every error the database is exactly as before the call (export before and after is identical).
- `schema_version` less than or equal to `SCHEMA_VERSION` is accepted.
- No sync, no clock, no file access.

## `export_attendance_csv`

```rust
pub fn export_attendance_csv<W: Write>(&self, w: W) -> Result<()>;
```

- R23. Header `date,start,end,course_code,course_name,kind,status,replaces_date,note`, then one
  record per session as in [data-model.md](../data-model.md). The writer is flushed on success.
- Errors: `Error::Db` on read failure; `Error::Io` on write failure.

## `export_assessments_csv`

```rust
pub fn export_assessments_csv<W: Write>(&self, w: W) -> Result<()>;
```

- R23. Header `course_code,kind,title,due_at,completed,notes`, then one record per assessment as in
  [data-model.md](../data-model.md). Same error behavior as above.
