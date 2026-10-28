# Academic Planner Constitution

## Core Principles

### I. The Spec Is the Source of Truth

- `docs/spec.md` defines the rules, schema, and API. Agents MUST NOT change rules, schema, or API
  without asking the human first.
- `DESIGN.md` defines the visuals. The UI MUST use `Theme.*` only; raw hex colors or px sizes
  MUST NOT appear outside `crates/app/ui/theme.slint`.
- Spec Kit features MUST NOT add requirements that are not in `docs/spec.md`.

Rationale: one authoritative document keeps parallel sessions and reviewers consistent.

### II. Test-First, Traceable

- The T-tests for a rule MUST be written before the rule is implemented.
- Test names MUST follow the pattern `t07_holiday_added_after_sync`.
- Doc comments MUST reference rule IDs, e.g. `/// R5: sync never deletes past or non-pending sessions.`
- Tests MUST use in-memory SQLite and MUST NOT touch the real user database.

Rationale: rule and test IDs let the agent and the reviewer trace coverage without reading the UI.

### III. Pure, Clock-Free Core

- The `core` crate MUST NOT depend on Slint, read the system clock, or open file dialogs.
  Callers pass `today`/`now` in; `clippy.toml` enforces the clock ban.
- `.slint` files MUST contain only bindings and callbacks. All formatting shown in the UI lives in
  `core` or `crates/app/src/viewmodel.rs` as pure functions with unit tests.

Rationale: logic that does not depend on the screen or the clock is deterministic and testable.

### IV. Append-Only Migrations

- Schema changes MUST be a new numbered migration file. An applied migration MUST NOT be edited.

Rationale: existing user databases must always upgrade cleanly.

### V. Gate Before Done

- A milestone is done only when the gate passes:
  `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
- Agents MUST NOT claim done without running the gate and pasting its summary.

Rationale: a single mechanical check replaces subjective "looks done" judgments.

## Agent Working Rules

- Use exactly the skill listed in `docs/spec.md` §12.0 for each step. Do not load design skills
  outside M0/M4.
- Update `.claude/state/progress.json` after each task and `.claude/state/checkpoint.md` at the end
  of each session.
- Chat replies use caveman mode. Code, comments, tests, commits, docs, and specs use normal English.
- Read only the files the current task needs. No repo-wide exploration unless asked.

## Development Workflow

- Work one milestone at a time, in order M0 → M6, using one Spec Kit feature per milestone
  (`docs/spec.md` §12).
- Make small commits, one per coherent step, with the message prefixed by the milestone, e.g.
  `M2: sync deletion rule R5`.

## Governance

This constitution mirrors `AGENTS.md` (`docs/spec.md` §11). Where they differ, `docs/spec.md` wins
and this file is amended to match. Amendments require human approval, a version bump (MAJOR for
removed or redefined principles, MINOR for added principles or sections, PATCH for wording), and an
updated Last Amended date. Every milestone review checks the gate and principles I–V.

**Version**: 1.0.0 | **Ratified**: 2026-10-28 | **Last Amended**: 2026-10-28
