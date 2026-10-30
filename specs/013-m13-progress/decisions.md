# M13 Decisions

One line per decision. The planner wrote D1–D14 (recommended option where a choice existed); the
builder appends new lines (D15, …) during the build. None changes a rule, the schema, or the API.

- D1 (planner): Page index 7 for Progress (Rules already uses 6); no existing index is renumbered; the NavItem sits between Dashboard and Timetable (R61, R85).
- D2 (planner): The page has `refresh` only and no `wire`, because it has no callbacks (read-only, no toasts).
- D3 (planner): Refresh on page open and after data changes by adding `pages::progress::refresh` to `App::refresh_all` (`page_selected` and `App::changed` already call it); not added to the 60-second `tick`, because no figure depends on `now` and a date rollover already calls `refresh_all`.
- D4 (planner): R63 `k` uses `i64::div_euclid` (true floor), so a small negative numerator gives `k < 0` ("out of reach"), not 0.
- D5 (planner): R63 check order: no target → no line; `r = 0` → "No classes left to mark"; then the `k` branches.
- D6 (planner): Count nouns follow the existing `count_text` convention: "class" when the number is 1, "classes" otherwise (R62 "1 of 1 class", R63 "1 remaining class"); every T62 example keeps the spec's plural text.
- D7 (planner): R62 target text is "Target 75%"; without a target it is "No target" (R62 names the target but gives no text).
- D8 (planner): R62 layout: the Attendance figure is the R13 percentage in numeral type ("—" when none); the full R62 line "87.5% · 14 of 16 classes" (or "No classes marked yet") is printed beside the bar (DESIGN.md "the percentage printed beside it").
- D9 (planner): R64 secured marks use one decimal and assessed marks use `format_mark`, like the existing `format_marks` ("Secured 129.0 of 150 assessed marks"); the Marks figure is `format_projected` ("86.7% · A+"), or "—" when not graded or nothing assessed.
- D10 (planner): R64 best-grade line is empty (hidden) when the course is not graded or every needed row is "Not reachable" (the spec gives no text for that case).
- D11 (planner): R65 grade points come from `grade_scale()` by the projected letter; no projected percentage or a letter not in the scale gives no points, so marks do not affect the status.
- D12 (planner): R65 Watch on attendance needs both a percentage and a target and means `pct < target + 5.0` on the rounded R13 percentage.
- D13 (planner): R61 top line reuses `gpa_header_text` (R40, M9) for the GPA and adds the counts text; Not enough data is not counted. The empty state replaces the top line and cards: no term → "No term yet. Create one in Settings → Term." (Dashboard text); no courses → "No courses yet. Add one on the Courses page."
- D14 (planner): `ProgressCardRow` goes in `types.slint` with the other row structs; `ProgressBar` is a local component of `progress.slint`; cards are not clickable (the spec gives no action).
- D15 (router): D6 confirmed, singular "class" when the count is 1 (small departure from literal R62/R63 text; every T62 example unchanged).
- D16 (builder): M13 stubs carry `#[allow(dead_code, unused_variables)]` (variables because the `todo!` bodies leave the parameters unused; clippy runs with `-D warnings`); both allows are removed when T007–T009 fill the bodies and T012 sweeps the rest. Stub field docs are `///` comments on `ProgressCardView` fields.
- D17 (builder): `progress.slint` also holds two local components, `FigureColumn` and `ProgressCard`, besides `ProgressBar` (same reason as D14: only this page uses them); the card property is named `card` because `row` is a built-in Rectangle property in Slint.
- D18 (builder): No new theme token. The Progress card uses existing tokens only: `surface`, `radius-none`, `space-lg`, `row-height`, `numeral-md-*`, `label-sm-*`, `body-sm-*`, `progress-track`, `progress-fill`, `progress-target`, `progress-height`, `progress-target-width`.
- D19 (builder): `pages::progress::refresh` clears `ProgressState.message` after a successful load (as an error-free refresh shows no stale text); the grade scale is read once per refresh and passed to every card. Checked D1–D14 against the code: all still match.
