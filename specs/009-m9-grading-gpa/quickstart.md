# Quickstart: validating M9

## Prerequisites

- M8 merged (the app track edits the same view-model and form files).
- No real user database is touched: tests use in-memory SQLite; manual runs use
  `ACADEMIC_PLANNER_DB` pointing at a scratch file.

## Core track

```sh
cargo test -p planner-core --test m9_rules     # T36, T43, R34/R39 seeds and validation
cargo test -p planner-core --test m9_marks     # T37–T41
cargo test -p planner-core --test m9_backup    # T42, migration 002 upgrade of a v1 file
cargo test -p planner-core                     # all earlier tests, with pins moved to version 2
```

Expected: all pass. T39 shows A+ 111.0 and A 96.0 of 150; T41 shows GPA 3.67 projected.

## App track

```sh
cargo test -p app viewmodel                    # formatting and form-parsing tests
```

## Gate (§7) and theme check

```sh
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

Plus the theme grep check from CI (`.github/workflows/ci.yml`): no raw hex color or `px` size in
any `.slint` file except `crates/app/ui/theme.slint`.

## Manual look (human, optional)

1. `ACADEMIC_PLANNER_DB=/tmp/m9.sqlite cargo run -p app`.
2. Rules page: Theory and Lab exist with the §13.3 values; the grade scale has ten rows.
3. Create a 3-credit course with rule set Theory; add quizzes counting toward Quizzes with scores
   12, 11, 13, 14 and a mid with 60 of 75; mark attendance to 92%.
4. Course detail: Quizzes 39.0 of 45; needed in final A+ 111.0, A 96.0.
5. Courses page header shows the term GPA with "(projected)".
6. Open a v1 database copy: a `backups/pre-migrate-v1-to-v2.sqlite` file appears and the data is
   intact.
