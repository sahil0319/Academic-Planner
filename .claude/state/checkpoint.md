# Checkpoint

- Milestone: M2 done (specs/002-m2-sessions-attendance, T001–T050). Next milestone: M3
- Last green gate: 7c9f2c3 (fmt ok, clippy -D warnings ok, cargo test --workspace: 95 tests pass incl. T01–T19, T21–T23, T27–T29, m2_exam_periods, M1 T20/T24/T26).
- Next: /speckit-specify for Milestone M3 from docs/spec.md (backup/restore R21–R22, CSV export R23). Do not add requirements.
- M2 notes:
  - sync_in(conn, today) in schedule.rs is shared by sync_sessions, change_slot (today = effective_from), end_slot (today = last_date + 1).
  - query_views + ACTIVE_SCOPE in schedule.rs are reused by dashboard.rs.
  - pct uses f64::round (ties away from zero, router clarification R-D9).
  - Remaining stubs: backup.rs (M3 R21/R22, M5 reminders), export.rs (M3 R23).
- Notes: rustc 1.90; Slint 1.16; rusqlite 0.41; packages planner-core / app.
- Decision (human, 2026-10-29): R5 sync never deletes a session a makeup references (FR-003, research R-D13; test r5_sync_keeps_session_referenced_by_makeup).
- Open questions: none
