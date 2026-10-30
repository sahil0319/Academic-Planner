# M17 Design Review (T012, `impeccable` critique mode)

Method: single-context critique of the 24 committed screenshots (12 screens x light and dark,
1100 x 720) against DESIGN.md v2, `spec.md` and `contracts/components.md`. Authorised by
docs/spec.md R86 (the recorded M17 exception to the "no design skills outside M0/M4" rule). No
code, DESIGN.md or spec file was edited. The bundled `impeccable detect` scanner targets web
markup and does not apply to a Slint app, so there is no deterministic scan; PRODUCT.md does not
exist, so the critique used DESIGN.md as the only visual authority.

Contrast figures below were computed (WCAG relative luminance) from the DESIGN.md hex values and
spot-checked against pixel samples of the screenshots (for example dark selected tab and field are
both `#393939`; the light switch track is `#0055D8`, the dark one `#59BDFF`). Findings marked
"computed, not visible in the shots" are derived from the token values, not seen on screen.

## Verdict

Overall the restyle is faithful and clean. Square corners, 40px rows, 32px controls, 3px rules,
IBM Plex Sans 400/600, flat surfaces, the single-blue rule, the sidebar and the inverse appearance
switch all match DESIGN.md in both themes. The dark theme reads as proper Carbon g100 (layers
`#161616` / `#262626` / `#393939`, readable chips and banner). Nothing is broken or unreadable
(no P0). There are 4 P1 items; three are small token or binding fixes, one (negative zero) is a
pre-existing view-model bug outside the M17 scope.

Design Health Score (Nielsen, 0-4; heuristics 7 and 9 `n/a`, not evidenced in static screenshots):

| # | Heuristic | Score | Key issue |
|---|---|---|---|
| 1 | Visibility of system status | 3 | Selected tab/segment is weight-only in dark (F-05) |
| 2 | Match system / real world | 3 | `-0.0 of 45` (F-04) |
| 3 | User control and freedom | 3 | Inline confirms with Cancel everywhere |
| 4 | Consistency and standards | 3 | Off-palette switch/checkbox, ragged course names (F-06, F-07) |
| 5 | Error prevention | 3 | Danger style only on confirms; one per panel |
| 6 | Recognition rather than recall | 3 | Truncated course names on action rows (F-08) |
| 7 | Flexibility and efficiency | n/a | Keyboard paths not visible in screenshots |
| 8 | Aesthetic and minimalist design | 3 | Dense but ordered; weak sub-heading tier (F-09) |
| 9 | Error recovery | n/a | No error state captured (F-11) |
| 10 | Help and documentation | 2 | One-line empty state only; acceptable for a desktop tool |
| **Total** | | **23/32** | Good |

Design specificity: the result is authored for this product (Carbon-derived, dense, data-led,
status chips with text always present, calendar fills with legend), not generic. The one place it
falls back to a platform default is the Fluent CheckBox/Switch (F-06).

## Findings

### P1 (high)

**F-01 Floating appearance switch covers scrolled content**
- Screens: `11-settings-restore-confirm-light.png`, `11-settings-restore-confirm-dark.png`; also
  sits over the scroll area in `06-*`, `08-*`, `09-*`, `10-*`.
- Severity: P1 (content hidden, DESIGN.md "scrollable content area" violated in practice).
- Evidence: in 11 the content is scrolled; the switch (x 996-1084, y 15-47) draws over the top of
  the "Upcoming window (days)" field, whose right end (to x 1060) is hidden behind it. The
  switch is a floating overlay with an opaque fill, so any control scrolled under it is both
  hidden and unclickable. It will hit every page that scrolls (Settings, Courses detail,
  Assessments, Rules, Dashboard).
- Fix (layout only, `app-window.slint`): give the shell a fixed header band that holds the
  switch and does not scroll, and start the content `ScrollView` below it. Use `Theme.neutral`
  fill, height `Theme.appearance-segment + 2 * Theme.space-sm`, switch at `Theme.space-lg` from
  the right, content top padding reduced by the band height so page titles do not move much.
  Alternative if the human accepts a DESIGN.md "top-right corner" deviation: place the switch at
  the bottom of the sidebar (`Theme.space-lg` insets), which never overlaps content. Either way,
  retake 11 scrolled.

