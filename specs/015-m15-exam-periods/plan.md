# Implementation Plan: M15 Mid exam and final exam periods

**Spec directory**: `specs/015-m15-exam-periods` (work happens on the current branch) | **Date**: 2026-10-31
**Spec**: `docs/spec.md` §13.10 (R71-R75, T71-T74). Related: R4, R5, R6 (sync), R21, R22 (backup),
R23 (CSV, unchanged), R25 (migrations), §13.13 R88, R89, R89a (toasts). Visuals: `DESIGN.md`
"Calendar days off" (colours), "Mini calendar", "Day panel lines", "Exam period cards (Settings)".
**Decisions**: [decisions.md](decisions.md) (D1-D8 fixed by the router; D9 onward by the planner).

## Summary

A day off gets a kind: `holiday`, `midterm`, or `final`. Every kind skips classes exactly as a
holiday does today; the kind only changes how the day is listed in Settings and drawn on the
dashboard calendar. Four parts:

1. **Core schema and model (R71, R72).** Migration `003_day_off_kind.sql` (spec number 004,
   renumbered to the next free number, D1) adds `holiday.kind`. `DayOffKind` and `Holiday.kind`
   in `model.rs`; `set_holidays` stores the kind and rejects a date listed twice (R71, D9, D10);
   `load_term` reads it. Sync keeps using every row of `term.holidays` (R72, no logic change).
2. **Core backup (R75).** Holiday rows carry `kind`; older backups restore every row as
   `holiday`; CSV untouched.
3. **View model (R73, R74; T74).** Kind names, fill keys, legend entries, per-kind rows,
   default labels, toast texts, the day panel line, and `CalendarDay.day_off`.
4. **UI (R73, R74).** Settings: Holidays, Mid exams, Final exams as one `DayOffCard` component.
   Dashboard: per-kind fill, three legend entries, per-kind day panel line. `Theme.*` only.

## Technical Context

**Language/Version**: Rust (workspace edition), Slint UI
**Primary Dependencies**: `rusqlite`, `chrono`, `serde`/`serde_json` (core); `slint` (app). No new dependency.
**Storage**: SQLite; one new migration (`ALTER TABLE holiday ADD COLUMN kind …`), exact SQL from §13.10.
**Testing**: `cargo test --workspace`; core integration tests on in-memory SQLite (the migration
test uses a temp file DB, as `m9_migration.rs`); app unit tests on pure view-model functions.
**Target Platform**: desktop (macOS, Windows, Linux).
**Project Type**: Cargo workspace: `crates/core` (library), `crates/app` (Slint desktop app).
**Constraints**: core reads no clock and has no Slint; `.slint` bindings only; `Theme.*` only, no
raw hex or px outside `ui/theme.slint` (comments count); square corners.
**Scale/Scope**: tens of days off per term.

No NEEDS CLARIFICATION remains; every interpretation is in decisions.md.

## Constitution Check

| Principle | Status |
|---|---|
| I. Spec is source of truth | Pass. Schema, rules, and API exactly as §13.10 (`DayOffKind`, `Holiday.kind`, `set_holidays` unchanged in signature). Migration renumbered per the spec's own note. Wording not fixed by the spec is in decisions.md. |
| II. Test-first, traceable | Pass. T71-T74 and supporting tests are tasks T001-T002, before any code; names `t71_…`; doc comments cite R71-R75. |
| III. Pure, clock-free core | Pass. No clock; every UI string from `viewmodel.rs`; `.slint` maps a kind key to a `Theme` token (a binding, like the existing holiday fill). |
| IV. Append-only migrations | Pass. New file `003_day_off_kind.sql`; 001 and 002 untouched. |
| V. Gate before done | Pass. T009 runs the gate and the theme grep. |

Re-check after design: still pass.

## Every place a Holiday is built or read

