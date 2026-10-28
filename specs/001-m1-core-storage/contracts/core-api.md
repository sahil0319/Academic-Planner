# Contract: `planner_core` public API (frozen after M1)

Signatures are `docs/spec.md` §5 exactly (including the 2026-10-28 `now` parameter on the two
session readers). Types are in [../data-model.md](../data-model.md). `Result<T>` is
`std::result::Result<T, planner_core::Error>`.

## Free functions

| Item | M1 |
|---|---|
| `resolve_db_path(env_override: Option<OsString>, data_dir: Option<PathBuf>) -> Result<PathBuf>` | ✅ R24 (pure) |
| `default_db_path() -> Result<PathBuf>` | ✅ R24 (reads `ACADEMIC_PLANNER_DB`, `ProjectDirs`) |

## Constants and trait impls

| Item | M1 |
|---|---|
| `DB_ENV_VAR: &str = "ACADEMIC_PLANNER_DB"` | ✅ R24 |
| `DB_FILE_NAME: &str = "planner.sqlite"` | ✅ R24 |
| `SCHEMA_VERSION: i64` (number of embedded migrations; 1 in M1) | ✅ R25 |
| `impl Default for Settings` (15, 7, true) | ✅ §3 defaults |

## `impl Planner`

| Method | M1 | Rules / notes |
|---|---|---|
| `open(path: &Path) -> Result<Self>` | ✅ | R24 creates parent dir; R25 migrate / no-op / refuse newer |
| `open_in_memory() -> Result<Self>` | ✅ | R25 |
| `active_term(&self) -> Result<Option<Term>>` | ✅ | R1 |
| `save_term(&mut self, input: TermInput) -> Result<Term>` | ✅ | R1 sets active; R2 dates |
| `set_holidays(&mut self, term_id: i64, days: &[Holiday]) -> Result<()>` | ✅ | replaces set; `NotFound` if no term |
| `create_course(&mut self, input: CourseInput) -> Result<Course>` | ✅ | R1 active term; R2 |
| `update_course(&mut self, id: i64, input: CourseInput) -> Result<Course>` | ✅ | R2 incl. duplicate code |
| `set_archived(&mut self, id: i64, archived: bool) -> Result<()>` | ✅ | R3 |
| `list_courses(&self, include_archived: bool) -> Result<Vec<Course>>` | ✅ | R1, R3; ordered by code |
| `add_slot(&mut self, input: SlotInput) -> Result<Slot>` | ✅ | R2 times |
| `change_slot(&mut self, id: i64, input: SlotInput, effective_from: NaiveDate) -> Result<Slot>` | stub | M2: R7 |
| `end_slot(&mut self, id: i64, last_date: NaiveDate) -> Result<()>` | stub | M2: R7, T27 |
| `list_slots(&self, on: NaiveDate) -> Result<Vec<Slot>>` | ✅ | research D9 |
| `sync_sessions(&mut self, today: NaiveDate) -> Result<SyncReport>` | stub | M2: R4, R5 |
| `sessions_between(&self, from: NaiveDate, to: NaiveDate, now: NaiveDateTime) -> Result<Vec<SessionView>>` | stub | M2 |
| `sessions_for_course(&self, course_id: i64, now: NaiveDateTime) -> Result<Vec<SessionView>>` | stub | M2 |
| `set_status(&mut self, id: i64, status: Status, now: NaiveDateTime) -> Result<()>` | stub | M2: R11, R12 |
| `reschedule(&mut self, id: i64, makeup: MakeupInput) -> Result<Session>` | stub | M2: R9 |
| `add_makeup(&mut self, makeup: MakeupInput) -> Result<Session>` | stub | M2: R10 |
| `attendance_summary(&self, course_id: i64) -> Result<AttendanceSummary>` | stub | M2: R13–R15 |
| `all_summaries(&self) -> Result<Vec<AttendanceSummary>>` | stub | M2: R13–R15 |
| `dashboard(&self, now: NaiveDateTime) -> Result<Dashboard>` | stub | M2: R16–R20 |
| `create_assessment(&mut self, input: AssessmentInput) -> Result<Assessment>` | ✅ | R2 title (FR-013a) |
| `update_assessment(&mut self, id: i64, input: AssessmentInput) -> Result<Assessment>` | ✅ | R2 title |
| `set_completed(&mut self, id: i64, done: bool) -> Result<()>` | ✅ | |
| `delete_assessment(&mut self, id: i64) -> Result<()>` | ✅ | |
| `list_assessments(&self, filter: AssessmentFilter) -> Result<Vec<Assessment>>` | partial | `All`, `Completed` ✅; `Upcoming`/`Overdue` stub (M2: R18, R19) |
| `settings(&self) -> Result<Settings>` | ✅ | defaults per §3 |
| `save_settings(&mut self, s: &Settings) -> Result<()>` | ✅ | |
| `export_json(&self, now: NaiveDateTime) -> Result<String>` | stub | M3: R21 |
| `restore_json(&mut self, json: &str) -> Result<()>` | stub | M3: R22 |
| `export_attendance_csv<W: Write>(&self, w: W) -> Result<()>` | stub | M3: R23 |
| `export_assessments_csv<W: Write>(&self, w: W) -> Result<()>` | stub | M3: R23 |
| `due_reminders(&mut self, now: NaiveDateTime) -> Result<Vec<Reminder>>` | stub | M5 |

Every mutation commits before returning (§1 "Saving"). No method panics on user input; only stubs
panic via `todo!()`.
