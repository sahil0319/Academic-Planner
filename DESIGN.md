---
version: alpha
name: Academic Planner
description: Quiet, dense, light-theme desktop UI for tracking courses, timetables, assessments, and attendance.
colors:
  primary: "#1F3A5F"
  primary-hover: "#2B4C78"
  on-primary: "#FFFFFF"
  neutral: "#F6F7F9"
  surface: "#FFFFFF"
  surface-alt: "#EEF0F3"
  border: "#D9DDE3"
  on-surface: "#1B1F24"
  on-surface-muted: "#5B6470"
  error: "#B42318"
  status-attended: "#1E7A46"
  status-attended-bg: "#E6F4EC"
  status-absent: "#B42318"
  status-absent-bg: "#FDECEA"
  status-cancelled: "#5B6470"
  status-cancelled-bg: "#EEF0F3"
  status-pending: "#8A5A00"
  status-pending-bg: "#FFF4DB"
  below-target: "#9A3412"
  below-target-bg: "#FFF1E6"
typography:
  title-lg:
    fontFamily: system-ui
    fontSize: 20px
    fontWeight: 600
    lineHeight: 1.3
  headline-md:
    fontFamily: system-ui
    fontSize: 15px
    fontWeight: 600
    lineHeight: 1.3
  body-md:
    fontFamily: system-ui
    fontSize: 13px
    fontWeight: 400
    lineHeight: 1.4
  body-sm:
    fontFamily: system-ui
    fontSize: 12px
    fontWeight: 400
    lineHeight: 1.4
  label-md:
    fontFamily: system-ui
    fontSize: 12px
    fontWeight: 600
    lineHeight: 1.2
  label-sm:
    fontFamily: system-ui
    fontSize: 11px
    fontWeight: 600
    lineHeight: 1.2
rounded:
  none: 0px
  sm: 4px
  md: 6px
  full: 9999px
spacing:
  xs: 4px
  sm: 8px
  md: 12px
  lg: 16px
  xl: 24px
  xxl: 32px
  nav-width: 200px
  row-height: 32px
  input-height: 28px
  window-width: 1100px
  window-height: 720px
components:
  nav-item:
    backgroundColor: "{colors.surface-alt}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
    rounded: "{rounded.sm}"
    height: "{spacing.row-height}"
    padding: "{spacing.sm}"
  nav-item-active:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.sm}"
    height: "{spacing.input-height}"
    padding: "{spacing.md}"
  button-primary-hover:
    backgroundColor: "{colors.primary-hover}"
  button-secondary:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    typography: "{typography.label-md}"
    rounded: "{rounded.sm}"
    height: "{spacing.input-height}"
    padding: "{spacing.md}"
  input-field:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
    rounded: "{rounded.sm}"
    height: "{spacing.input-height}"
    padding: "{spacing.sm}"
  input-error-text:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.error}"
    typography: "{typography.body-sm}"
  list-row:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    typography: "{typography.body-md}"
    height: "{spacing.row-height}"
    padding: "{spacing.sm}"
  divider:
    backgroundColor: "{colors.border}"
    height: 1px
  section-card:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface}"
    rounded: "{rounded.md}"
    padding: "{spacing.lg}"
  status-chip-attended:
    backgroundColor: "{colors.status-attended-bg}"
    textColor: "{colors.status-attended}"
    typography: "{typography.label-sm}"
    rounded: "{rounded.full}"
    padding: "{spacing.sm}"
  status-chip-absent:
    backgroundColor: "{colors.status-absent-bg}"
    textColor: "{colors.status-absent}"
    typography: "{typography.label-sm}"
    rounded: "{rounded.full}"
    padding: "{spacing.sm}"
  status-chip-cancelled:
    backgroundColor: "{colors.status-cancelled-bg}"
    textColor: "{colors.status-cancelled}"
    typography: "{typography.label-sm}"
    rounded: "{rounded.full}"
    padding: "{spacing.sm}"
  status-chip-pending:
    backgroundColor: "{colors.status-pending-bg}"
    textColor: "{colors.status-pending}"
    typography: "{typography.label-sm}"
    rounded: "{rounded.full}"
    padding: "{spacing.sm}"
  banner-below-target:
    backgroundColor: "{colors.below-target-bg}"
    textColor: "{colors.below-target}"
    typography: "{typography.body-md}"
    rounded: "{rounded.md}"
    padding: "{spacing.md}"
  empty-state:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.on-surface-muted}"
    typography: "{typography.body-md}"
    padding: "{spacing.xl}"
---

# Academic Planner Design System

## Overview

A quiet, dense productivity tool for university students. It should feel like a well-kept paper
planner: calm, legible, and fast to scan. Information density matters more than decoration. A student
opens it between classes, marks attendance in two clicks, checks what is due, and closes it.

- **Personality:** calm, precise, unobtrusive. No illustrations, gradients, or animation beyond
  basic hover feedback.