| Place | Today | M15 change |
|---|---|---|
| `crates/core/src/migrations/` | `holiday(term_id, date, label)` | new `003_day_off_kind.sql` (T003) |
| `crates/core/src/db.rs` `MIGRATIONS`, `SCHEMA_VERSION` | 2 entries | 3 entries; `SCHEMA_VERSION` follows (T003) |
| `crates/core/src/model.rs` `Holiday` | `date`, `label` | + `kind: DayOffKind`; new `DayOffKind` (T003) |
| `crates/core/src/db.rs` `set_holidays` | delete + insert `(term_id, date, label)` | R71 duplicate check, insert `kind` (T003) |
| `crates/core/src/db.rs` `load_term` | `SELECT date, label` | `SELECT date, label, kind` (T003) |
| `crates/core/src/db.rs` `DayOffKind::as_db/from_db` | — | new, beside `Status`/`SessionKind` (T003) |
| `crates/core/src/schedule.rs` `sync_in` | `term.holidays` dates skipped | unchanged logic; doc cites R72 (T004) |
| `crates/core/src/backup.rs` `HolidayRow`, export query, `apply`, `check_texts`, `restore_json` | no kind | `kind` exported, restored, version-gated (T004) |
| `crates/core/src/export.rs` CSV | no holidays | unchanged (R75 "CSV is unchanged") |
| `crates/core/src/terms.rs` | mentions holidays in a doc comment only | unchanged |
| `crates/app/src/viewmodel.rs` `expand_holiday_range` (builds `Holiday`) | `kind` absent | sets `DayOffKind::Holiday` (T005) |
| `crates/app/src/viewmodel.rs` `merge_holidays`, `remove_holiday`, `holiday_row`, `holiday_text`, `month_grid`, `day_panel_holiday_text` | kind-blind | per D12, D13, D20, D25 (T005) |
| `crates/app/src/bridge.rs` `calendar_day`, `holiday_row` | — | `day_off` key (T006); new `legend_item` (T008) |
| `crates/app/src/pages/settings.rs` refresh, `add_holiday`, `remove_holiday` | one card | three kinds (T007) |
| `crates/app/src/pages/dashboard.rs` `fill_calendar` | holiday line | day-off line, legend model (T008) |
| `crates/app/ui/types.slint` `CalendarDay`, `HolidayRow` | — | `CalendarDay.day-off` (T006); new `LegendItem` (T008) |
| `crates/app/ui/components.slint` (`SectionCard`) | title only | optional swatch (T006) |
| `crates/app/ui/settings.slint` | Holidays card | `DayOffCard` ×3 (T007) |
| `crates/app/ui/dashboard.slint` `DayCell`, legend, day panel | one fill | three fills, legend model (T008) |
| Tests with `Holiday { .. }` literals | no `kind` | tester adds `kind` (T001, T002) |

Not on this branch (no change): import (`crates/core/src/import`, `pages/import.rs`) does not touch
holidays. If the build shows another `Holiday { .. }` literal, it gets `kind: DayOffKind::Holiday`.

## Frozen interfaces

Tester and builder code against these. A change needs the router.

### Migration `crates/core/src/migrations/003_day_off_kind.sql`

Exactly the §13.10 SQL:

```sql
ALTER TABLE holiday ADD COLUMN kind TEXT NOT NULL DEFAULT 'holiday'
  CHECK (kind IN ('holiday','midterm','final'));
```

Header comment (normal SQL comment) names R71, R72 and says it is spec migration 004 renumbered.
`db.rs` `MIGRATIONS` gains `include_str!("migrations/003_day_off_kind.sql")`; `SCHEMA_VERSION`
becomes 3 automatically.

### Core model (`crates/core/src/model.rs`)

```rust
/// R71: the kind of a day off. Every kind skips classes (R72).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DayOffKind {
    #[default]
    Holiday,
    Midterm,
    Final,
}

/// R71: a day without classes (`holiday` table): a holiday, a mid exam day, or a final exam day.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Holiday {
    pub date: NaiveDate,
    pub label: Option<String>,
    pub kind: DayOffKind,
}
```

`Term.holidays` doc: "Days off of this term (every kind), sorted by date."

Crate-private, in `db.rs` beside the other `as_db`/`from_db`:

```rust
impl DayOffKind {
    pub(crate) fn as_db(self) -> &'static str;          // "holiday" | "midterm" | "final"
    pub(crate) fn from_db(s: &str) -> Option<Self>;
    /// R71 message noun: "holiday" | "mid exam day" | "final exam day".
    pub(crate) fn noun(self) -> &'static str;
}
```

