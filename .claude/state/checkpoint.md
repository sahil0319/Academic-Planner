# Checkpoint

- All agent work for M4, M5, M6 is done; each milestone waits only on human steps. Nothing pushed (origin is ~70 commits behind).
- Last green gate: M6 final (fmt ok, clippy 0 warnings, 258 tests pass, theme grep clean). DMG built locally: target/release/Academic Planner_0.1.0_aarch64.dmg (7.1 MiB, hdiutil VALID).
- Human tasks:
  - M4 T029: §9 smoke (specs/004-m4-desktop-ui/quickstart.md §6; archive a course + delete an assessment instead of "delete a course's data").
  - M5 T012: see one desktop notification (router check already logged one in reminder_log; visual pending).
  - M6 T016: capture docs/screenshots/{dashboard,timetable,courses,assessments,settings}.png.
  - M6 T019–T021: git push origin main; git tag v0.1.0 && git push origin v0.1.0; confirm draft release has .dmg, -setup.exe, .deb, .AppImage, THIRD-PARTY-NOTICES.html; install one.
- Demo data: ACADEMIC_PLANNER_DB="$PWD/target/scratch/demo.sqlite" cargo run -p app
- Router decisions (overnight delegation 2026-10-29):
  - M4: no file-DB test for pre-restore copy (Q1 = B); same-minute suffix -2/-3; tick skips Settings refresh; only P1 critique items fixed (critique 22/40; P2 in design-review.md).
  - M5: due window fire_at <= now < max(T, fire_at + 1 min); missed reminders skipped; notifications off = no check; titles in core; clippy.toml allow-invalid on chrono bans.
  - M6: draft release on tag v*; tag/version check; placeholder solid-terracotta icon; DMG Apple Silicon only; notices generated (git-ignored) and bundled.
- Follow-ups (not required by spec): P2 critique items (timetable single card, nav hover token, custom toggle/checkbox instead of Fluent blue).
- App icon: real icon from the human (Gemini), corners made transparent and fitted to the macOS 824/1024 grid; icon.png is 512 px (cargo-packager icns needs it), master in crates/app/icons/icon-1024.png.
- Notes: rustc 1.90; Slint 1.16 (fluent-light); rusqlite 0.41; rfd 0.17.2; notify-rust 4.18.2; cargo-packager 0.11.8; cargo-about 0.9.3 (target/tools).
