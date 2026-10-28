# DESIGN.md conformance review (code-level pre-review)

**Scope**: `crates/app/ui/*.slint` (app-window, components, dashboard, timetable, courses,
assessments, settings) checked against every section of DESIGN.md (front-matter tokens, Overview,
Colors, Typography, Layout, Elevation, Shapes, Components, Do's and Don'ts).

**Method**: read the layout code only; sizes at the default 1100 × 720 window are estimated from
the layout tree, not measured. This is a pre-review. The §12.3 screenshot critique (T028) is still
pending and remains the authoritative visual check; items marked "left for T028" need a screenshot
to judge.

Line numbers refer to the files after the fixes in this review.

## Summary

| Severity | Found | Fixed | Left |
|----------|-------|-------|------|
| High     | 1     | 1     | 0    |
| Medium   | 9     | 6     | 3    |
| Low      | 10    | 4     | 6    |

## Checks that passed

- Status chips (`components.slint:153` `StatusChip`) always render their text label with the
  matching tinted background / foreground pair for Attended, Absent, Cancelled, Pending; the course
  detail "Below target" badge (`courses.slint:237`) shows the viewmodel label on the below-target
  pair. No color-only meaning in chips.
- Primary buttons: at most one per panel on every page (course form, makeup form, slot form, end
  form, reschedule panel, each settings card, restore confirm panel). Row actions are
  `SecondaryButton`.
- Fields are 30px (`Theme.input-height`), label above, error text below in `Theme.error` at body-sm;
  date/time errors from `viewmodel.rs` state the format ("Use YYYY-MM-DD, e.g. …", "Use HH:MM, …")
  and field labels carry the format hint.
- Serif (Georgia) only in `PageTitle` (one per page, plus the startup-error screen) and
  `SectionCard` titles. Everything else uses the body font; only weights 400 and 600 are used.
- Navigation: 208px (`Theme.nav-width`) on oat (`Theme.surface-alt`); active item uses
  `Theme.surface-selected` with label weight; accent never used for navigation.
- Section cards: paper, `radius-md` (10px), 1px border, 16px padding, 8px inner gap. Content
  padding 24px (`app-window.slint:108`).
- Every list has a muted empty state (today, next class, upcoming, overdue, awaiting, week, slots,
  courses, sessions, assessments, holidays, no term). Missing percentage shows "—"
  (`viewmodel.rs` `DASH`).
- No raw colors or sizes outside `theme.slint`; no shadows anywhere (no dialogs exist yet); no pure
  white background or pure black text (`on-primary` white is only text on terracotta, as specified).

## Findings

