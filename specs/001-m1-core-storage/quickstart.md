# Quickstart: validate M1

## Prerequisites

- Rust 1.90+ (`rustup show`), repository root as working directory.
- No user database is needed or touched: tests use `Planner::open_in_memory()` or a temp file.

## Run the milestone tests

```sh
cargo test -p planner-core --test migrations   # t20_*, t26_*  (R25)
cargo test -p planner-core --test validation   # t24_*         (R2)
cargo test -p planner-core --test terms_courses   # R1, R3, holidays, courses
cargo test -p planner-core --test slots_settings  # slots, settings
cargo test -p planner-core --test assessments     # assessment storage (FR-013a)
cargo test -p planner-core --test api_surface     # every §5 signature
cargo test -p planner-core --lib               # R24 resolve_db_path unit tests
```

Expected: all pass. T20 shows `user_version` = 1 after a fresh open and unchanged after reopen.
T26 shows open failing with `SchemaTooNew` on a file whose `user_version` is 2, and the file's
version still 2 afterwards.

## Check the API surface

```sh
cargo doc -p planner-core --no-deps
```

Every method in [contracts/core-api.md](contracts/core-api.md) appears on `Planner`. Stub methods
say their milestone and rule IDs in the `todo!()` message.

## Gate (done = pass)

```sh
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

## Done when

- Gate passes; T20, T24, T26 pass (`docs/spec.md` §7 M1 row).
