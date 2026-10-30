# Feature Specification: M17 Carbon Restyle

**Feature Branch**: `017-m17-carbon-restyle` (spec directory; work happens on the current branch)

**Created**: 2026-10-30

**Status**: Draft

**Input**: User description: "Milestone M17 from docs/spec.md §13.12: rules R82–R86, against
DESIGN.md v2. No T-tests are defined for M17. Do not add requirements."

**Source of truth**: `docs/spec.md` §7 (M17 row), §13.12 (R82–R86), §6 (visual rules, appearance
switch, existing pages), §12.0 and §12.3 (skills, theme grep check, DESIGN.md lint, screenshot
critique). `DESIGN.md` version 2 is the source of truth for every visual value and component. This
feature changes no rule, schema, core API, view-model text, or behaviour.

## Dependencies and order

- **Order (human decision 2026-10-30, §13.12)**: M17 is built first, before M18 and M12–M16, so
  every later page is built in the new style. M19 is independent.
- **What exists today** (the pages M17 restyles): Dashboard, Timetable (Week and Rules tabs),
  Courses (list, course detail, delete confirm), the Import panel on the Courses page,
  Assessments, Rules, Settings (including the restore confirm), About (including the update banner
  shown on every page), and the startup error screen.
- **Not in M17** (§13.12): the Progress page and its nav item (M13), exam period colours as their
  own kinds (M15; until then exam weeks are holidays and use the holiday fill), the public
  holidays card (M16), the term line, term card, and archive term (M14), the bi-weekly marker
  (M12), and toasts (M18). Their tokens are added now (R82) so those milestones add none; their
  components are built by their own milestones.

## User Scenarios & Testing *(mandatory)*

The user is a student who uses the app every day. They should see the same app, with the same
pages, buttons, and data, in the IBM Carbon look of DESIGN.md v2: square, dense, grey cards on a
white page, one blue accent, IBM Plex Sans, and a matching dark theme.

### User Story 1 - Carbon look on every existing page, light theme (Priority: P1)

The user opens the app in light appearance. Every page uses the DESIGN.md v2 colours, IBM Plex Sans
at 400 and 600, the v2 sizes (14px body, 40px rows, 32px controls), and square corners everywhere
except status chips.

**Why this priority**: This is the milestone: the visual layer changes and nothing else.

**Independent Test**: theme grep check, DESIGN.md lint, gate, and the light screenshot of every
page compared with DESIGN.md.

**Acceptance Scenarios**:

1. **Given** the app in Light appearance, **When** any page is shown, **Then** the page background
   is neutral `#FFFFFF`, section cards are surface `#F4F4F4`, text is on-surface `#161616` /
   on-surface-muted `#525252` / on-surface-subtle `#6F6F6F`, and blue `#0F62FE` appears only on
   the primary button, the active nav bar, links, focus rings, and today in the calendar (R82,
   DESIGN.md Colors).
2. **Given** any page, **When** text is shown, **Then** it is IBM Plex Sans at weight 400 or 600
   only, embedded in the app (no installed font needed) (R83).
3. **Given** any page, **When** buttons, fields, nav items, cards, banners, panels, calendar cells,
   and the appearance switch are shown, **Then** their corners are square, and so are status chips
   (R84, DESIGN.md Shapes; human decision 2026-10-31).
4. **Given** any list, **When** rows are shown, **Then** each row is 40px tall with a 1px border
   between rows, and every button and field is 32px tall (R84).
5. **Given** any page, **When** compared with the build before M17, **Then** the same controls,
   texts, and data are present and every action behaves the same (§13.12: behaviour and layout
   structure stay).

---

### User Story 2 - Dark theme in very dark navy (Priority: P1)

The user switches the appearance to Dark (or System on a dark OS). Every page uses the `dark-*`
values of DESIGN.md v2.

**Why this priority**: The appearance switch is an existing feature; every token must have its
dark value or dark mode breaks.

**Independent Test**: the dark screenshot of every page; `theme.slint` defines every colour as
`dark ? <dark-*> : <light>`.

**Acceptance Scenarios**:

1. **Given** Dark appearance, **When** any page is shown, **Then** the page is `#0B1220`, cards
   `#111A2E`, fields `#1A2542`, text `#E8EDF7` / `#A9B4CC` / `#8C98B3`, links `#78A9FF`, focus
   `#FFFFFF` (R82; navy values are a human decision of 2026-10-31, replacing the Carbon g100 greys).