- **Density:** compact rows (32px), small body text (13px), tight spacing on a 4px grid.
- **Theme:** light only. A cool off-white page with white content cards.
- **Meaning is never carried by color alone.** Every status color is always paired with its text
  label ("Attended", "Absent", "Cancelled", "Pending", "Below target").

## Colors

A cool neutral base with one deep navy accent. Status colors are muted and only appear inside chips
and banners, always next to a text label.

- **Primary (#1F3A5F) "Planner Navy":** the only interactive accent. Used for the active nav item
  and the one primary button per panel. Hover uses #2B4C78.
- **Neutral (#F6F7F9):** the page background behind cards.
- **Surface (#FFFFFF):** cards, list rows, inputs.
- **Surface Alt (#EEF0F3):** the left nav background and table header rows.
- **Border (#D9DDE3):** 1px dividers between rows and around inputs and cards.
- **On Surface (#1B1F24) / On Surface Muted (#5B6470):** primary text and secondary text
  (metadata, times, rooms, empty states).
- **Error (#B42318):** inline validation messages under text fields.
- **Attended (#1E7A46 on #E6F4EC):** status chip for attended sessions.
- **Absent (#B42318 on #FDECEA):** status chip for absent sessions.
- **Cancelled (#5B6470 on #EEF0F3):** status chip for cancelled and rescheduled sessions.
- **Pending (#8A5A00 on #FFF4DB):** status chip for sessions awaiting attendance.
- **Below target (#9A3412 on #FFF1E6):** the dashboard banner and course badge when attendance is
  below the course's minimum.

All text/background pairs meet WCAG AA (4.5:1) for normal text.

## Typography

The platform system font throughout (`system-ui`), so the app feels native on macOS, Windows, and
Linux. Two weights only: 400 and 600.

- **Title (20px / 600):** page titles ("Dashboard", "Timetable"). One per page.
- **Headline (15px / 600):** section headings inside a page ("Today's classes", "Overdue").
- **Body (13px / 400):** list rows, form fields, banner text.
- **Body small (12px / 400):** metadata lines, inline validation errors.
- **Label (12px / 600):** button text, form field labels.
- **Label small (11px / 600):** status chip text.

Percentages and times are shown as plain text; the missing-value placeholder is an em dash "—".

## Layout

One window, default 1100 × 720. A fixed 200px left navigation column with five items (Dashboard,
Timetable, Courses, Assessments, Settings), and a scrollable content area on the right.

- A strict 4px spacing scale: 4, 8, 12, 16, 24, 32.
- Content area padding: 24px. Gap between sections: 24px. Gap inside a section: 8px.
- Lists are single-column rows, 32px tall, separated by 1px borders. No visual time grid; the
  timetable is a list grouped by weekday.
- Forms are inline panels or dialogs: label above field, 28px fields, error text directly below
  the field.
- Every list has an empty state: one muted sentence explaining what will appear and how to add it.

## Elevation & Depth

Flat. Hierarchy comes from tonal layers (neutral page, white cards) and 1px borders, not shadows.
Dialogs are the only elevated surface and use a single soft shadow over a dimmed backdrop.

## Shapes

Small, consistent radii: 4px for buttons, inputs, and nav items; 6px for cards and banners;
fully rounded pills for status chips.

## Components

- **Nav item:** 32px tall, 8px padding, body text. The active item uses the primary background
  with white text.
- **Primary button:** navy background, white label, 28px tall, 12px horizontal padding. At most one
  per panel (e.g. "Save").
- **Secondary button:** white background, 1px border, dark label. Used for row actions such as
  Attended / Absent / Cancelled.
- **Input field:** white, 1px border, 28px tall. On invalid input the border and helper text
  switch to the error color, and the message states the expected format (`YYYY-MM-DD`, `HH:MM`).
- **List row:** 32px, white, 1px bottom border. Primary text left, muted metadata next to it,
  status chip and actions right-aligned.
- **Status chip:** pill with the status text label in label-small type on its tinted background.
  The text is always present.
- **Below-target banner:** tinted card at the top of the dashboard listing each course below its
  attendance target, e.g. "CSE101 — 66.7% (target 75%)".
- **Section card:** white card, 6px radius, 16px padding, with a headline-md title.
- **Empty state:** muted body text centered in the section, e.g. "No classes today."

## Do's and Don'ts

- Do pair every status color with its text label; never show a colored dot or bar alone.
- Do use the primary color only for the active nav item and one primary button per panel.
- Do keep rows at 32px and text at 13px so a full week fits on screen without scrolling.
- Do show "—" for a missing percentage, never "0%" or "NaN".
- Don't hardcode colors or sizes in `.slint` files; reference `Theme.*` from `ui/theme.slint`.
- Don't add shadows, gradients, or decorative icons.
- Don't use more than two font weights on a single screen.
