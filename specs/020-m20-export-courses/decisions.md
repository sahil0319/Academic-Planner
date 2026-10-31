# M20 decisions (overnight, unattended)

- The branch was rebased onto `main` first, because the section 13.15 spec commit lives only there.
- `export_courses_json` lives in `crates/core/src/import/export.rs` next to the importer it must round trip with; the document is built from `serde` structs, so keys keep the order of the section 13.4 example.
- With `Some(ids)` the export also takes archived courses (the user ticked them with "Show archived"); the list is sorted by code and a repeated id counts once.
- `biweekly` is relative to the course's own term start, not only the active term, so a course is always exported consistently.
- A slot that starts after `today` but has not ended is exported (R97 says "not ended"); it is then not valid on the effective date in the exporting planner, which is the one round-trip edge case.
- `instructor`, `room`, and `min_attendance_pct` are written as `null` when empty, `credits` and `rule_set` are omitted when the course is not graded (R97).
- The app returns `ExportNote::Toast` or `ExportNote::Inline` from `viewmodel.rs`, so the "No courses to export." case is unit-tested without a window.
- Copy JSON uses the Import panel's mechanism (a hidden read-only `TextInput` with select-all, copy, clear-selection). A synchronous Slint callback `prepare-json-copy() -> bool` fills `export-text` first; `json-copied` then toasts. The T91 list holds `copied_json` and `save_json`.
- R100: `docs/import/README.md` is new (the `biweekly` meaning for exports), `ai-prompt.md` gets a sentence that an exported file is pasted straight into Import, and the Import panel line gains "An export from another Academic Planner works too."
- `template.json` is unchanged (it already documents `biweekly`).