**F-02 Due marker on today cell has no contrast (computed, not visible in the shots)**
- Screens: `01-dashboard-light.png`, `01-dashboard-dark.png` (the sample data has a due date on
  27 Oct, not on today, so the case is not on screen). Known item 3.
- Severity: P1 (WCAG 1.4.11 non-text contrast fail; and it is exactly the day that matters most).
- Evidence: light `error #DA1E28` on `primary #0F62FE` is 1.0:1 (equal luminance, the marker
  vanishes); dark `dark-error #FF8389` on `#0F62FE` is 2.1:1. Needs 3:1.
- Fix (`dashboard.slint`, DayCell binding only): `background: is-today ? Theme.on-primary :
  Theme.error` for the 4px marker (white on primary is 5.0:1). This departs from the literal
  "error-colour square" in DESIGN.md for the today cell only; human should confirm (see OQ-3).

**F-03 Out-of-month day number on a weekend cell fails AA**
- Screens: `01-dashboard-light.png` (the "1" of 1 Nov, Sunday, at about x 525 y 697).
- Severity: P1 by the stated rubric (text contrast fail), low reach (only out-of-month days that
  fall on a weekend, light theme only).
- Evidence: `on-surface-subtle #6F6F6F` on `surface-hover #E8E8E8` is 4.1:1 for 14px text; AA
  needs 4.5:1. The same digit on the plain card is 4.57:1 (passes), dark theme passes (5.3:1).
- Fix (`dashboard.slint` DayCell): use `Theme.on-surface-muted` (6.4:1 on the weekend fill) for
  out-of-month digits on weekend cells, or for all out-of-month digits to keep one rule.

**F-04 Negative zero in the marks table**
- Screens: `06-course-detail-light.png`, `06-course-detail-dark.png`.
- Severity: P1 (clearly wrong text a user will see: `-0.0 of 45`, `-0.0 of 75`, `-0.0 of 150`).
- Evidence: Quizzes, Mid and Final rows print `-0.0`; Attendance prints `0.0`. Not a styling
  defect and almost certainly pre-existing; FR-018 forbids view-model changes in M17.
- Fix: not a theme or layout fix. In `viewmodel.rs` normalise `-0.0` to `0.0` before formatting,
  with a unit test. Recommend the router logs it as a bug for a later milestone rather than
  reopening M17 (human decision, OQ-4); it should not block the visual gate.

### P2 (polish a user may notice)

**F-05 Selected tab / segment differs from the others by weight only in dark**
- Screens: `03-timetable-week-dark.png`, `04-timetable-rules-dark.png`,
  `09-assessments-dark.png` (also the AM/PM toggle in `03-*-dark`).
- Evidence: selected tab fill `dark-surface-selected #393939` equals unselected `dark-field
  #393939` (pixel-sampled, ratio 1.0); only the 600 weight and a barely lighter label separate
  them. Light is clear (`#E0E0E0` vs `#FFFFFF`). DESIGN.md allows weight as the cue, so this is
  compliant but weak. Root cause is a token collision in DESIGN.md (selected = field in dark).
- Fix (layout, `components.slint` AmPmSegment, `timetable.slint` tabs, `assessments.slint`
  FilterTab): add an inset `Theme.rule-width` bar in `Theme.primary` on the bottom edge of the
  selected tab (the same device as the active nav item; "active" is an allowed blue use). Do not
  edit DESIGN.md.