2. **Given** System appearance, **When** the OS switches between light and dark, **Then** the app
   follows as it does today (§6 Appearance; detection logic unchanged).
3. **Given** Dark appearance, **When** status chips, the below-target banner, and calendar fills are
   shown, **Then** they use their `dark-*` pairs and stay readable (DESIGN.md Colors).

---

### User Story 3 - Carbon components (Priority: P1)

Buttons, fields, chips, cards, banners, and rows follow the DESIGN.md v2 component rules.

**Why this priority**: The components carry most of the look; pages are built from them.

**Independent Test**: screenshots of a form (Courses, Settings), a list with chips (Timetable,
Dashboard), the delete confirm, and the restore confirm, in both themes.

**Acceptance Scenarios**:

1. **Given** a text field, **When** it is idle, **Then** it has the field fill, no side or top
   border, and a 1px border-strong bottom rule; **When** focused, **Then** it shows a 2px inset
   focus ring; **When** invalid, **Then** a 2px inset error ring and the existing error text in
   body-small error colour below (R84, DESIGN.md Input field).
2. **Given** a primary button, **Then** it is primary-filled with a white label, hover
   primary-hover, press primary-active, disabled surface-selected with on-surface-subtle text.
   **Given** a secondary button, **Then** it is field-filled with a 1px border-strong edge (R84).
3. **Given** the Courses delete confirm or the Settings restore confirm, **When** shown, **Then**
   the destructive button (the Courses delete button shown when courses are selected and the
   delete button in its confirm panel; "Replace all data" in the restore confirm) uses the danger
   style: field fill, 1px error edge, error label, at most one per panel (R84).
4. **Given** the About page, **Then** the GitHub button has the surface-inverse fill, the
   on-surface-inverse label, and the 16px GitHub mark, square (R84).
5. **Given** the dashboard below-target banner, **Then** it has the below-target-bg fill, a 3px
   error-colour rule on its left edge, and the existing "Below target" label and lines (R84,
   DESIGN.md Below-target banner).
6. **Given** a status chip, **Then** it is square with 12px label text on its tinted fill, 8px
   horizontal padding, and its text label always present (DESIGN.md Status chip).
7. **Given** a section card, **Then** it is surface-filled, square, with no border, 16px padding,
   and a headline (16px / 600) title (DESIGN.md Section card).

---

### User Story 4 - Sidebar shell and appearance switch (Priority: P2)

The left sidebar keeps its place and pages, in the Carbon look.

**Why this priority**: Visible on every page, but it is one component.

**Independent Test**: one screenshot per theme of any page shows the shell.

**Acceptance Scenarios**:

1. **Given** the app window, **Then** the sidebar is 240px wide on the surface-alt colour with a
   1px border on its right edge, and the nav items read, top to bottom: Dashboard, Timetable,
   Courses, Assessments, Rules, Settings, About (R85).
2. **Given** a nav item, **Then** it is 32px tall with 16px left padding and on-surface-muted body
   text; on hover surface-hover fill and on-surface text; when active surface-selected fill,
   body-emphasis on-surface text, and a 3px primary bar on its left edge (R84, DESIGN.md Nav item).
3. **Given** the appearance switch, **Then** it is a square segmented control 16px from the
   top-right corner with three 28px icon segments and a 1px border-strong edge; the selected
   segment is surface-inverse with an on-surface-inverse icon; accessible labels are unchanged
   (R84, DESIGN.md Appearance switch).
4. **Given** the app starts, **Then** the default window size is still 1100 × 720 (R85).

---

### User Story 5 - Dashboard mini calendar (Priority: P2)

The mini calendar uses square cells and the calendar fills.

**Why this priority**: The calendar is the most colour-dependent element on the main page.

**Independent Test**: dashboard screenshots in both themes with a holiday range, a weekend, a due
assessment, today, and a different selected day visible.

**Acceptance Scenarios**:

1. **Given** the mini calendar, **Then** day cells are square 36px cells, Monday first, with
   weekday initials in label text (DESIGN.md Mini calendar).
