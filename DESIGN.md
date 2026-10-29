---
version: alpha
name: Academic Planner
description: Warm, quiet, dense light-theme desktop UI for tracking courses, timetables, assessments, and attendance.
colors:
  primary: "#B4532F"
  primary-hover: "#9C4526"
  on-primary: "#FFFFFF"
  neutral: "#F5F4ED"
  surface: "#FDFCF9"
  surface-alt: "#EFEDE5"
  surface-selected: "#E4E1D6"
  border: "#E3E0D5"
  on-surface: "#1F1E1B"
  on-surface-muted: "#68645B"
  error: "#A8321F"
  status-attended: "#2F6B3F"
  status-attended-bg: "#E4EEE1"
  status-absent: "#A8321F"
  status-absent-bg: "#F8E4DE"
  status-cancelled: "#68645B"
  status-cancelled-bg: "#ECEAE2"
  status-pending: "#7F5610"
  status-pending-bg: "#F6ECD2"
  below-target: "#8A3A1A"
  below-target-bg: "#F6E3D6"
typography:
  title-lg:
    fontFamily: Georgia
    fontSize: 22px
    fontWeight: 400
    lineHeight: 1.25
  headline-md:
    fontFamily: Georgia
    fontSize: 16px
    fontWeight: 400
    lineHeight: 1.3
  body-md:
    fontFamily: system-ui
    fontSize: 13px
    fontWeight: 400
    lineHeight: 1.45
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
  sm: 6px
  md: 10px
  full: 9999px
spacing:
  xs: 4px
  sm: 8px
  md: 12px
  lg: 16px
  xl: 24px
  xxl: 32px
  nav-width: 208px
  row-height: 34px
  input-height: 30px
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
    backgroundColor: "{colors.surface-selected}"
    textColor: "{colors.on-surface}"
    typography: "{typography.label-md}"
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.on-primary}"
    typography: "{typography.label-md}"
    rounded: "{rounded.sm}"
    height: "{spacing.input-height}"
    padding: "{spacing.md}"
  button-primary-hover:
    backgroundColor: "{colors.primary-hover}"
    textColor: "{colors.on-primary}"
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
  section-meta:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.on-surface-muted}"
    typography: "{typography.body-sm}"
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

A warm, quiet, dense productivity tool for university students. It should feel like a good paper
notebook on a desk: cream pages, ink-dark text, a single terracotta accent, and serif headings that
give it a literary, calm character. Information density matters more than decoration. A student
opens it between classes, marks attendance in two clicks, checks what is due, and closes it.

- **Personality:** warm, calm, thoughtful, unobtrusive. No illustrations, gradients, or animation
  beyond basic hover feedback.
- **Density:** compact rows (34px), small body text (13px), tight spacing on a 4px grid.
- **Theme:** light only. A cream page with slightly lighter, near-white content cards.
- **Meaning is never carried by color alone.** Every status color is always paired with its text
  label ("Attended", "Absent", "Cancelled", "Pending", "Below target").

## Colors

Warm, paper-like neutrals with one terracotta accent. Status colors are earthy and muted and only
appear inside chips and banners, always next to a text label.

