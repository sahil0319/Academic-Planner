---
version: "2.0"
name: Academic Planner
description: Light, square, dense desktop UI for tracking courses, timetables, assessments, attendance, and marks. Derived from IBM Carbon (the same family as the Paper-Triage-Carbon design of agentic-paper-search), with a dark theme in the colours of the VS Code theme GitHub Dark Dimmed (human decision 2026-10-31) that keeps the Carbon structure. IBM Plex Sans at 400 and 600.
colors:
  primary: "#0F62FE"
  primary-hover: "#0050E6"
  primary-active: "#002D9C"
  on-primary: "#FFFFFF"
  neutral: "#FFFFFF"
  surface: "#F4F4F4"
  surface-alt: "#FFFFFF"
  surface-hover: "#E8E8E8"
  surface-selected: "#E0E0E0"
  surface-inverse: "#161616"
  on-surface-inverse: "#FFFFFF"
  field: "#FFFFFF"
  border: "#E0E0E0"
  border-strong: "#8D8D8D"
  on-surface: "#161616"
  on-surface-muted: "#525252"
  on-surface-subtle: "#6F6F6F"
  link: "#0F62FE"
  focus: "#0F62FE"
  error: "#DA1E28"
  status-attended: "#0E6027"
  status-attended-bg: "#A7F0BA"
  status-absent: "#A2191F"
  status-absent-bg: "#FFD7D9"
  status-cancelled: "#393939"
  status-cancelled-bg: "#E0E0E0"
  status-pending: "#8E6A00"
  status-pending-bg: "#FCF4D6"
  below-target: "#A2191F"
  below-target-bg: "#FFF1F1"
  progress-track: "#E0E0E0"
  progress-fill: "#161616"
  progress-target: "#0F62FE"
  calendar-holiday: "#D0E2FF"
  calendar-midterm: "#D4BBFF"
  calendar-final: "#FFAFD2"
  calendar-public-holiday: "#9EF0F0"
  success: "#24A148"
  success-bg: "#DEFBE6"
  dark-primary: "#0F62FE"
  dark-primary-hover: "#0050E6"
  dark-primary-active: "#002D9C"
  dark-on-primary: "#FFFFFF"
  dark-neutral: "#22272E"
  dark-surface: "#2D333B"
  dark-surface-alt: "#1C2128"
  dark-surface-hover: "#373E47"
  dark-surface-selected: "#444C56"
  dark-surface-inverse: "#F4F4F4"
  dark-on-surface-inverse: "#161616"
  dark-field: "#22272E"
  dark-border: "#444C56"
  dark-border-strong: "#768390"
  dark-on-surface: "#CDD9E5"
  dark-on-surface-muted: "#B4C1CE"
  dark-on-surface-subtle: "#9CA9B7"
  dark-link: "#5FA3F7"
  dark-focus: "#FFFFFF"
  dark-error: "#FF8389"
  dark-status-attended: "#6FDC8C"
  dark-status-attended-bg: "#044317"
  dark-status-absent: "#FFB3B8"
  dark-status-absent-bg: "#750E13"
  dark-status-cancelled: "#E0E0E0"
  dark-status-cancelled-bg: "#525252"
  dark-status-pending: "#F1C21B"
  dark-status-pending-bg: "#483700"
  dark-below-target: "#FFB3B8"
  dark-below-target-bg: "#4A0C10"
  dark-progress-track: "#444C56"
  dark-progress-fill: "#CDD9E5"
  dark-progress-target: "#5FA3F7"
  dark-calendar-holiday: "#002D9C"
  dark-calendar-midterm: "#6929C4"
  dark-calendar-final: "#9F1853"
  dark-calendar-public-holiday: "#005D5D"
  dark-success: "#24A148"
  dark-success-bg: "#071908"
