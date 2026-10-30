# Tasks: M12 Bi-weekly slots

Source of truth: `docs/spec.md` section 13.7 (R54-R60, T55-T61). Light flow like M11: no plan, checklist, or analyze.
Tests (tester) come before implementation (builder). Decisions are in `decisions.md`.

## Phase 1: types, stubs, migration (builder)
- [x] T001 Migration `003_week_parity.sql`, SCHEMA_VERSION 3, `WeekParity`, `week_parity` on `SlotInput`/`Slot`, `SessionView.biweekly`, `term_week_parity` and `shift_biweekly` stubs, exports
- [x] T002 Fix every existing caller and test fixture for the new fields (weekly = `None`); gate passes with stubs

## Phase 2: tests first (tester)
- [x] T003 Core tests T55-T60 plus R7 slot change, sync parity, backup round trip (`crates/core/tests/m12_biweekly.rs`)
- [ ] T004 App view-model tests T61 and bi-weekly labels (`crates/app/src/viewmodel/tests.rs`)
- [x] T005 (core part; app T004 pending) Confirm new tests fail for the right reason

## Phase 3: core (builder)
- [x] T006 R54 `term_week_parity` (T55)
- [x] T007 R55 sync and slot persistence with parity (T56)
- [x] T008 R57 `shift_biweekly` (T57, T58)
- [x] T009 R60 backup and migration (T60)
- [ ] T010 R59 import `biweekly` key, R45 comparison, template and AI prompt (T59)

## Phase 4: app (builder)
- [ ] T011 Slot form, parity conversion in `viewmodel.rs` (R56, T61)
- [ ] T012 `SessionView.biweekly`, "Every other week" label, "Move to next week" button, controller wiring (R58)

## Phase 5: gate
- [ ] T013 Gate, theme grep, rebase onto main, gate again