- **Primary (#B4532F) "Terracotta":** the only accent color. Used for the one primary button per
  panel and for focus rings. Hover deepens to #9C4526.
- **Neutral (#F5F4ED) "Cream":** the page background behind cards.
- **Surface (#FDFCF9) "Paper":** cards, list rows, inputs.
- **Surface Alt (#EFEDE5) "Oat":** the left nav background and list header rows.
- **Surface Selected (#E4E1D6):** the active nav item and selected rows. Selection is shown by a
  tonal fill, not by the accent color.
- **Border (#E3E0D5):** 1px dividers between rows and around inputs and cards.
- **On Surface (#1F1E1B) "Ink" / On Surface Muted (#68645B) "Graphite":** primary text and
  secondary text (metadata, times, rooms, empty states).
- **Error (#A8321F):** inline validation messages under text fields.
- **Attended (#2F6B3F on #E4EEE1):** sage-green status chip.
- **Absent (#A8321F on #F8E4DE):** brick-red status chip.
- **Cancelled (#68645B on #ECEAE2):** warm-grey status chip for cancelled and rescheduled sessions.
- **Pending (#7F5610 on #F6ECD2):** ochre status chip for sessions awaiting attendance.
- **Below target (#8A3A1A on #F6E3D6):** the dashboard banner and course badge when attendance is
  below the course's minimum.

All text/background pairs meet WCAG AA (4.5:1) for normal text.

## Typography

Two families: a serif for page and section headings, and the platform system font for everything
else so controls feel native on macOS, Windows, and Linux.

- **Title (Georgia, 22px / 400):** page titles ("Dashboard", "Timetable"). One per page.
- **Headline (Georgia, 16px / 400):** section headings inside a page ("Today's classes", "Overdue").
- **Body (system, 13px / 400):** list rows, form fields, banner text.
- **Body small (system, 12px / 400):** metadata lines, inline validation errors.
- **Label (system, 12px / 600):** button text, form field labels, active nav item.
- **Label small (system, 11px / 600):** status chip text.

Headings use regular weight; the serif carries the hierarchy on its own. Percentages and times are
plain text; the missing-value placeholder is an em dash "—". If Georgia is unavailable, headings
fall back to the platform default font.

## Layout

One window, default 1100 × 720. A fixed 208px left navigation column on the oat background with five
items (Dashboard, Timetable, Courses, Assessments, Settings), and a scrollable cream content area on
the right.

- A strict 4px spacing scale: 4, 8, 12, 16, 24, 32.
- Content area padding: 24px. Gap between sections: 24px. Gap inside a section: 8px.
- Lists are single-column rows, 34px tall, separated by 1px borders. No visual time grid; the
  timetable is a list grouped by weekday.
- Forms are inline panels or dialogs: label above field, 30px fields, error text directly below
  the field.
- Every list has an empty state: one muted sentence explaining what will appear and how to add it.

## Elevation & Depth

Flat. Hierarchy comes from tonal layers (cream page, paper cards, oat nav) and 1px warm borders, not
shadows. Dialogs are the only elevated surface and use a single soft shadow over a dimmed backdrop.

## Shapes

Soft, consistent radii: 6px for buttons, inputs, and nav items; 10px for cards and banners; fully
rounded pills for status chips.

## Components

- **Nav item:** 34px tall, 8px padding, body text on the oat background. The active item uses the
  surface-selected fill with label-weight text; the accent color is not used for navigation.
- **Primary button:** terracotta background, white label, 30px tall, 12px horizontal padding. At
  most one per panel (e.g. "Save").
- **Secondary button:** paper background, 1px border, ink label. Used for row actions such as
  Attended / Absent / Cancelled.
- **Input field:** paper background, 1px border, 30px tall; the border turns terracotta on focus.
  On invalid input the border and helper text switch to the error color, and the message states
  the expected format (`DD-MM-YYYY`, `HH:MM`). Dates shown as text read like "October 26, 2026".
- **List row:** 34px, paper background, 1px bottom border. Primary text left, muted metadata next
  to it, status chip and actions right-aligned.
- **Status chip:** pill with the status text label in label-small type on its tinted background.
  The text is always present.
- **Below-target banner:** tinted card at the top of the dashboard listing each course below its
  attendance target, e.g. "CSE101 — 66.7% (target 75%)".
- **Section card:** paper card, 10px radius, 1px border, 16px padding, with a serif headline title.
- **Empty state:** muted body text centered in the section, e.g. "No classes today."
- **Mini calendar:** a section card with the month name in the headline serif and prev/next
  secondary buttons; a 7-column grid of 36px day cells starting Monday, weekday initials in label-sm
  muted. Day numbers in body text; days outside the month muted. Today: a filled terracotta circle
  with white number. Selected day: surface-selected fill. Holidays and exam
  periods get a light terracotta highlight (primary at about 15% opacity); weekends get a fainter oat
  (surface-alt) highlight; a day with an assessment due shows a 4px error-red dot under the number.
  Class days carry no marker. A one-line legend under the grid names all three, so meaning never
  rests on color alone.

## Do's and Don'ts

- Do pair every status color with its text label; never show a colored dot or bar alone.
- Do reserve terracotta for one primary button per panel and focus rings.
- Do use the serif only for page titles and section headings; everything else uses the system font.
- Do keep rows at 34px and text at 13px so a full week fits on screen without scrolling.
- Do show "—" for a missing percentage, never "0%" or "NaN".
- Don't use pure white (#FFFFFF) as a background or pure black for text; stay on the warm neutrals.
- Don't hardcode colors or sizes in `.slint` files; reference `Theme.*` from `ui/theme.slint`.
- Don't add shadows, gradients, or decorative icons.
- Don't use more than two font weights on a single screen.