2. **Given** today, **Then** it is a primary-filled square with a white number. **Given** the
   selected day, **Then** it has a 2px focus-colour inset outline that stays visible on any fill.
3. **Given** a holiday (including exam weeks entered as holidays, which stay holidays until M15),
   **Then** the cell uses the calendar-holiday fill; weekends use surface-hover; a day with an
   incomplete assessment due shows a 4px error-colour square under the number; day numbers stay
   on-surface on every fill.
4. **Given** the legend under the grid, **Then** it names every fill in use plus the due marker,
   each with a 12px swatch, and wraps when needed.

---

### User Story 6 - Critique and approval (Priority: P3)

The router critiques screenshots of every page in both themes with `impeccable`, high-severity
items are fixed, and the human approves the screenshots.

**Why this priority**: It closes the milestone (§7 Done-when) but adds no visual change of its own.

**Independent Test**: the screenshots exist in `specs/017-m17-carbon-restyle/screenshots/`, the
critique has no open high-severity item, and the human's approval is recorded.

**Acceptance Scenarios**:

1. **Given** the restyled app on a scratch database, **When** screenshots are taken, **Then** every
   page in the set of FR-016 exists in light and dark (R86).
2. **Given** the critique, **When** an item is high severity, **Then** it is fixed in
   `theme.slint` or the layout, never with a hardcoded value (§12.3), and the affected screenshots
   are retaken.

---

### Edge Cases

- **Glyphs missing from Plex** (for example non-Latin course names): fall back to the platform
  font (DESIGN.md Typography). The em dash "—" placeholder is in Plex.
- **Weights**: any text that asked for a weight other than 400 or 600 (for example the v1 label
  weight 600 or a heading at 400) takes the DESIGN.md v2 weight of its typography role; no other
  weight is embedded.
- **Today is also selected, or today is a holiday**: the primary fill wins over the day-off fill,
  and the selected outline stays visible on the primary fill.
- **A weekend holiday**: shows the holiday fill (as today, the holiday wins over the weekend).
- **Long banner or confirm text**: wraps; the 3px left rule runs the full height.
- **Disabled primary button**: surface-selected fill with on-surface-subtle text, in both themes.
- **Standard Slint widgets** (CheckBox, Switch, ComboBox, ScrollView, ProgressIndicator): keep the
  platform-neutral `fluent` widgets; they follow the light/dark scheme as today and are recoloured
  with theme tokens only as far as Slint allows (DESIGN.md Checkbox and switch).
- **Startup error screen** (database cannot be opened): uses the same tokens and the appearance
  switch.
- **Running from another directory or an installed bundle**: the fonts still load, because they
  are embedded at build time.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001** (R82): `crates/app/ui/theme.slint` MUST define every value of DESIGN.md v2: every
  colour with its light value and its `dark-*` value, the typography roles (size and weight;
  line heights as comments), `radius-none` and `radius-full`, the spacing scale including
  `space-xxxl`, and the sizes (`nav-width`, `nav-item-height`, `row-height`, `input-height`,
  `progress-height`, window size, and the component sizes DESIGN.md states). Token names follow
  DESIGN.md.
- **FR-002** (R82): The new tokens `surface-hover`, `surface-inverse`, `on-surface-inverse`,
  `field`, `border-strong`, `on-surface-subtle`, `link`, `focus`, `success`, `success-bg`,
  `progress-*`, `calendar-*`, `nav-item-height`, and `space-xxxl` MUST exist, including the ones no
  M17 page uses yet (`success*`, `progress-*`, `calendar-midterm`, `calendar-final`,
  `calendar-public-holiday`), so later milestones add no token.
- **FR-003** (R82): `radius-sm` and `radius-md` MUST be removed and every `.slint` file MUST use
  `radius-none` (status chips included; `radius-full` is kept but unused). During the milestone they MAY exist as
  temporary aliases of `radius-none` so every intermediate commit builds; they MUST be gone at the
  end.
- **FR-004** (R82, §6, §12.3): No `.slint` file other than `theme.slint` contains a raw hex colour
  or px size; the theme grep check passes after every task that touches `.slint` files.
- **FR-005** (R83): IBM Plex Sans Regular (400) and SemiBold (600) TTF files from IBM's official
  release MUST be vendored under `crates/app/ui/fonts/` and embedded with Slint font imports. No
  other weight or family is added.