### `set_holidays` (signature unchanged)

```rust
/// R71, R72, D4: replaces every day off of the term (all kinds) with `days`, in one transaction.
pub fn set_holidays(&mut self, term_id: i64, days: &[Holiday]) -> Result<()>;
```

Order: term exists (else `NotFound`, as today); then R71: walk `days` in order; on the first date
already seen, return `Error::Validation(format!("{} is already a {}", date.format("%B %-d, %Y"),
kind.noun()))` where `kind` is the kind stored for that date in this term if a row exists, else the
kind of the date's first entry in `days` (D9, D10). Nothing is written before this check. Then
`DELETE` and `INSERT INTO holiday (term_id, date, label, kind)`.

`load_term`: `SELECT date, label, kind FROM holiday WHERE term_id = ?1 ORDER BY date`; an unknown
kind text is a conversion error, handled like an unknown `Status`.

### Sync (`crates/core/src/schedule.rs`)

No logic change: `sync_in` already skips every date in `term.holidays`, which now holds all
kinds. Update its doc to cite R72 ("not a day off of any kind").

### Backup (`crates/core/src/backup.rs`)

```rust
/// R75: first schema version whose backups carry `holidays[].kind` (= the M15 migration number;
/// renumber with the migration).
const DAY_OFF_KIND_VERSION: i64 = 3;

struct HolidayRow { term_id: i64, date: String, label: Option<String>, kind: String }
```

- Export: `SELECT term_id, date, label, kind FROM holiday ORDER BY term_id, date`; JSON key order
  `term_id, date, label, kind`; `kind` is the stored text.
- `restore_json`, before deserialising: if `version >= DAY_OFF_KIND_VERSION`, every object in
  `holidays` must have a `kind` key, else ``Error::Validation("backup has the wrong shape: missing
  field `kind`")``; if `version < DAY_OFF_KIND_VERSION`, every row's `kind` is set to `"holiday"`
  (a `kind` present in an older document is ignored). `kind` deserialises with a serde default so
  older documents parse.
- `check_texts`: a `kind` that `DayOffKind::from_db` rejects is
  `bad("holiday kind", &h.kind)` ("backup has an invalid holiday kind: \"exam\"").
- `apply`: `INSERT INTO holiday (term_id, date, label, kind) VALUES (?1, ?2, ?3, ?4)`.
- CSV (`export.rs`) unchanged.

### `crates/core/tests/api_surface.rs` (under `// M15 (§13.10)`)

```rust
let _kinds = [DayOffKind::Holiday, DayOffKind::Midterm, DayOffKind::Final];
let _ = Holiday { date: NaiveDate::default(), label: None, kind: DayOffKind::Midterm };
```

### App view model (`crates/app/src/viewmodel.rs`)

```rust
/// R71, R74: "Holiday" | "Mid exam" | "Final exam".
pub fn day_off_kind_name(kind: DayOffKind) -> &'static str;

/// R74: Slint fill key: "holiday" | "midterm" | "final"; "" for None.
pub fn day_off_key(kind: Option<DayOffKind>) -> &'static str;

/// R74: legend entries for the day-off fills, in order Holiday, Mid exam, Final exam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LegendItem { pub key: String, pub text: String }
pub fn day_off_legend() -> Vec<LegendItem>;

/// R73, D13: sets `kind` on every day; for Midterm / Final a `None` label becomes
/// `Some(day_off_kind_name(kind))`. Holiday days keep `None`.
pub fn as_day_off(days: Vec<Holiday>, kind: DayOffKind) -> Vec<Holiday>;

/// R73: rows of one card: the days of `kind` only, sorted by date (`holiday_row` each).
pub fn day_off_rows(days: &[Holiday], kind: DayOffKind) -> Vec<HolidayRow>;

/// R89a, D18: "Holidays saved." | "Mid exams saved." | "Final exams saved."
pub fn day_off_saved_text(kind: DayOffKind) -> String;     // Holiday -> holidays_saved_text()
/// R89a, D18: "Holiday removed." | "Mid exam removed." | "Final exam removed."
pub fn day_off_removed_text(kind: DayOffKind) -> String;   // Holiday -> holiday_removed_text()

/// R74: day panel line: "" for None; "Holiday: {text}" (via day_panel_holiday_text),
/// "Mid exam: {text}", "Final exam: {text}"; text = label, else the kind name.
pub fn day_panel_day_off_text(day: Option<&Holiday>) -> String;

/// R74: the day off on `date`, if any.
pub fn day_off_on(days: &[Holiday], date: NaiveDate) -> Option<&Holiday>;
```

Changed behaviour, signatures unchanged:

- `expand_holiday_range` builds `Holiday { .., kind: DayOffKind::Holiday }`.
- `merge_holidays(existing, added)`: one entry per `(date)` per kind; a date in both lists with
  the same kind: the added one wins; with different kinds: both kept, existing first (D11, D12).
  Output sorted by date (stable).
- `holiday_text(days, date)`: label, else `day_off_kind_name(kind)` ("Holiday" for holidays, as
  today), `""` when no day off.
- `CalendarDay` gains `pub day_off: Option<DayOffKind>` (set by `month_grid` from
  `day_off_on`); `holiday` and `label` unchanged in meaning (D22).

`HolidayRow`, `holiday_row`, `remove_holiday`, `parse_holiday_form`, `day_panel_holiday_text`,
`holidays_saved_text`, `holiday_removed_text` unchanged.

### Slint (`crates/app/ui`)

`types.slint`: `CalendarDay` gains `day-off: string` (fill key, D20); new
`export struct LegendItem { key: string, text: string }`.

`components.slint` `SectionCard`: `in property <bool> has-swatch: false;` and
`in property <brush> swatch: transparent;`. The title becomes a `HorizontalLayout`
(`spacing: Theme.space-sm`) with, when `has-swatch`, a `Theme.swatch-size` square
(`Theme.radius-none`, `background: root.swatch`, vertically centred) before the unchanged title
`Text`. Without a swatch the card looks exactly as before.

`settings.slint`:

- New `component DayOffCard` (bindings only): `in` title, has-swatch, swatch, rows
  (`[HolidayRow]`), empty-text, add-text, start-error, end-error, form-error; `in-out` start, end,
  label; `callback add()`, `callback remove(string)`. Layout = today's Holidays card (rows with
  date, label, Remove; three `Field`s; `FormError`; primary button).
