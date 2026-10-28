# Quickstart: validate M5 Reminders

## Prerequisites

- Rust 1.90 toolchain; repository root as the working directory.
- For the manual check: a desktop session with a notification service (macOS Notification Center,
  Windows Action Center, or a Linux notification daemon).

## Automated checks

```sh
cargo test -p planner-core --test m5_reminders      # reminder dedupe and eligibility tests
cargo tree -p app -i libdbus-sys                     # expect: no match (research R-1)
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
```

Expected: every `m5_*` test passes, every earlier test stays green, and the gate passes (SC-001,
SC-002, SC-004). Test-to-scenario mapping: [contracts/core-api-m5.md](contracts/core-api-m5.md).

## Manual notification check (FR-010, §7 "one manual notification observed")

Use a throwaway database so the real one is untouched:

```sh
ACADEMIC_PLANNER_DB=/tmp/ap-m5-check.db cargo run -p app
```

1. Settings: create an active term that contains today; make sure "notifications" is on.
2. Add a course and a slot for today's weekday whose start is about 16–17 minutes from now
   (default `class_reminder_minutes` = 15).
3. Leave the app open. Within about 60 seconds after `start - 15 minutes`, one desktop notification
   appears: summary "Academic Planner", body like `CSE101 Intro to Programming at 14:30 in A-101`.
4. Keep the app open two more minutes: no second notification for the same class.
5. Optional: turn notifications off, add an assessment due in ~10 minutes with reminder minutes 5,
   wait past its fire time plus a minute while still before the due time, turn notifications on;
   one notification appears at the next tick.

Record the result (OS, time observed) in `.claude/state/checkpoint.md`. If the run is unattended,
leave the task marked `[human]` pending.

On macOS the first notification may prompt for permission or appear under the terminal's name when
run with `cargo run`; that is expected for an unbundled binary.
