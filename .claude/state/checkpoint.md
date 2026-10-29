# Checkpoint

- Milestone: M7 (About page, signed updater, pre-migration backup; docs/spec.md §13.1). Next: M8 12-hour times (§13.2), then M9 grading/GPA, M10 JSON import.
- Last green gate: 1420e8f (fmt ok, clippy 0 warnings, 347 tests pass, theme grep clean). Nothing pushed since 19c2153's parent (origin/main = a7abf1c).
- M7 tasks: T001–T033 done. Open: T034 manual dev-build checks (specs/007-m7-updater/quickstart.md §4, human), T035 human release run (quickstart §5: v0.2.0 then v0.2.1, publish, update in-app).
- Signing: human made the minisign key (ID 4B63356BFCA1B763, private key in ~/.academic-planner-signing) and GitHub secrets CARGO_PACKAGER_SIGN_PRIVATE_KEY(+_PASSWORD). The private-key secret had a stray trailing `%`; human told to re-paste it via `pbcopy < ~/.academic-planner-signing/updater.key`.
- Models: builder now Sonnet (.claude/agents/builder.md); router = main session /model (user switches).
- Manual checks must use ACADEMIC_PLANNER_DB pointing at a scratch DB, never the real one.
- Demo data: ACADEMIC_PLANNER_DB="$PWD/target/scratch/demo.sqlite" cargo run -p app
- Notes: rustc 1.90; Slint 1.16; rusqlite 0.41; cargo-packager 0.11.8; cargo-packager-updater 0.2.3; cargo-about 0.9.3 (target/tools). M4–M6 history in git log.