- `SettingsState` additions: `midterms`, `finals` (`[HolidayRow]`); `mid-start`, `mid-end`,
  `mid-label`, `fin-start`, `fin-end`, `fin-label` (`in-out string`); `mid-start-error`,
  `mid-end-error`, `mid-form-error`, `fin-start-error`, `fin-end-error`, `fin-form-error`;
  `callback add-midterm()`, `callback add-final()`. Existing `holidays`, `hol-*`, `add-holiday`,
  `remove-holiday(string)` keep their names (all three cards call `remove-holiday`, D17).
- Cards after the Past terms card: Holidays (no swatch, today's texts), Mid exams (swatch
  `Theme.calendar-midterm`, add-text "Add mid exam"), Final exams (swatch `Theme.calendar-final`,
  "Add final exam"); empty texts per D19.

`dashboard.slint`:

- `DayCell` background: today first (unchanged), then `day-off == "holiday"` →
  `Theme.calendar-holiday`, `"midterm"` → `Theme.calendar-midterm`, `"final"` →
  `Theme.calendar-final`, then weekend/hover, else transparent. Update the comment above it.
- `DashboardState`: `in property <[LegendItem]> day-off-legend;`. Legend: first line repeats
  `day-off-legend` (swatch filled by the same key → token mapping, `LegendText` with `text`);
  second line keeps the static Weekend and Assessment due entries (D21). The key → token mapping
  is written once (a `pure function` in the file) if Slint allows it there, else repeated.
- Day panel keeps `panel-holiday` (now the day-off line).

`bridge.rs`: `calendar_day` sets `day_off: vm::day_off_key(c.day_off).into()`; new
`pub fn legend_item(i: &vm::LegendItem) -> LegendItem`.

### Page wiring (`crates/app/src`)

`pages/settings.rs`:

