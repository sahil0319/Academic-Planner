# Quickstart: Validate M10

## Prerequisites

- M9 core merged (migration 002, `credits`, `rule_set_id`, seeded rule sets).
- `docs/import/template.json` and `docs/import/ai-prompt.md` exist (the app embeds them).

## Automated

```sh
cargo test -p planner-core --test m10_import          # T44–T50, robustness, R50 template
cargo test -p planner-core --test api_surface         # new signatures bound
cargo test -p app viewmodel::import                   # panel formatting, can_apply
cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace
! grep -rnE '#[0-9a-fA-F]{6}|[0-9]+px' crates/app/ui --include=*.slint | grep -v theme.slint   # theme grep, as M7
```

Expected: all pass; the grep prints nothing.

## Manual (human, ~5 minutes, on a copy of the DB via `ACADEMIC_PLANNER_DB`)

1. Fresh DB → Courses → Import… → Copy template, paste it, set the effective date to 2026-01-05,
   Preview: "Creates term … adds 1 course", no errors. Apply: the course and its Mon/Fri slots
   appear on Timetable from 5 Jan.
2. Preview the same text again: "No changes. Everything already matches."; Apply is disabled.
3. Change the Monday end to `12:00`, effective date a week later, Preview: one slot change
   ("Ends after …", "From …"). Apply: earlier Monday sessions keep 11:45, later ones end 12:00.
4. Paste a file with a bad day, an end before start, a duplicate code, and rule set "Foo":
   four errors with paths; Apply disabled.
5. Add `"teacher": "X"` to a course: a warning, not an error.
6. Copy AI prompt: the clipboard holds the prompt, including the merged-cell rule and the
   7th-semester example.