typography:
  title-lg:
    fontFamily: IBM Plex Sans
    fontSize: 24px
    fontWeight: 400
    lineHeight: 32px
  headline-md:
    fontFamily: IBM Plex Sans
    fontSize: 16px
    fontWeight: 600
    lineHeight: 22px
  numeral-md:
    fontFamily: IBM Plex Sans
    fontSize: 24px
    fontWeight: 400
    lineHeight: 32px
  body-md:
    fontFamily: IBM Plex Sans
    fontSize: 14px
    fontWeight: 400
    lineHeight: 20px
  body-emphasis:
    fontFamily: IBM Plex Sans
    fontSize: 14px
    fontWeight: 600
    lineHeight: 18px
  body-sm:
    fontFamily: IBM Plex Sans
    fontSize: 12px
    fontWeight: 400
    lineHeight: 16px
  label-md:
    fontFamily: IBM Plex Sans
    fontSize: 12px
    fontWeight: 400
    lineHeight: 16px
  label-sm:
    fontFamily: IBM Plex Sans
    fontSize: 12px
    fontWeight: 400
    lineHeight: 16px
rounded:
  none: 0px
  full: 9999px
spacing:
  xs: 4px
  sm: 8px
  md: 12px
  lg: 16px
  xl: 24px
  xxl: 32px
  xxxl: 48px
  nav-width: 240px
  nav-item-height: 32px
  row-height: 40px
  input-height: 32px
  progress-height: 8px
  window-width: 1100px
  window-height: 720px