- `refresh`: from the active term, `holidays` = `vm::day_off_rows(&t.holidays, Holiday)`,
  `midterms` = `…Midterm`, `finals` = `…Final` (empty without a term).
- `add_holiday` becomes `add_day_off(app, kind)` with the three callbacks wired to it. Per kind it
  reads and writes that card's fields and errors (small `match kind` accessors). Steps as today:
  `parse_holiday_form(start, end, label)` → `vm::as_day_off(added, kind)` →
  `vm::merge_holidays(&term.holidays, &added)` → `set_holidays`. Err: that card's `*-form-error`
  = `error_text(e)` (R71 text, fields kept). Ok: clear that card's fields and errors,
  `changed_then_toast(app, true, &vm::day_off_saved_text(kind))` (R6 sync, toast).
- `remove_holiday(app, key)`: finds the day off by key (any kind), `vm::remove_holiday`,
  `set_holidays`; Ok: `changed_then_toast(app, true, &vm::day_off_removed_text(removed.kind))`.

`pages/dashboard.rs` `fill_calendar`: `panel-holiday` =
`vm::day_panel_day_off_text(vm::day_off_on(&holidays, selected))`; `day-off-legend` =
`vm::day_off_legend()` via `bridge::legend_item`. `month_grid` call unchanged.

## Project Structure

```text
specs/015-m15-exam-periods/
├── plan.md        # this file (analysis report at the end)
├── tasks.md
└── decisions.md

crates/core/src/
├── migrations/003_day_off_kind.sql   # new (spec 004, renumbered)
├── db.rs          # MIGRATIONS, set_holidays (R71), load_term, DayOffKind as_db/from_db/noun
├── model.rs       # DayOffKind, Holiday.kind
├── schedule.rs    # doc only (R72)
└── backup.rs      # HolidayRow.kind, DAY_OFF_KIND_VERSION (R75)
crates/core/tests/
├── m15_exam_periods.rs   # new: T71, T72, T73 + R71/R72/R75 support
├── api_surface.rs        # + M15 lines
└── common/mod.rs, terms_courses.rs, m3_backup_export.rs, m3_backup_restore.rs,
    migrations.rs, m9_migration.rs, m9_backup.rs   # kind in literals, version numbers
crates/app/src/
├── viewmodel.rs, viewmodel/tests.rs, bridge.rs
└── pages/{settings,dashboard}.rs
crates/app/ui/
├── types.slint, components.slint, settings.slint, dashboard.slint
```

No research.md, data-model.md, contracts/, or quickstart.md: the frozen interfaces above are the
contract and decisions.md replaces research (router instruction).

## Test plan (written first)

Core, new `crates/core/tests/m15_exam_periods.rs` (`mod common;`, in-memory SQLite unless noted).
Fixture: `synced_fixture()` (term 2026-01-05..2026-01-25, CSE101 with a Monday slot).

- `t71_exam_days_skip_classes`: midterm 2026-01-12, final 2026-01-19 (set before or after a sync
  with today 2026-01-05); no session on 01-12 or 01-19; the 01-05 Monday session (and every other
  slot day) kept; `active_term().holidays` kinds are Midterm and Final.
- `t71_holiday_midterm_final_skip_alike`: one day of each kind on three slot days; all three have
  no session; removing the final day (set_holidays without it) and syncing recreates that session.
- `t72_date_with_other_kind_is_rejected`: holiday on 01-12 stored; `set_holidays(term, [holiday
  01-12, midterm 01-12])` → `Error::Validation("January 12, 2026 is already a holiday")`;
  `active_term().holidays` and the session list unchanged.
- `t72_date_listed_twice_names_first_kind`: no stored row; `[final 01-19, midterm 01-19]` →
  `Validation("January 19, 2026 is already a final exam day")`; nothing stored.
- `t72_kinds_stored_and_loaded`: set three kinds in one call (unsorted input); loaded sorted by
  date with labels and kinds kept.
- `t72_existing_rows_stay_holiday_after_migration` (R72; temp file DB as `m9_migration.rs`):
  build a version 2 DB with 001 + 002 + grading seed SQL (`include_str!`), insert a term and a
  holiday row, open with `Planner::open`; `user_version == SCHEMA_VERSION`; the row's kind is
  `DayOffKind::Holiday`.