- **FR-006** (R83): `font-heading` and `font-body` MUST both name "IBM Plex Sans".
- **FR-007** (R83): The SIL Open Font License 1.1 text MUST ship in `THIRD-PARTY-NOTICES.html`,
  and the release workflow's regenerated notices MUST keep it.
- **FR-008** (R84): Square corners everywhere except status chips; 32px buttons and fields; 40px
  rows.
- **FR-009** (R84): Carbon fields: field fill, 1px border-strong bottom rule, 2px inset focus ring,
  2px inset error ring when invalid, error text below.
- **FR-010** (R84): Nav items with a 3px primary bar on the active item, plus the DESIGN.md nav
  item fills and text styles.
- **FR-011** (R84): A danger button style MUST exist and MUST be used for the destructive confirms
  that exist today: the Courses delete button shown when courses are selected (label from
  `delete-label`) and the delete button of its confirm panel (today a primary button), and the
  Settings restore confirm button "Replace all data" (today a primary button). Archive term uses
  it when M14 builds it. Other delete actions without a confirm (an assessment row's "Delete", a
  rule set's "Delete") stay secondary buttons, as R84 and DESIGN.md list only the cases above.
- **FR-012** (R84): The About page GitHub button MUST use surface-inverse with on-surface-inverse
  text and mark.
- **FR-013** (R84): The appearance switch MUST be square, with the selected segment in
  surface-inverse.
- **FR-014** (R84): Calendar day cells MUST be square and use the calendar fills; banners MUST have
  a 3px left rule.
- **FR-015** (R85): Sidebar width 240px; nav order Dashboard, Timetable, Courses, Assessments,
  Rules, Settings, About; window default size unchanged (1100 × 720).
- **FR-016** (R86): Screenshots of every page in light and dark MUST be saved to
  `specs/017-m17-carbon-restyle/screenshots/`: Dashboard (with data), Dashboard (empty state, fresh
  database), Timetable Week tab, Timetable Rules tab, Courses list, course detail, Courses delete
  confirm, Import panel, Assessments, Rules, Settings (with the restore confirm visible), About.
- **FR-017** (R86, §7): `impeccable` critiques the screenshots against DESIGN.md; no high-severity
  item stays open; the human approves the screenshots.
- **FR-018** (§13.12 intro): Behaviour, layout structure, texts, and the sidebar stay. No Rust
  behaviour, view-model string, core, schema, or migration changes. All existing tests pass
  unchanged.
- **FR-019** (§7, §12.3): DESIGN.md lint (`npx -y @google/design.md@0.4.0 lint DESIGN.md`) and the
  gate pass at the end of the milestone. DESIGN.md itself is not edited.

### Key Entities

- **Theme token**: a named `Theme.*` property in `theme.slint` with a light value and, for colours,
  a dark value. See [contracts/theme-tokens.md](contracts/theme-tokens.md).
- **Component style**: the DESIGN.md v2 component a shared Slint component implements. See
  [contracts/components.md](contracts/components.md).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The theme grep check prints nothing, and `grep -rn "radius-sm\|radius-md"
  crates/app/ui` finds nothing.
- **SC-002**: DESIGN.md lint reports 0 errors.
- **SC-003**: The gate passes with the same test count as before M17 (no test removed).
- **SC-004**: 24 screenshots (12 screens × 2 themes) exist in the screenshots folder.
- **SC-005**: The `impeccable` critique lists zero open high-severity items.
- **SC-006**: The human records approval of the screenshots.

## Assumptions

- No T-tests are defined for M17 in `docs/spec.md`; it is a visual change verified by the gate,
  the grep check, the lint, the screenshots, the critique, and the human. The existing tests guard
  that behaviour did not change.
- Exam weeks are entered as holidays until M15, so the dashboard shows them with the holiday fill
  and the "Holiday" legend entry.
- The update banner (About, shown on every page) is a banner in the sense of R84 and gets the 3px
  left rule; DESIGN.md does not name its colours, so the plan picks neutral tokens (research D9)
  and the human sees it in the screenshots.
- The `impeccable` skill is allowed in M17 for the critique step only (R86, human decision).
- The screenshots are taken by the router (computer use or the OS screenshot tool) on a scratch
  database via `ACADEMIC_PLANNER_DB`, never the real user database.