| # | Severity | File:line | Issue | Fixed? |
|---|----------|-----------|-------|--------|
| 1 | High | settings.slint:95, 167, 205 | Field rows used `alignment: start`, so each `Field` shrank to its preferred width (label or typed text). The Term "Name" field was about as wide as the word "Name", so typed text clipped; widths also changed while typing. | Yes: rows now stretch fields evenly, like the other forms. |
| 2 | Medium | components.slint:208 | `ListRow` padding of 4px top and bottom plus a 30px button made every row with an action 38px tall instead of 34px. | Yes: vertical inset is now `(Theme.row-height - Theme.input-height) / 2`, so button rows are 34px. |
| 3 | Medium | dashboard.slint:128, 151, 163, 175; timetable.slint:283, 444; courses.slint:404; assessments.slint:197; settings.slint:137 | List rows sat directly in the `SectionCard` layout and inherited its 8px spacing, so rows had 8px gaps instead of being flush and separated by the 1px divider. | Yes: each list is wrapped in a zero-spacing `VerticalLayout` (shown only when the list is non-empty, so the empty state keeps the normal 8px gap). |
| 4 | Medium | dashboard.slint:104, timetable.slint:480, courses.slint:115, assessments.slint:140, settings.slint:77 | Gap between page sections was 16px (`space-lg`); DESIGN.md Layout says 24px. | Yes: `Theme.space-xl`. |
| 5 | Medium | components.slint:243 | The below-target banner showed only course lines ("CSE101 — 66.7% (target 75%)") on the tinted background, with no "Below target" text label (Overview: every status color is paired with its label). Lines also used weight 600 instead of body-md 400. | Yes: banner now starts with a "Below target" label (label-md) and lines use body-md weight. |
| 6 | Medium | timetable.slint:131 | Timetable `ComboField` combo box had the fluent style height, not the 30px field height (the assessments copy already set it). | Yes: `height: Theme.input-height`. |
| 7 | Medium | timetable.slint:297 | Week-row status chip was a direct child of the row layout, so the pill stretched to the full row content height. | Yes: centered in a `VerticalLayout` like the other pages. |
| 8 | Medium | settings.slint:84 (also dashboard.slint:111, timetable.slint:506, courses.slint:122, assessments.slint:181) | One `message` property carries both errors and success notes. Settings shows it in ink, so errors (backup, restore, CSV export failures) are not in the error color; the other pages always use the error color (their messages are errors in practice). Telling them apart needs a new bridge property, which is a contract change. | No: left; needs a decision on the UI bridge contract. |
| 9 | Medium | assessments.slint:199, courses.slint:129, settings.slint:220, timetable.slint:131, assessments.slint:111 | Standard widgets (`CheckBox`, `Switch`, `ComboBox`, fluent-light style) draw their own colors: blue accent for checked/on states and near-white fills, outside the warm palette and the terracotta-only accent. | No: left for T028; fixing needs custom components. |
| 10 | Medium | courses.slint:406 (course list row), assessments.slint:223 (row with notes) | Two-line rows (course code/name over instructor/room/target; assessment title over notes) are taller than 34px. Content is readable; DESIGN.md asks for 34px rows. | No: left for T028 to judge from a screenshot. |
| 11 | Low | courses.slint:70 | `Caption` used label-sm (11px), which DESIGN.md reserves for status chip text, for field captions and sub-headings ("Sessions", "Add makeup class"). | Yes: label-md, like field labels. |
| 12 | Low | courses.slint:138 | "New course" was a terracotta primary button in the toolbar, while "Add slot" on Timetable is secondary; with the form open the page showed two terracotta buttons. | Yes: secondary button; terracotta stays on the form's Save. |
| 13 | Low | timetable.slint:178 | Session panel status action was labelled "Cancel" (reads like dismissing the panel); other pages use "Cancelled" for the same action. | Yes: "Cancelled". |
| 14 | Low | dashboard.slint:37, timetable.slint:96 | Row text had no `overflow: elide`; a long course name could push the row wider than the card instead of eliding. | Yes: `overflow: elide`. |
| 15 | Low | components.slint:14, assessments.slint:74 | Hover on nav items and filter tabs uses the same `surface-selected` fill as the active item, so a hovered item looks active. DESIGN.md defines no hover token. | No: left; needs a token decision. |
| 16 | Low | courses.slint:423 | The "Archived" marker reuses the cancelled chip colors for a non-status meaning (text label is present). | No: left. |
| 17 | Low | courses.slint:267 | Course detail sessions list is a fixed-height `ScrollView` inside the page `ScrollView` (nested scrolling). | No: left for T028. |
| 18 | Low | timetable.slint:146, settings.slint:269 | Inline panels (session panel, restore confirm) use the oat fill, which DESIGN.md names for nav and list header rows only. Readable and distinct from cards. | No: left. |
| 19 | Low | assessments.slint:181 vs others | Page messages use body-md on Dashboard / Assessments and body-sm on Timetable / Courses. | No: left (folds into finding 8). |
| 20 | Low | all pages | Text clipping at 1100 × 720: estimated content width is about 830px; the widest row (course detail session row with four actions) needs about 720px, so nothing clips. Needs confirmation by screenshot. | No: left for T028 (estimate only). |

## T028 critique follow-ups (P2, not fixed per §12.3)

Source: `.impeccable/critique/2026-10-28T21-17-59Z__crates-app-ui.md` (score 22/40). The three P1
items (dashboard order, keyboard focus and accessibility of custom controls, empty-state copy) are
fixed. The P2 items below stay open; §12.3 fixes only high-severity items.

- [ ] P2: Timetable week view uses one card per weekday (timetable.slint, week tab), which breaks
  the "full week fits" rule. Suggested: one card with oat weekday header rows, a today marker, and
  whole-row tap targets.
- [ ] P2: Off-system states. Nav item and filter tab hover use the active `surface-selected` fill,
  so a hovered item looks active (components.slint `NavItem`, assessments.slint `FilterTab`; see
  finding 15). Standard Fluent `Switch` / `CheckBox` draw a blue accent (settings.slint Switch,
  assessments.slint and courses.slint CheckBox; see finding 9). Suggested: hover `Theme.surface`,
  custom terracotta toggle and check components.