- `t73_backup_keeps_kinds`: three kinds → `export_json` has `"kind": "holiday" | "midterm" |
  "final"` per row → `restore_json` into a fresh planner → same kinds; re-export equal except
  `exported_at`.
- `t73_older_backup_restores_every_day_off_as_holiday`: export with three kinds, set
  `schema_version` to 2 (D24); once with the `kind` keys removed, once with them kept → every
  restored row is `Holiday`.
- `t73_current_backup_kind_missing_or_invalid_is_rejected` (R75, R22): current version, one row
  without `kind` → `Validation` containing "missing field `kind`"; one row with `"exam"` →
  `Validation` containing "holiday kind"; DB unchanged in both.
- CSV unchanged: the existing `m3_csv.rs` tests stay green (no new test).

Existing core tests updated by the tester (D26): every `Holiday { .. }` literal gets
`kind: DayOffKind::Holiday` (`common/mod.rs` `holidays` and `holiday_range`, `terms_courses.rs`,
`m3_backup_restore.rs`, `m3_backup_export.rs`); `m3_backup_export.rs`
`r21_export_rows_keep_ids_and_value_types` expects `"kind": "holiday"` in the holiday row;
`migrations.rs` `assert_eq!(SCHEMA_VERSION, 3)`; `m9_migration.rs` expects `user_version ==
SCHEMA_VERSION` and `backups/pre-migrate-v1-to-v{SCHEMA_VERSION}.sqlite`; `m9_backup.rs`
`doc["schema_version"] == SCHEMA_VERSION`; plus any other assertion the full run shows that only
encodes the old schema version or holiday shape.

App, `crates/app/src/viewmodel/tests.rs` (helper `holiday()` gains `kind: DayOffKind::Holiday`;
new helper `day_off(date, label, kind)`):

- `t74_month_grid_marks_each_day_off_kind`: October 2026, holiday 10-06, midterm 10-14, final
  10-22 → those cells have `day_off` `Some(Holiday)`, `Some(Midterm)`, `Some(Final)`, and
  `day_off_key` gives `"holiday"`, `"midterm"`, `"final"`; every other cell `None` / `""`; a
  weekend midterm keeps `is_weekend` and `day_off`.
- `t74_legend_lists_holiday_mid_exam_final_exam`: `day_off_legend()` texts `["Holiday", "Mid
  exam", "Final exam"]`, keys `["holiday", "midterm", "final"]`.
- `r74_day_panel_day_off_text_per_kind`: None → `""`; holiday "Break" → `"Holiday: Break"`;
  midterm "Week 7" → `"Mid exam: Week 7"`; final without label → `"Final exam: Final exam"`.
- `r74_holiday_text_unlabeled_uses_kind_name`: unlabeled midterm → `"Mid exam"`; unlabeled
  holiday → `"Holiday"` (existing behaviour).
- `r73_as_day_off_sets_kind_and_default_labels`: blank label → `"Mid exam"` / `"Final exam"`;
  given label kept; Holiday keeps `None`.
- `r73_day_off_rows_filter_by_kind`: mixed list → each card gets only its kind, sorted.
- `r71_merge_keeps_other_kind_on_same_date`: existing holiday 10-14 + added midterm 10-14 → both
  kept (holiday first); same kind same date → added label wins (existing test stays).
- `r89a_day_off_toast_texts`: the six texts of D18.
- Existing `day_panel_holiday_text_empty_and_labeled`, `month_grid_holiday_text_from_label`,
  `merge_holidays_*`, `expand_holiday_range_*`, `parse_holiday_form_*` stay green unchanged
  apart from the `holiday()` helper and literal `kind` fields.

## Complexity Tracking

None. No constitution violation.

## Analysis report (speckit-analyze, read-only)

