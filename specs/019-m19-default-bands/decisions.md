# M19 decisions (overnight, unattended)

- The worktree branch was behind main (M13 to M18 missing); rebased onto main first so spec section 13.14 and current code are present.
- The migration file is named by purpose. It was 003 until M12 (003_week_parity) landed on main; renumbered to `004_default_bands.sql` during the rebase (commit messages before the rebase still say 003).
- Test files use `SCHEMA_VERSION` instead of literal 2, so a later migration number does not break them.
- T90 is a pure view-model function `bands_with_defaults(current)`. The Rules page fills default bands only when an attendance component newly appears (add component, or toggle to attendance) and there are no band rows; this avoids refilling after the user deletes all band rows and edits something else.
- A version-1 backup restore runs the grading seed and then the same migration SQL, so it ends with the new table.
- The migration SQL is idempotent and only touches components whose bands are exactly the six old placeholder pairs.
- Spec section 8 T39 numbers (111.0/96.0 needed, 129 secured, 86%) are stale under the new bands (92% attendance now earns 24.0 of 30). Tests use the new numbers; the spec table was not edited. The human may want to refresh it.
- Disk filled up during the build (several worktrees each hold a multi-GB target dir); this worktree's target was cleaned and rebuilt with debug info off.