components:
  nav-item:
    backgroundColor: "{colors.surface-alt}"
    textColor: "{colors.on-surface-muted}"
    typography: "{typography.body-md}"
    rounded: "{rounded.none}"
    height: "{spacing.nav-item-height}"
    padding: "{spacing.lg}"
  nav-item-hover:
    backgroundColor: "{colors.surface-hover}"
    textColor: "{colors.on-surface}"
  nav-item-active:
    backgroundColor: "{colors.surface-selected}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-emphasis}"
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    typography: "{typography.body-md}"
    rounded: "{rounded.none}"
    height: "{spacing.input-height}"
    padding: "{spacing.lg}"
  button-primary-hover:
    backgroundColor: "{colors.primary-hover}"
    textColor: "{colors.on-primary}"
  button-primary-active:
    backgroundColor: "{colors.primary-active}"
    textColor: "{colors.on-primary}"
  button-secondary:
    backgroundColor: "{colors.field}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
    rounded: "{rounded.none}"
    height: "{spacing.input-height}"
    padding: "{spacing.lg}"
  button-secondary-hover:
    backgroundColor: "{colors.surface-hover}"
    textColor: "{colors.on-surface}"
  button-danger:
    backgroundColor: "{colors.field}"
    textColor: "{colors.error}"
    typography: "{typography.body-md}"
    rounded: "{rounded.none}"
    height: "{spacing.input-height}"
    padding: "{spacing.lg}"
  button-github:
    backgroundColor: "{colors.surface-inverse}"
    textColor: "{colors.on-surface-inverse}"
    typography: "{typography.body-md}"
    rounded: "{rounded.none}"
    height: "{spacing.input-height}"
    padding: "{spacing.lg}"
  input-field:
    backgroundColor: "{colors.field}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
    rounded: "{rounded.none}"
    height: "{spacing.input-height}"
    padding: "{spacing.lg}"
  input-label:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface-muted}"
    typography: "{typography.label-md}"
  input-error-text:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.error}"
    typography: "{typography.body-sm}"
  list-row:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
    height: "{spacing.row-height}"
    padding: "{spacing.lg}"
  divider:
    backgroundColor: "{colors.border}"
    height: 1px
  section-card:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    rounded: "{rounded.none}"
    padding: "{spacing.lg}"
  section-meta:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface-subtle}"
    typography: "{typography.body-sm}"
  status-chip-attended:
    backgroundColor: "{colors.status-attended-bg}"
    textColor: "{colors.status-attended}"
    typography: "{typography.label-sm}"
    rounded: "{rounded.none}"
    padding: "{spacing.sm}"
  status-chip-absent:
    backgroundColor: "{colors.status-absent-bg}"
    textColor: "{colors.status-absent}"
    typography: "{typography.label-sm}"
    rounded: "{rounded.none}"
    padding: "{spacing.sm}"
  status-chip-cancelled:
    backgroundColor: "{colors.status-cancelled-bg}"
    textColor: "{colors.status-cancelled}"
    typography: "{typography.label-sm}"
    rounded: "{rounded.none}"
    padding: "{spacing.sm}"
  status-chip-pending:
    backgroundColor: "{colors.status-pending-bg}"
    textColor: "{colors.status-pending}"
    typography: "{typography.label-sm}"
    rounded: "{rounded.none}"
    padding: "{spacing.sm}"
  banner-below-target:
    backgroundColor: "{colors.below-target-bg}"
    textColor: "{colors.below-target}"
    typography: "{typography.body-md}"
    rounded: "{rounded.none}"
    padding: "{spacing.lg}"
  toast:
    backgroundColor: "{colors.success-bg}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
    rounded: "{rounded.none}"
    padding: "{spacing.md}"
    width: 320px
  toast-close:
    textColor: "{colors.on-surface}"
    rounded: "{rounded.none}"
    size: 24px
  toast-timer-track:
    backgroundColor: "{colors.border}"
    height: 3px
  toast-timer-bar:
    backgroundColor: "{colors.success}"
    height: 3px
  term-line:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.on-surface-muted}"
    typography: "{typography.body-md}"
  calendar-day-holiday:
    backgroundColor: "{colors.calendar-holiday}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
  calendar-day-midterm:
    backgroundColor: "{colors.calendar-midterm}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
  calendar-day-final:
    backgroundColor: "{colors.calendar-final}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
  calendar-day-public-holiday:
    backgroundColor: "{colors.calendar-public-holiday}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
  progress-bar:
    backgroundColor: "{colors.progress-track}"
    textColor: "{colors.progress-fill}"
    rounded: "{rounded.none}"
    height: "{spacing.progress-height}"
  progress-target-marker:
    backgroundColor: "{colors.progress-target}"
    height: "{spacing.progress-height}"
  text-link:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.link}"
    typography: "{typography.body-md}"
  focus-ring:
    backgroundColor: "{colors.focus}"
    height: 2px
  control-edge:
    backgroundColor: "{colors.border-strong}"
    height: 1px
  success-rule:
    backgroundColor: "{colors.success}"
    height: 3px
  empty-state:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface-muted}"
    typography: "{typography.body-md}"
    padding: "{spacing.xl}"
  dark-nav-item:
    backgroundColor: "{colors.dark-surface-alt}"
    textColor: "{colors.dark-on-surface-muted}"
    typography: "{typography.body-md}"
  dark-nav-item-hover:
    backgroundColor: "{colors.dark-surface-hover}"
    textColor: "{colors.dark-on-surface}"
  dark-nav-item-active:
    backgroundColor: "{colors.dark-surface-selected}"
    textColor: "{colors.dark-on-surface}"
    typography: "{typography.body-emphasis}"
  dark-button-primary:
    backgroundColor: "{colors.dark-primary}"
    textColor: "{colors.dark-on-primary}"
    typography: "{typography.body-md}"
  dark-button-primary-hover:
    backgroundColor: "{colors.dark-primary-hover}"
    textColor: "{colors.dark-on-primary}"
  dark-button-primary-active:
    backgroundColor: "{colors.dark-primary-active}"
    textColor: "{colors.dark-on-primary}"
  dark-progress-target-marker:
    backgroundColor: "{colors.dark-progress-target}"
  dark-text-link:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-link}"
    typography: "{typography.body-md}"
  dark-focus-ring:
    backgroundColor: "{colors.dark-focus}"
    height: 2px
  dark-control-edge:
    backgroundColor: "{colors.dark-border-strong}"
    height: 1px
  dark-success-rule:
    backgroundColor: "{colors.dark-success}"
    height: 3px
  dark-button-secondary:
    backgroundColor: "{colors.dark-field}"
    textColor: "{colors.dark-on-surface}"
    typography: "{typography.body-md}"
  dark-button-secondary-hover:
    backgroundColor: "{colors.dark-surface-hover}"
    textColor: "{colors.dark-on-surface}"
  dark-button-danger:
    backgroundColor: "{colors.dark-field}"
    textColor: "{colors.dark-error}"
    typography: "{typography.body-md}"
  dark-button-github:
    backgroundColor: "{colors.dark-surface-inverse}"
    textColor: "{colors.dark-on-surface-inverse}"
    typography: "{typography.body-md}"
  dark-input-field:
    backgroundColor: "{colors.dark-field}"
    textColor: "{colors.dark-on-surface}"
    typography: "{typography.body-md}"
  dark-input-label:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-on-surface-muted}"
    typography: "{typography.label-md}"
  dark-input-error-text:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-error}"
    typography: "{typography.body-sm}"
  dark-list-row:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-on-surface}"
    typography: "{typography.body-md}"
  dark-divider:
    backgroundColor: "{colors.dark-border}"
    height: 1px
  dark-section-card:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-on-surface}"
  dark-section-meta:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-on-surface-subtle}"
    typography: "{typography.body-sm}"
  dark-status-chip-attended:
    backgroundColor: "{colors.dark-status-attended-bg}"
    textColor: "{colors.dark-status-attended}"
    typography: "{typography.label-sm}"
  dark-status-chip-absent:
    backgroundColor: "{colors.dark-status-absent-bg}"
    textColor: "{colors.dark-status-absent}"
    typography: "{typography.label-sm}"
  dark-status-chip-cancelled:
    backgroundColor: "{colors.dark-status-cancelled-bg}"
    textColor: "{colors.dark-status-cancelled}"
    typography: "{typography.label-sm}"
  dark-status-chip-pending:
    backgroundColor: "{colors.dark-status-pending-bg}"
    textColor: "{colors.dark-status-pending}"
    typography: "{typography.label-sm}"
  dark-banner-below-target:
    backgroundColor: "{colors.dark-below-target-bg}"
    textColor: "{colors.dark-below-target}"
    typography: "{typography.body-md}"
  dark-toast:
    backgroundColor: "{colors.dark-success-bg}"
    textColor: "{colors.dark-on-surface}"
    typography: "{typography.body-md}"
  dark-toast-close:
    textColor: "{colors.dark-on-surface}"
  dark-toast-timer-track:
    backgroundColor: "{colors.dark-border}"
    height: 3px
  dark-toast-timer-bar:
    backgroundColor: "{colors.dark-success}"
    height: 3px
  dark-term-line:
    backgroundColor: "{colors.dark-neutral}"
    textColor: "{colors.dark-on-surface-muted}"
    typography: "{typography.body-md}"
  dark-calendar-day-holiday:
    backgroundColor: "{colors.dark-calendar-holiday}"
    textColor: "{colors.dark-on-surface}"
  dark-calendar-day-midterm:
    backgroundColor: "{colors.dark-calendar-midterm}"
    textColor: "{colors.dark-on-surface}"
  dark-calendar-day-final:
    backgroundColor: "{colors.dark-calendar-final}"
    textColor: "{colors.dark-on-surface}"
  dark-calendar-day-public-holiday:
    backgroundColor: "{colors.dark-calendar-public-holiday}"
    textColor: "{colors.dark-on-surface}"
  dark-progress-bar:
    backgroundColor: "{colors.dark-progress-track}"
    textColor: "{colors.dark-progress-fill}"
  dark-empty-state:
    backgroundColor: "{colors.dark-surface}"
    textColor: "{colors.dark-on-surface-muted}"
    typography: "{typography.body-md}"
