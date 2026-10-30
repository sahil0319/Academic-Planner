# Checkpoint

- Milestones M7–M10 (docs/spec.md §13) are all built; version bumped to 0.2.0 for release. Each waits only on human steps.
- M11 (§13.6, light flow, 2026-10-30): delete courses (multi-select, hard delete R51) and the GitHub button on About. Gate green at 573263e (544 tests). Human checked it in the app. Released as v0.2.1 (tag pushed 2026-10-30); the draft release must be published on GitHub, then an installed 0.2.0 can update in-app (M7 T035).
- Last green gate: 1183eb5 (fmt ok, clippy 0 warnings, 538 tests pass, theme grep clean, no todo!/dead_code allows). The v1→v2 upgrade was also checked on a copy of the real DB (9 courses, 90 sessions kept, backup written).
- Human tasks:
  - M7 T035: install 0.2.0 by hand, then release 0.2.1 and update in-app (specs/007-m7-updater/quickstart.md §5).
  - M8 T025: visual pass on 12-hour times (specs/008-m8-12-hour-times/quickstart.md).
  - M9 T028: look at Rules page, marks card, needed-in-final, GPA header (specs/009-m9-grading-gpa/quickstart.md).
  - M10 T029: Import panel, including pasting after Copy template / Copy AI prompt (clipboard untested; fallback arboard, research D17).
  - Publish the draft v0.2.0 release on GitHub (it has latest.json only once published, so in-app updates see it after that).
- Signing: minisign key ID 4B63356BFCA1B763, private key in ~/.academic-planner-signing, GitHub secrets CARGO_PACKAGER_SIGN_PRIVATE_KEY(+_PASSWORD). Back both up outside the repo.
- Router decisions (2026-10-29): M8 notifications and error texts are 12-hour too; M9 pending quiz counts as missing and GPA is final only when none is missing; core rejects scores for components outside the course's rule set; M10 same-code duplicates in the DB are an error, R45 ends future-dated slots too. Worked example in docs/import/ai-prompt.md publishes the human's course list.
- Models: builder Sonnet (.claude/agents/builder.md); router = main session /model.
- Manual checks use ACADEMIC_PLANNER_DB="$PWD/target/scratch/demo.sqlite", never the real DB.
- Notes: rustc 1.90; Slint 1.16; rusqlite 0.41; cargo-packager 0.11.8; cargo-packager-updater 0.2.3; cargo-about 0.9.3 (target/tools).
