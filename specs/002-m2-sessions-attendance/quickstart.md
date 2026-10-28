# Quickstart: validate M2

## Prerequisites

- Rust 1.90+ (`rustup show`), repository root as working directory.
- No user database is needed or touched: all tests use in-memory SQLite with the §8 fixture
  (`crates/core/tests/common/mod.rs`).

## Run the milestone tests

```sh
cargo test -p planner-core --test m2_sync          # t04–t07, t14   (R4, R5)
cargo test -p planner-core --test m2_slot_changes  # t10, t11, t27  (R7, R5)
cargo test -p planner-core --test m2_exceptions    # t08, t09, t19, t22 (R8–R10)
cargo test -p planner-core --test m2_attendance    # t01–t03, t12, t21, t23 (R11–R15)
cargo test -p planner-core --test m2_dashboard     # t13, t16–t18, t29 (R16–R20)
cargo test -p planner-core --test m2_archive       # t15, t28 (R3)
cargo test -p planner-core --test m2_exam_periods  # User Story 7 (FR-006a)
cargo test -p planner-core --lib                   # display status, pct rounding unit tests
```

Expected: all pass. T04 shows 6 sessions; T05 a second sync reporting `{created: 0, deleted: 0}`
with the same IDs; T09 the original's display status `Rescheduled → 2026-01-13`.

## Check M1 still passes and nothing is stubbed

```sh
cargo test -p planner-core --test migrations --test validation --test api_surface
grep -n 'todo!("M2' crates/core/src/*.rs   # expect no output
```

## Gate (done = pass)

```sh
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

## Done when

- Gate passes; T01–T19, T21–T23, T27–T29 pass (`docs/spec.md` §7 M2 row), plus the User Story 7
  exam-period scenarios.