---

# Academic Planner Design System

Version 2 (human decision 2026-10-30). This is the target of milestone M17 (docs/spec.md §13.12).
Until M17 is built, `ui/theme.slint` still holds the version 1 values (warm cream, terracotta,
Georgia; see this file at tag `v0.2.1`). Code written before M17 keeps using `Theme.*` tokens only,
so M17 is a token swap plus the component changes described here.

## Overview

A quiet, square, dense tool for one student who checks it between classes. The look comes from IBM
Carbon, the same family as the Paper-Triage-Carbon design of agentic-paper-search: white page, light
grey `#F4F4F4` cards, charcoal ink, and one IBM Blue accent. Information density matters more than
decoration.

- **Personality:** calm, precise, unobtrusive. No illustrations, gradients, glass, glow, or
  animation beyond short colour changes on hover and press.
- **Density:** 40px list rows, 32px controls, 14px body text, a 4px grid.
- **Theme:** light by default, with a dark theme in the colours of the VS Code theme GitHub Dark Dimmed, a
  blue-tinted slate (human decision 2026-10-31, replacing the Carbon g100 neutral greys; the Carbon
  layer structure stays): slate `#22272E` page and fields, `#2D333B` cards, `#1C2128` navigation
  column, light slate text. The three-way System / Light / Dark switch
  stays in the window's top-right corner on every page.