**F-06 CheckBox and Switch are off-palette and not square (known items 2 and 5; OQ-2)**
- Screens: `12-about-dark.png`, `12-about-light.png`, `11-*`, `05-*`, `07-*`, `09-*`.
- Evidence: switch track and checked box use the Fluent accent, `#0055D8` (light) and `#59BDFF`
  (dark), not `primary #0F62FE`; in `12-about-dark` the light-blue track with a black thumb sits
  beside the real-blue "Check for updates" button, reading as a second accent (DESIGN.md "no
  second accent colour"). Checkbox corners are slightly rounded; the switch is a pill.
  Contrast itself is fine (track vs card 5.8:1 light, 7.3:1 dark; checked state is also marked by
  a tick), so this is fidelity, not accessibility.
- Fix (`theme.slint`): if the Slint version allows writing the `Palette` global
  (`accent-background`, `accent-foreground`), bind them to `Theme.primary` / `Theme.on-primary`
  so both widgets use the DESIGN.md blue in both themes (verify the property exists in the pinned
  Slint version before promising it). Squareness needs a small custom component (see OQ-2).

**F-07 Course name column is ragged**
- Screens: `05-courses-light.png`, `05-courses-dark.png`, `07-courses-delete-confirm-*.png`.
- Evidence: the course name starts at x 535, 522, 563, 496 on successive rows, so it looks
  centred in its box rather than left-aligned; the code column and the name column do not form a
  scan line. The meta line below ("Instructor Dr. Rahman Room B-204 Target 75%") has label and
  value run together with no separator.
- Fix (`courses.slint` row): `horizontal-alignment: left` on the name text and a fixed code
  column width (a `Theme.*` size), so names share one left edge; add `Theme.space-md` between
  the label/value pairs of the meta line.

**F-08 Course names truncated on rows with three or four buttons**
- Screens: `01-dashboard-light.png`, `01-dashboard-dark.png` (Awaiting attendance: "HUM 4741 .
  Busine...", "CSE 4739 . Data Min...", "CSE 4712 . Artificial Intel..."; day panel: "CSE 4711 .
  Artifici...", "HUM 4741 . Business C...").
- Evidence: weekday + date + time take about 330px, leaving too little for the course label next
  to three 90px buttons. The truncated part is the part that tells the rows apart. See also F-13
  (padding).
- Fix (`components.slint` ListRow / `dashboard.slint`): give the course label the stretch
  priority and elide the date column last, reduce ListRow side inset from `Theme.space-lg` to
  `Theme.space-sm`, and reduce the gap between date, time and label to `Theme.space-sm`. Text
  must not change (FR-018).

**F-09 Weak sub-heading tier inside panels**
- Screens: `03-timetable-week-*` ("Reschedule"), `06-course-detail-*` ("Marks", "Score needed in
  the final..."), `08-import-*` ("Courses to update"), `10-rules-*` ("Saved rules", "Components,
  in order").
- Evidence: sub-sections are introduced by 12px regular text in the same size and nearly the
  same colour as field labels, so "Reschedule" reads as a label, not a section; in 08 "Courses
  to update" is a 16px weight-400 line below a bold 14px "updates 1 course", an inverted order.
  DESIGN.md has no 16px/400 role.
- Fix: use `Theme.body-emphasis` (14/600) `Theme.on-surface` for these sub-headings and
  `Theme.headline-md` only for card titles; make "Courses to update" body-emphasis and put the
  count line after it.

**F-10 Vertical rhythm: blocks touch**
- Screens: `06-course-detail-*` (Marks table ends at y 478 and "Secured" starts at 487; "Marks"
  label 8px under the stats row; caption directly above the grade table), `08-import-*` (Effective
  date label 6px under the Copy buttons; Preview/Apply 12px above the result lines).
- Evidence: groups are separated by 6-10px where DESIGN.md prescribes 4px-grid steps and 16px
  inside cards; related and unrelated items look equally close.
- Fix (`courses.slint`, `import.slint`): set the parent `VerticalLayout` spacing to
  `Theme.space-md` and insert `Theme.space-lg` before each new group.

**F-11 Evidence gap: states the spec asks for are not in any screenshot**
- Screens: none (absence).
- Evidence: no focus ring (2px inset `focus`), no invalid field with `error` ring and error
  text, no disabled primary button, no update banner, no hover, no dashboard legend (it is below
  the 720px fold). The critique cannot confirm spec User Story 3 scenarios 1-2 or the edge cases.
- Fix: before the human approves, retake a small supplementary set (keyboard focus on a field and
  a button in both themes; an invalid field; a disabled primary; About with the update banner
  shown; the dashboard scrolled to the legend). Same scratch DB; no UI change.

### P3 (nits and known items rated)

**F-12 Dashboard fold (known item 1).** `01-dashboard-*`. The 5th calendar row is not cut off:
its cells span y 682-718 and fit inside 720. What is off-screen is the card's bottom padding and
the legend, so the first view of the main page never shows the legend or the due-marker key. This
is window size, not a defect; cover it in the supplementary shot (F-11). Optional: nothing to
change in the UI.

**F-13 ListRow side padding (known item 4).** `01-*`, `03-*`, `05-*`, `09-*`. The row inset is
16px per side (text starts at x 296 inside a card edge at 264+16). Rows with four buttons (day
panel, wrapped to two lines) and three buttons (Awaiting attendance) do not look cramped; the
real cost is the truncation in F-08. Keep `Theme.space-lg` unless F-08 needs the 8px.

**F-14 Update banner not evidenced (known item 6, OQ-1).** No screenshot shows it (it only
appears when an update exists). Cannot judge. See OQ-1.

**F-15 Import Close button top-right (known item 7).** `08-import-*`. Works: Close sits on the
title's centre line (y 75) at the card's right inner edge, discoverable, and the Import title
remains the heading. Nit: it sits 12px under the appearance switch with a different right edge
(1060 vs 1084), so the two controls stack with a small jog; harmless once F-01 moves the switch.

**F-16 Empty-state text is centred.** `02-dashboard-empty-*`. "No term yet. Create one in Settings -> Term." is centred in a
card whose title is left-aligned; DESIGN.md shows muted body text in the card. Left-align with
`Theme.space-xl` padding for consistency with other empty states.

**F-17 Below-target banner label is smaller than its lines.** `01-dashboard-*`. "Below target"
is 12px above 14px lines; the label should lead. Use `Theme.body-emphasis` for the label.
Contrast is fine (7:1+ light, readable dark).

**F-18 "Below target" chip invisible on the light card.** `06-course-detail-light.png`.
`below-target-bg #FFF1F1` on `surface #F4F4F4` is about 1.0:1, so the pill shape disappears
(text is 7:1 and still legible). Dark is fine. Consider `Theme.status-absent-bg` for the chip.

**F-19 Assessments form grid.** `09-assessments-*`. "Score (optional)" starts at x 707 while
"Kind" above it starts at x 711 (4px off), and the Score field ends at x 920 while other rows end
at 1060. Align the grid columns with `Theme.space-lg` gaps.

**F-20 Database block differs between pages.** `11-*` vs `12-*`. Title "Database file" with
14px path (Settings) vs "Database" with 12px path (About). Pick one.

**F-21 Select-all checkbox not aligned with row checkboxes.** `05-*`, `07-*`. "Select all" box at
x 289, row boxes at x 305 (the row inset). Align both to the same inset.

**F-22 Time fields show `1030` / `1145` without a colon.** `03-timetable-week-*`. Labels and
rows show "10:30 AM". Verify this is the input format; if it is, no change.

## What works well

- Tokens and structure: square everywhere except chips; cards `#F4F4F4` on white, `#262626` on
  `#161616`; no shadows or gradients; 3px rules on the nav bar, below-target banner and update
  banner; one blue for primary, active nav, today, focus and links.
- Sidebar: clear active state (selected fill, 600 weight, 3px blue bar), consistent in both
  themes; the inverse segment of the appearance switch is crisp.
- Danger buttons in the delete and restore confirms (`07-*`, `11-*`): error outline and label,
  one per panel, readable (5.0:1 light, 4.9:1 dark), clearly different from Cancel.
- Status chips keep their text, with good contrast in both themes (about 5.8:1 light "Attended").
- Dark theme: real layering (page, card, field), borders visible, GitHub button inverts as
  specified, calendar holiday fill readable.
- Fields: Carbon bottom-rule style with labels above is consistent across Timetable, Assessments,
  Rules, Import and Settings; the paste box matches.
- Typography: IBM Plex Sans renders at 400/600 only; page titles 24/400, card titles 16/600 form a
  clear two-step hierarchy.
- Light and dark screens are structurally identical (same layout, same states).

## Open questions for the human

**OQ-1 Update banner: `surface` fill with `border-strong` rule.** Recommendation: accept. A
neutral banner keeps blue for actions only and error red for the below-target banner, matching
DESIGN.md's colour rules; `border-strong` against `surface` is 3.0:1, so the 3px rule is visible.
Condition: no screenshot shows it yet, so take one (F-11) before closing the milestone; if it
reads as just another card, switch the fill to `field` (white / `#393939`) rather than adding a
colour.

**OQ-2 Slint CheckBox/Switch are not square.** Recommendation: accept the Fluent shapes for M17
(spec edge case allows them), but fix the colour (F-06): bind `Palette.accent-*` to the primary
tokens if Slint allows it, so the dark switch stops using a second blue. Record the square
variants as a follow-up (a small custom `Rectangle` checkbox with a `border-strong` 1px edge and
`primary` fill when checked) for a later milestone, because it needs new components and tests of
keyboard and accessibility behaviour, which exceeds "restyle only".

**OQ-3 Due marker on today (F-02).** Recommendation: approve the white-on-blue marker for the
today cell only; it is the only way to meet 3:1 without changing the today fill.

**OQ-4 Negative zero (F-04).** Recommendation: log as a bug for the next milestone that touches
`viewmodel.rs`; do not reopen M17.

Questions skipped for the caller: the router relays OQ-1 to OQ-4 to the human.

## Decisions and fixes (T012 builder part)

Router decisions applied by the builder. Layout and token changes only: no Rust, core,
`types.slint`, `*State` property or callback changed; no value hardcoded outside `theme.slint`
(`! grep -rnE '#[0-9a-fA-F]{6}|[0-9]+px' crates/app/ui --include=*.slint | grep -v theme.slint`
prints nothing). Human approval is not recorded here; that is the human's step (T012).

| Finding | Result | What changed |
|---|---|---|
| F-01 switch overlays scrolled content | Fixed | `app-window.slint`: a fixed header band (`space-lg + appearance-segment + space-sm` high, `neutral`) holds the switch 16px (`space-lg`) from the top and right, as DESIGN.md says. The page `ScrollView` starts below the band (content top padding 0, so the page title stays near its old position). The update banner moved from the old `padding-top` hack to sit directly under the band. Screens 11 and 14 to 17 show scrolled content stopping at the band. |
| F-02 due marker on today | Fixed, departs from DESIGN.md | `dashboard.slint` DayCell: marker background is `Theme.on-primary` on the today cell, `Theme.error` elsewhere (error on the primary fill is 1.0:1 light, 2.1:1 dark). DESIGN.md is not edited in M17. Evidence: `17-dashboard-legend-*` (a Lab report due today). Human confirms as OQ-3. |
| F-03 out-of-month digit contrast | No change needed | The code already used `Theme.on-surface-muted` for out-of-month digits (pixel sample of the "1" of 1 Nov in `01-dashboard-light` is `#525252`, 6.4:1 on the weekend fill). The finding assumed `on-surface-subtle`; it was a false positive. |
| F-04 negative zero | Not fixed in M17 | FR-018 forbids view-model changes. **Follow-up bug:** `viewmodel.rs` prints `-0.0 of 45` (and 75, 150) in the marks table when a component's earned marks are negative zero. Normalise `-0.0` to `0.0` before formatting, with a unit test, in the next milestone that touches `viewmodel.rs` (OQ-4). |
| F-05 selected tab differs by weight only | Fixed | A `rule-width` `primary` bar on the bottom edge of the selected tab in `AmPmSegment` (components.slint), `TabButton` (timetable.slint) and `FilterTab` (assessments.slint). Visible in `03`, `04`, `09` (light and dark). |
| F-06 CheckBox/Switch accent colour | Skipped | Not possible cleanly: the Fluent `CheckBox` and `Switch` read `FluentPalette.accent-background`, an `out` property computed inside the widget style, not `Palette.accent-*`. Binding `Palette` has no effect on them. Left as recorded follow-up (OQ-2: keep the Fluent shapes; a custom square checkbox and switch with `primary` fill is a later milestone). |
| F-07 ragged course names | Fixed | `courses.slint` row: fixed code column (new token `code-column-width`, 96px, in `theme.slint`), name left-aligned and stretching, label/value pairs of the meta line separated by `space-md`. |
| F-08 truncated course names | Partly fixed | The course label already had the stretch factor. The three-button Awaiting rows need about 900px of content in 764px, so some elision cannot be avoided without a two-line row. Applied the router's fallback: `ListRow` side inset `space-sm` (was `space-lg`, which DESIGN.md `list-row.padding` names; this is a recorded departure). Gain is about 16px per row; "HUM 4741 . Business Communicati..." and the day panel still elide. Tried a `min-width` on the course label so the date columns elide first: it truncated the weekday, date and time on the longest row, which is worse than eliding the course tail, so it was reverted. Remaining options for the human: two-line rows, or a wider default window. |
| F-09 sub-headings | Fixed | `body-emphasis` for "Marks", "Sessions", "Add makeup class", the needed-in-final caption (courses.slint), "Saved rules" and "Components, in order" (rules.slint), "Reschedule" (timetable.slint) and the import result section line "Courses to update" (import.slint). The reorder of "Courses to update" before the count line is a view-model order (FR-018), not done. |
| F-10 vertical rhythm | Fixed | `courses.slint` detail card: Marks, Sessions and Add makeup class are groups with `space-sm` above and between parts, `space-md` above Secured/Projected and the needed-in-final table. `import.slint`: parent spacing `space-md`, `space-md` extra above the effective date, the Preview/Apply row and the results. |
| F-11 evidence gap | Done | New screens `13-focus-field`, `14-invalid-field`, `15-disabled-primary`, `16-update-banner`, `17-dashboard-legend`, light and dark. Hover is not captured (a static snapshot has no pointer). |
| F-12 to F-22 (P3) | Left alone | Nits, not part of the router's scope. F-13 is answered by F-08 above. F-12 is covered by `17-dashboard-legend-*`. F-14 is answered by `16-update-banner-*`: the `surface` fill with the `border-strong` rule reads as a banner in both themes (OQ-1 can stay at the default). F-19 is also visible in `14-invalid-field-*`: an error under "Due date" pushes the neighbouring fields down 20px, so the row loses its baseline; a layout fix (top-align the row) is a follow-up. |

Screenshots: all 24 reshot (the header band moves the switch and the content), plus the 10 new
ones, in `specs/017-m17-carbon-restyle/screenshots/`. The screenshots were taken with a temporary
harness (a debug scroll property and a timer driver, not committed) on scratch databases only.

## Human approval (2026-10-31)

The human reviewed the running app (not the screenshots) and approved M17 after these changes: appearance switch gap, Rules form columns, square status chips in a fixed column, text inputs with constant width, and the dark theme set to GitHub Dark Dimmed. The router recorded these answers (recommended defaults, accepted by "approve M17"):
- OQ-1 update banner: keep the surface fill with a `border-strong` rule.
- OQ-2 Fluent CheckBox/Switch: keep; square or themed widgets are a follow-up.
- OQ-3 white marker on the today cell: accepted (departure from DESIGN.md, DESIGN.md not edited in M17).
- OQ-4 negative zero: logged as a separate bug fix after M17.
- F-08: residual elision on the three-button Awaiting rows accepted.