Artifacts compared: `docs/spec.md` §13.10 (+ R4-R6, R21-R23, R25, R88-R89a), this plan,
tasks.md, decisions.md, constitution, DESIGN.md ("Calendar days off", "Mini calendar", "Day panel
lines", "Exam period cards").

| ID | Category | Severity | Location | Summary | Resolution |
|---|---|---|---|---|---|
| A1 | Inconsistency | LOW | §13.10 "Migration 004" vs repo | Only 001, 002 exist; M12/M19 also take 003 on parallel branches | 003 now; renumber after final rebase, keep `SCHEMA_VERSION`, `DAY_OFF_KIND_VERSION`, migration tests consistent (D1, D24). T009 checks. |
| A2 | Underspecification | MEDIUM | R71 vs §13.10 API | "adding a date that already has another kind" but `set_holidays` replaces everything, so core cannot see an "add" | Core rejects any date listed twice, naming the stored (else first) kind; the app sends the other-kind row plus the new one (D9, D11). |
| A3 | Underspecification | LOW | R71 | Message text not given | D10: "January 12, 2026 is already a holiday". |
| A4 | Ambiguity | LOW | R73 "list of ranges, and remove per day" | Grouped ranges vs one row per day | One row per day with Remove, as the Holidays card (D14). |
| A5 | Underspecification | LOW | R73 "default labels" | Pre-filled field vs stored default | Blank label stores the default (D13). |
| A6 | Underspecification | LOW | R75 | Shape rules for `kind` in a current backup | Required and validated at current version; ignored below (D23). |
| A7 | Coverage | LOW | T74 (app) "legend lists" | Legend is Slint text; no headless UI tests | Legend names come from `vm::day_off_legend()` and are tested (D21). |
| A8 | Inconsistency | LOW | dashboard.slint legend "Holiday / exam" | Old combined entry contradicts R74 | Replaced by the three entries (T008). |
| A9 | Constitution | LOW | `.slint` key → token ternary | Binding vs logic | Same pattern as today's holiday fill; strings come from Rust. No violation. |
| A10 | Coverage | — | R72 sync | No sync code change needed (all kinds live in `term.holidays`) | T71 tests guard it. |
| A11 | Coverage | LOW | Existing tests | `Holiday` field and schema version change break old tests | Tester edits listed in T001/T002 (D26). |
| A12 | Inconsistency | LOW | DESIGN "swatch before each card title" (exam cards) | Holidays card has none | Kept as DESIGN says (D16). |
| A13 | Inconsistency | MEDIUM | plan.md task references vs tasks.md | The first plan draft numbered ten tasks (gate T010); tasks.md has nine (gate T009) | Remediated: plan.md references now match tasks.md (D27). |
| A14 | Inconsistency | MEDIUM | tasks.md T006 | `LegendItem` (Slint struct) and `bridge::legend_item` in T006 would be unused until T008; Slint only generates Rust types for structs reachable from the window, so T006 might not compile | Remediated: both moved into T008 with `day-off-legend` (D27). |
| A15 | Coverage | LOW | T002 imports | App tests need `DayOffKind` in scope | Tester imports `planner_core::DayOffKind` in `viewmodel/tests.rs` (or through `super::*` once T005 imports it). |

Coverage summary:

| Requirement | Has task? | Task IDs | Notes |
|---|---|---|---|
| Migration (kind column) | yes | T001, T003, T009 | t72 migration test |
| R71 kinds, one per date, Validation | yes | T001, T003, T005, T007 | T72 |
| R72 every kind skips classes; old rows holiday | yes | T001, T003, T004 | T71 |
| R73 Mid exams / Final exams cards | yes | T002, T005, T006, T007 | per-day rows (D14) |
| R74 fills, legend, day panel | yes | T002, T005, T006, T008 | T74 |
| R75 backup kind, old backups holiday, CSV unchanged | yes | T001, T004 | T73 |
| R88/R89a toasts for exam Add/Remove | yes | T002, T005, T007 | D18 |
| API (`DayOffKind`, `Holiday.kind`) | yes | T001, T003 | `api_surface.rs` |

Constitution alignment issues: none. Unmapped tasks: none (T009 = gate).

Metrics: 5 rules + 4 tests; 9 tasks; coverage 100%; ambiguities 5 (A2-A6, resolved in
decisions.md); duplications 0; critical 0; high 0; remediated 2 (A13, A14).
Open questions for the human: none blocking. A2 (D9) is the one interpretation that shapes core
behaviour; it adds no API and matches T72.
Next action: implementation (tester T001-T002, then builder T003-T009).