- **Square:** everything you click or type into has square corners, status chips included.
- **Meaning is never carried by colour alone.** Every status colour is paired with its text label
  ("Attended", "Absent", "Cancelled", "Pending", "Below target", "On track", "Watch", "At risk",
  "Not enough data", "Holiday", "Mid exam", "Final exam", "Public holiday"), and every bar has its number printed beside it, and every calendar fill is named in the legend.

## Colors

Neutral greys (blue-tinted slate in dark) with one blue accent. Colour appears only on actions, focus, links, errors, status
chips, the below-target banner, and the progress-bar target marker.

- **Primary (#0F62FE) "IBM Blue":** the one primary button per panel, the active nav bar, focus
  rings, links, today in the calendar. Hover #0050E6, press #002D9C. The same blue stays the primary
  fill in dark; dark links and the dark target marker use #5FA3F7.
- **Neutral (#FFFFFF) / dark (#22272E):** the page behind cards. **Surface Alt (#FFFFFF) / dark
  (#1C2128):** the nav column.
- **Surface (#F4F4F4) / dark (#2D333B):** section cards, list rows, panels.
- **Field (#FFFFFF) / dark (#22272E):** text fields and secondary buttons on a card. Fields
  alternate against their layer, as in Carbon (in dark they are darker than the card, as in VS Code).
- **Surface Hover (#E8E8E8) / dark (#373E47)** and **Surface Selected (#E0E0E0) / dark
  (#444C56):** hover fills and the active nav item or selected row. Selection is a tonal fill plus
  weight 600 and a 3px blue bar, not colour alone.
- **Surface Inverse (#161616) / dark (#F4F4F4):** the GitHub button and the selected segment of the
  appearance switch, with on-surface-inverse text.
- **Border (#E0E0E0) / dark (#444C56):** 1px dividers between rows and around cards. Decorative.
- **Border Strong (#8D8D8D) / dark (#768390):** edges of controls (field bottom rule, secondary
  buttons, checkbox). 3.0:1 on the light card, 3.3:1 on the dark card and 3.9:1 on the dark field.
- **On Surface (#161616) / dark (#CDD9E5), On Surface Muted (#525252) / dark (#B4C1CE), On Surface
  Subtle (#6F6F6F) / dark (#9CA9B7):** the whole text ladder. Do not add greys. Subtle text is never
  placed on surface-selected.
- **Error (#DA1E28) / dark (#FF8389):** inline validation messages, danger buttons.
- **Attended (#0E6027 on #A7F0BA; dark #6FDC8C on #044317):** green chip, also "On track".
- **Absent (#A2191F on #FFD7D9; dark #FFB3B8 on #750E13):** red chip, also "At risk".
- **Cancelled (#393939 on #E0E0E0; dark #E0E0E0 on #525252):** grey chip for cancelled,
  rescheduled, and "Not enough data".
- **Pending (#8E6A00 on #FCF4D6; dark #F1C21B on #483700):** yellow chip for sessions awaiting
  attendance, also "Watch".
- **Below target (#A2191F on #FFF1F1; dark #FFB3B8 on #4A0C10):** the dashboard banner, with a 3px
  error-colour rule on its left edge (Carbon notification).
- **Progress track (#E0E0E0 / dark #444C56), fill (#161616 / dark #CDD9E5), target marker
  (#0F62FE / dark #5FA3F7):** the attendance bar on the Progress page.
- **Calendar days off:** each kind has its own fill behind the day number, and the legend names
  it. Holiday #D0E2FF / dark #002D9C (blue), Mid exam #D4BBFF / dark #6929C4 (purple), Final exam
  #FFAFD2 / dark #9F1853 (magenta), Public holiday #9EF0F0 / dark #005D5D (teal). Day numbers stay
  in on-surface on every fill (at least 7:1 light, 5.3:1 dark).
- **Success (#24A148 rule on #DEFBE6; dark #24A148 on #071908):** the toast
  notification (see Components): every confirmation of a completed action, such as "Term saved.",
  with a 3px success rule on the left edge. Text is on-surface.

Every text/background pair above was checked: all meet WCAG AA (4.5:1) for normal text; the lowest
are pending (4.52:1) and the light link and error on the card (4.55:1). In dark the lowest text pair
is subtle text on surface-hover (4.52:1), then the link on the card (4.90:1) and muted text on
surface-selected (4.75:1); control edges (border-strong) and the focus ring meet 3:1 on the dark
page, card, and field (lowest 3.29:1 on the card). In dark, the selected fill (#444C56), hover fill
(#373E47), and field (#22272E) are three different values. The dark border equals the selected
fill and the dark field equals the page; neither pair is ever drawn against itself.

## Typography

One family, **IBM Plex Sans**, at weights 400 (Regular) and 600 (SemiBold) only. The two TTF files
come from IBM's official release (github.com/IBM/plex, SIL Open Font License 1.1) and are embedded in
the app, so every platform looks the same. Glyphs missing from Plex fall back to the platform font.
The licence goes into `THIRD-PARTY-NOTICES.html`.

- **Title (24px / 400):** page titles. One per page.
- **Headline (16px / 600):** section card titles ("Today's classes", "Marks").
- **Numeral (24px / 400):** big figures on the Progress page (attendance %, projected %).
- **Body (14px / 400):** list rows, field text, button labels, banner text.
- **Body emphasis (14px / 600):** course codes in rows, the active nav item.
- **Body small (12px / 400):** metadata lines, inline validation errors.
- **Label (12px / 400):** field labels, figure captions, status chip text. Labels are sentence
  case, never all caps.

Hierarchy comes from size and the 400/600 pair, never from colour. The missing-value placeholder is
an em dash "—".

## Layout

One window, default 1100 × 720. A fixed 240px left navigation column (Dashboard, Progress,
Timetable, Courses, Assessments, Rules, Settings, About) on the page colour with a 1px border on its
right edge, and a scrollable content area on the right. The sidebar stays (human decision
2026-10-30); there is no top bar.

- 4px grid: 4, 8, 12, 16, 24, 32, 48.
- Content area padding 24px; 32px above a section card, 16px inside it.
- Lists are single-column rows, 40px tall, separated by 1px borders. The timetable stays a list
  grouped by weekday.
- Forms are inline panels: label above field, 32px fields, error text directly below.
- Every list has an empty state: one muted sentence saying what will appear and how to add it.

## Elevation & Depth

Flat. Depth comes from layers (page, then card, then field or selected fill) and 1px rules. No
shadows anywhere; confirm panels are inline, not overlays.

## Shapes

Radius 0 on buttons, fields, nav items, cards, banners, panels, the progress bar, calendar cells,
the appearance switch, and status chips. There are no rounded or pill shapes.

## Components

- **Nav item:** 32px tall, 16px left padding, body text in on-surface-muted. Hover: surface-hover
  fill and on-surface text. Active: surface-selected fill, body-emphasis on-surface text, and a 3px
  primary bar on the left edge.
- **Primary button:** primary fill, white body label, 32px tall, 16px padding, square. At most one
  per panel. Hover primary-hover, press primary-active. Disabled: surface-selected fill,
  on-surface-subtle text.
- **Secondary button:** field fill, 1px border-strong edge, on-surface label. Hover surface-hover.
  Row actions such as Attended / Absent / Cancelled.
- **Danger button:** field fill, 1px error edge, error label. Used for "Delete selected" and the
  confirm of a destructive action; never more than one per panel.
- **GitHub button:** surface-inverse fill, on-surface-inverse label and 16px GitHub mark, square.
- **Focus:** a 2px focus-colour ring inset on every focusable control (Carbon). Filled buttons add a
  1px inner ring in on-primary.
- **Input field:** field fill, no side or top border, a 1px border-strong bottom rule, 32px tall,
  16px inline padding. Focus: 2px inset focus ring. Invalid: 2px inset error ring and the message in
  body-small error below, stating the expected format (`DD-MM-YYYY`, digits for time).
- **Checkbox and switch:** the platform-neutral Slint widgets, recoloured with these tokens.
- **List row:** 40px, surface fill, 1px bottom border. Primary text left, muted metadata next to
  it, status chip and actions right-aligned.
- **Status chip:** square, 12px label text on its tinted fill, 8px horizontal padding, 4px vertical
  padding. Text always present. Chips sit in a fixed-width, left-aligned column so the chips of all
  rows start at the same x; a long "Rescheduled → date" label elides instead of breaking the column.
- **Below-target banner:** below-target-bg fill, 3px error rule on the left, body text listing each
  course below target, e.g. "CSE101 — 66.7% (target 75%)".
- **Section card:** surface fill, square, no border, 16px padding, headline title.
- **Progress card (Progress page):** a section card per course: code and name in body-emphasis,
  the status chip on the right; two columns of figures (Attendance, Marks) in numeral type with
  label captions; the attendance bar (8px, square, progress-track with a progress-fill segment and a
  2px progress-target marker at the target, the percentage printed beside it); then one body line
  each for "classes you can still miss" and the best reachable grade.
- **Empty state:** muted body text in the card, e.g. "No classes today."
- **Mini calendar:** square 36px day cells, Monday first, weekday initials in label text. Today: a
  primary-filled square with white number. Selected day: a 2px focus-colour inset outline, so it
  stays visible on any fill. Days off use their kind's fill (holiday, mid exam, final exam); public
  holidays that are not days off use the public-holiday fill; weekends use surface-hover; a day with
  an assessment due shows a 4px error-colour square under the number. The legend under the grid
  names every fill in use plus the due marker, each with a 12px swatch, and wraps to two lines when
  needed.
- **Day panel lines:** "Holiday: {label}", "Mid exam: {label}", "Final exam: {label}", and
  "Public holiday: {name}" in body text, above the day's classes.
- **Term line (Dashboard):** body text in on-surface-muted directly under the page title, e.g.
  "Spring 2026 · January 5 – May 1, 2026 · Week 3 of 17". When the term has ended, the sentence
  "Archive it in Settings." follows in link colour as a text button.
- **Term card (Settings):** the active term read-only (name in body-emphasis, dates and week line in
  muted body) with Edit (secondary) and Archive (danger) buttons; Archive is disabled before the end
  date with a muted line saying when it becomes available. The form appears in the card only when
  creating or editing.
- **Toast notification:** the one way the app confirms a completed action ("Term saved.", "Rule set
  saved.", "Backup saved to …"). It is a 320px panel, success-bg fill, 3px success rule on the left
  edge, on-surface body text, 12px padding, with a 24px close button (a cross icon, accessible label
  "Dismiss") in the top-right corner. A 3px timer bar runs along the bottom edge: a border-coloured
  track with a success-coloured bar that starts full and shrinks from right to left, linearly, over
  5 seconds; the toast disappears when the bar is empty. Toasts sit in the bottom-right corner of the
  window, 16px from the edges, above every page and dialog, stacked 8px apart with the newest at the
  bottom, at most three at once. No shadow, no slide or fade: the toast appears and disappears at
  once. Errors are not toasts: they stay inline next to the thing that failed until fixed.
- **Past terms list (Settings):** a section card of 40px rows: name in body-emphasis, date range,
  then "{n} courses", "Attendance {pct}", "GPA {gpa}" as muted figures. Read-only: no buttons.
- **Exam period cards (Settings):** Mid exams and Final exams use the same layout as the Holidays
  card (range input, label, Add, list of ranges with Remove); a 12px swatch of the kind's calendar
  fill sits before each card title so the link to the calendar colour is visible.
- **Public holidays card (Settings):** the country choice, a muted status line ("Updated today",
  "Couldn't load public holidays" with a Retry text button), and a list of the term's public
  holidays with checkboxes and one secondary button "Add selected as holidays".
- **Appearance switch:** a square segmented control (Carbon content switcher) 16px from the
  top-right corner: three 28px icon segments (monitor, sun, moon) with a 1px border-strong edge;
  the selected segment is surface-inverse with on-surface-inverse icon. Accessible labels "Use
  system appearance", "Light", "Dark".
- **Bi-weekly marker:** sessions and slots of a bi-weekly slot carry the text "Every other week" in
  body-small muted next to the time. Text only, no icon.

## Do's and Don'ts

- Do pair every status colour with its text label; never show a coloured dot or bar alone.
- Do reserve blue for the one primary button per panel, the active nav bar, links, focus, today, and
  the progress target marker.
- Do keep corners square everywhere, status chips included.
- Do keep rows at 40px and body text at 14px so a full week fits on screen.
- Do show "—" for a missing percentage, never "0%" or "NaN".
- Don't use weights other than 400 and 600, or all-caps tracked labels.
- Don't hardcode colours or sizes in `.slint` files; reference `Theme.*` from `ui/theme.slint`.
- Don't add shadows, gradients, rounded buttons, or a second accent colour.
