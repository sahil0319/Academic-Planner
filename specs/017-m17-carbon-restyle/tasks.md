---

description: "Task list for M17 Carbon restyle"
---

# Tasks: M17 Carbon Restyle

**Input**: Design documents from `specs/017-m17-carbon-restyle/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md), [data-model.md](data-model.md), [quickstart.md](quickstart.md), [contracts/theme-tokens.md](contracts/theme-tokens.md), [contracts/components.md](contracts/components.md), [checklists/ui.md](checklists/ui.md)

**Tests**: No new tests. `docs/spec.md` defines no T-test for M17 and no Rust behaviour changes
(research D13). The existing suite must pass unchanged after every task (same count as v0.2.1,
544 tests). Visual verification is the screenshots (T011), the critique, and the human (T012).

**Commits**: exactly one small commit per task, message prefix `M17:` (for example
`M17: theme.slint v2 tokens, light and dark`). The router commits. Task IDs T000–T012 are the
router's task numbers 0–12.

**Done-checks** used below:

- **Gate**: `cargo fmt --check && cargo clippy --workspace --all-targets -- -D warnings && cargo test --workspace`
  (paste the summary line).
- **Grep**: `! grep -rnE '#[0-9a-fA-F]{6}|[0-9]+px' crates/app/ui --include=*.slint | grep -v theme.slint`
  (prints nothing).
- **Same behaviour**: no file under `crates/core/`, `crates/app/src/` (except a font setup line if
  research D1 needs one), or `crates/app/ui/types.slint` changes; no `*State` global property or
  callback is renamed, added, or removed (contracts/components.md).

**Human visual check**: required only in T012 (approval of the 24 screenshots). Optional quick
looks: after T002 (fonts and colours swap everywhere at once) and after T004 (shell). T005–T010 are
checked through the T011 screenshots, so the builder needs no human between them.

**Organization**: Setup (fonts) → Foundational (theme, components) → shell (US4) → pages
(US5, US1–US3) → screenshots, critique, approval (US6).

## Format: `[ID] [P?] [Story] (owner) Description`

- **[P]**: can run in parallel (different files, no dependency on an incomplete task)
- **[Story]**: user story from spec.md (US1–US6)
- **(planner)** / **(builder)** / **(router)** / **(human)**: who does the task; **(tester)** has no
  task in M17 (no new tests)

---

## Phase 0: Planning

- [x] T000 (planner) Spec Kit design for M17 in specs/017-m17-carbon-restyle/: spec.md, plan.md,
  research.md, data-model.md, quickstart.md, contracts/theme-tokens.md, contracts/components.md,
  checklists/ui.md, tasks.md; analyze run with no blocking finding. No code. Done-check: the files
  exist; no file outside specs/017-m17-carbon-restyle/ changed (gate not needed). Commit
  `M17: Spec Kit plan, checklist, tasks`. Needs nothing.

---

## Phase 1: Setup (fonts, R83)

- [ ] T001 (builder) Vendor and embed IBM Plex Sans (research D1, D2): download the official IBM
  Plex release (github.com/IBM/plex, `@ibm/plex-sans`, `fonts/complete/ttf/`) and copy only
  `IBMPlexSans-Regular.ttf` and `IBMPlexSans-SemiBold.ttf` into crates/app/ui/fonts/, plus the
  release licence as crates/app/ui/fonts/OFL.txt; add `import "fonts/IBMPlexSans-Regular.ttf";`
  and `import "fonts/IBMPlexSans-SemiBold.ttf";` at the top of crates/app/ui/app-window.slint with
  a comment giving the release tag and each file's SHA-256 and citing R83; add a static "Fonts"
  section with the Plex copyright line (from OFL.txt) and the full SIL OFL 1.1 text to about.hbs,
  then regenerate THIRD-PARTY-NOTICES.html with `target/tools/bin/cargo-about generate --workspace
  --locked --fail -o THIRD-PARTY-NOTICES.html about.hbs` (if the tool fails, hand-edit the HTML so
  the section is identical to the template). `font-body` is not switched yet (T002). If there is
  no network, stop and ask the router for the three files; use no other source. Done-check: Gate,
  Grep, `ls crates/app/ui/fonts` shows exactly the three files, `grep -c "SIL Open Font License"
  about.hbs THIRD-PARTY-NOTICES.html` ≥ 1 each. Commit `M17: vendor IBM Plex Sans 400/600, OFL in
  notices`. Needs T000

---

## Phase 2: Foundational (blocking)

- [ ] T002 (builder) Rewrite crates/app/ui/theme.slint to contracts/theme-tokens.md (R82): every
  colour as `dark ? <dark-*> : <light>` with the exact DESIGN.md v2 values; `font-heading` and
  `font-body` = "IBM Plex Sans" (R83); typography sizes and weights incl. new `numeral-md`,
  `body-emphasis`, label weights 400, line heights as comments; `radius-none`, `radius-full`, and
  `radius-sm`/`radius-md` as temporary aliases of `radius-none` with the comment "temporary alias,
  removed in M17 T011" (research D4); spacing incl. `space-xxxl`; every size in the contract incl.
  `nav-item-height`, `control-edge-width`, `focus-ring-width`, `focus-inner-width`, `rule-width`,
  `swatch-size`, `progress-height`, `progress-target-width`, `toast-width`, `toast-close-size`,
  `toast-timer-height`; keep `calendar-holiday-highlight` and `calendar-today` for now (removed in
  T011); leave `mode`, `dark`, `std-background` unchanged; update the header comment to "DESIGN.md
  v2". In crates/app/ui/app-window.slint set `default-font-family: Theme.font-body;` on the window.
  Cross-check the colour values against DESIGN.md with a script (not committed). Done-check: Gate,
  Grep, Same behaviour. Optional human quick look (Light and Dark). Commit `M17: theme.slint v2
  tokens, light and dark`. Needs T001
- [ ] T003 [US3] (builder) Restyle the shared components in crates/app/ui/components.slint per
  contracts/components.md rows for task 3 (R84): `PrimaryButton` (states, disabled, focus with
  inner ring), `SecondaryButton` (field fill, border-strong edge), new `DangerButton` (same
  interface as `SecondaryButton`; field fill, 1px error edge, error label), `Field`/`TimeField`
  Carbon fields (field fill, 1px border-strong bottom rule, 2px inset focus ring, 2px inset error
  ring when invalid, error text below; research D6), `ComboField` label/error, square
  `AmPmToggle`/`AmPmSegment` (selected surface-selected + 600), `StatusChip` (pill, label-sm,
  space-sm padding), `SectionCard` (surface, square, no border, space-lg, headline title),
  `ListRow` (row-height 40px, border divider, inset for a 32px button), `EmptyState`, `Banner`
  (below-target-bg, 3px `rule-width` error left rule, space-lg padding, wraps), `PageTitle`
  (title-lg, comment no longer "Serif"). Replace `radius-sm`/`radius-md` with `radius-none` in this
  file except in `NavItem`/`AppearanceSwitch` (T004). Keep every property and callback name. Doc
  comments cite R84. Done-check: Gate, Grep, Same behaviour. Commit `M17: Carbon components incl.
  danger button`. Needs T002

---

## Phase 3: User Story 4 - Sidebar shell and appearance switch (P2, R84, R85)

**Goal**: the shell around every page in the Carbon look. **Independent test**: any page's
screenshot in both themes (T011 screens 01–12 all show it).

- [ ] T004 [US4] (builder) Restyle `NavItem` and `AppearanceSwitch`/`AppearanceSegment` in
  crates/app/ui/components.slint and the shell in crates/app/ui/app-window.slint per
  contracts/components.md (R84, R85): sidebar `nav-width` 240px on `surface-alt` with a 1px
  `border` right edge, nav padding/spacing on the 4px grid; `NavItem` `nav-item-height`, space-lg
  left padding, body-md on-surface-muted, hover surface-hover + on-surface, active surface-selected
  + body-emphasis + 3px `rule-width` `primary` bar on the left edge; nav order stays Dashboard,
  Timetable, Courses, Assessments, Rules, Settings, About (no Progress); square appearance switch
  16px (space-lg) from the top-right, three 28px segments, 1px border-strong edge, selected
  surface-inverse with on-surface-inverse icon, accessible labels unchanged; startup error screen
  in the same tokens; content area on `neutral` with space-xl padding; window default stays
  `window-width` × `window-height`. Done-check: Gate, Grep, Same behaviour. Optional human quick
  look. Commit `M17: app shell, sidebar 240px, square appearance switch`. Needs T003

---

## Phase 4: User Story 5 - Dashboard mini calendar (P2, R84)

**Goal**: dashboard in v2 incl. calendar cells and fills. **Independent test**: T011 screens 01,
02.

- [ ] T005 [US5] (builder) Restyle crates/app/ui/dashboard.slint (R84, research D7): `DayCell`
  square `calendar-cell` with `radius-none`; fill priority today (primary fill, on-primary number)
  > holiday (`calendar-holiday`, replaces `calendar-holiday-highlight`) > weekend (`surface-hover`)
  > none; selected = 2px `focus-ring-width` inset `focus` outline, plus a 1px `on-primary` inner
  ring when the selected day is today; due marker = `calendar-marker` square in `error` under the
  number; stop using `calendar-today`; weekday initials in label text; legend keeps its three
  texts ("Holiday / exam", "Weekend", "Assessment due") with `swatch-size` square swatches and
  wraps; below-target banner uses the T003 `Banner`; session and assessment rows on `ListRow` with
  `StatusChip`; section spacing space-xxl above cards, space-lg inside. Replace every
  `radius-sm`/`radius-md`. Done-check: Gate, Grep, Same behaviour. Commit `M17: dashboard
  restyle, square calendar cells and fills`. Needs T004

---

## Phase 5: User Stories 1–3 - Carbon look on every remaining page (P1, R82, R84)

**Goal**: each page in v2, light and dark, with the components of T003. **Independent test**: the
T011 screenshots listed per task.

- [ ] T006 [P] [US1] (builder) Restyle crates/app/ui/timetable.slint: Week/Rules tabs square with
  selected surface-selected + 600 (research D11), week rows on `ListRow` (40px) with
  `StatusChip`, `SessionPanel` and the reschedule/slot forms as square section cards with Carbon
  fields, page-local `ComboField`/text components on v2 roles; replace every `radius-sm`/
  `radius-md`. Done-check: Gate, Grep, Same behaviour; screens 03, 04. Commit `M17: timetable
  restyle`. Needs T004
- [ ] T007 [P] [US3] (builder) Restyle crates/app/ui/courses.slint: list rows, course detail
  figures and session list, makeup and course forms with Carbon fields; the delete button shown
  when courses are selected and the delete button in the confirm panel become `DangerButton`
  (today `SecondaryButton` and `PrimaryButton`; research D10); confirm panel square `surface`;
  replace every `radius-sm`/`radius-md`. Done-check: Gate, Grep, Same behaviour; screens 05, 06,
  07. Commit `M17: courses restyle, danger delete confirm`. Needs T004
- [ ] T008 [P] [US1] (builder) Restyle crates/app/ui/assessments.slint: `FilterTab` square,
  selected surface-selected + 600, hover surface-hover, 2px inset focus ring (replaces the
  primary border); rows on `ListRow`; form with Carbon fields; row "Delete" stays
  `SecondaryButton` (research D10); replace every `radius-sm`/`radius-md`. Done-check: Gate, Grep,
  Same behaviour; screen 09. Commit `M17: assessments restyle`. Needs T004
- [ ] T009 [P] [US1] (builder) Restyle crates/app/ui/rules.slint and crates/app/ui/import.slint:
  rule set list rows and form (bands, components, grade scale) with Carbon fields, `Divider` on
  `border`/`divider-width`, rule set "Delete" stays secondary; Import panel paste box as a Carbon
  field (field fill, bottom rule, 2px inset focus ring instead of the primary border), result lines
  on v2 roles; replace every `radius-sm`/`radius-md`. Done-check: Gate, Grep, Same behaviour;
  screens 08, 10. Commit `M17: rules and import restyle`. Needs T004
- [ ] T010 [P] [US3] (builder) Restyle crates/app/ui/settings.slint and crates/app/ui/about.slint:
  Settings cards and forms with Carbon fields; restore confirm "Replace all data" becomes
  `DangerButton` (research D10), confirm panel square `surface`; About `GitHubButton` with
  `surface-inverse` fill, `on-surface-inverse` label and `link-icon` mark, square, 2px inset focus
  ring; text links in `link`; `UpdateBanner` square `surface` with a 3px `rule-width`
  `border-strong` left rule (research D9, default of OQ-1); replace every `radius-sm`/`radius-md`.
  Done-check: Gate, Grep, Same behaviour; screens 11, 12. Commit `M17: settings and about
  restyle, inverse GitHub button, danger restore`. Needs T004

---

## Phase 6: User Story 6 - Cleanup, screenshots, critique, approval (P3, R82, R86, §7)

- [ ] T011 [US6] (builder, then router) Builder: delete `radius-sm`, `radius-md`,
  `calendar-holiday-highlight`, and (if unreferenced) `calendar-today` from
  crates/app/ui/theme.slint and fix any leftover use in crates/app/ui/*.slint; run
  `! grep -rnE 'radius-sm|radius-md|calendar-holiday-highlight' crates/app/ui`; run
  `npx -y @google/design.md@0.4.0 lint DESIGN.md` (0 errors; DESIGN.md is not edited); run the
  Gate and Grep and paste the summaries. Router: build the scratch databases and take the 24
  screenshots of quickstart.md §3–§4 (12 screens × Light and Dark, 1100 × 720) into
  specs/017-m17-carbon-restyle/screenshots/ as `NN-<screen>-<light|dark>.png`, never on the real
  database. Done-check: Gate, Grep, alias grep empty, lint 0 errors, 24 files present, Same
  behaviour. Commit `M17: remove radius aliases, lint, light and dark screenshots`. Needs T005,
  T006, T007, T008, T009, T010
- [ ] T012 [US6] (router, builder, human) Router: run `impeccable` (allowed in M17 by R86, critique
  only) on the 24 screenshots against DESIGN.md and write the findings with severity into
  specs/017-m17-carbon-restyle/checklists/design-review.md; builder: fix every high-severity item
  in crates/app/ui/theme.slint or layout only, never with a hardcoded value (§12.3); router:
  retake the affected screenshots; human: look at the screenshots and approve (also answer OQ-1
  update banner colours and OQ-2 fluent CheckBox/Switch corners, plan.md); router records the
  approval and the answers in design-review.md. If the human picks a non-default option for OQ-1 or
  OQ-2, the builder applies it in this task. Done-check: Gate, Grep, no open high-severity item,
  approval recorded. **Human visual check required.** Commit `M17: critique fixes, screenshots
  approved`. Needs T011

---

## Dependencies & Execution Order

- T000 → T001 → T002 → T003 → T004 (strict chain: T002 needs the font files of T001; T003 needs
  the tokens; T004 edits components.slint after T003).
- T005–T010 each need T004. T005 is listed first only because the dashboard is the main page;
  T006–T010 touch different files and are marked [P]. One builder runs them in order T005 → T010;
  none edits components.slint (if a page finds a missing shared variant, it stops and the router
  decides, so two tasks never edit the same file).
- T011 needs T005–T010. T012 needs T011.

## Parallel Examples

- After T004: T006 (timetable.slint), T007 (courses.slint), T008 (assessments.slint), T009
  (rules.slint, import.slint), T010 (settings.slint, about.slint) touch disjoint files. With one
  builder they run in sequence; with two, split T005–T007 / T008–T010.

## Implementation Strategy

1. Foundation: T001–T004. After T004 the whole app already has v2 colours, Plex, square corners,
   and Carbon components; pages only refine.
2. Pages: T005–T010, each a small commit with a green gate.
3. Close: T011 (cleanup, lint, screenshots), T012 (critique, fixes, human approval) = §7 Done-when.

## Task counts

| Owner | Tasks | Total |
|---|---|---|
| planner | T000 | 1 |
| builder | T001–T010, T011 (cleanup part), T012 (fixes) | 10 + 2 shared |
| router | T011 (screenshots), T012 (critique, records) | 2 shared |
| human | T012 (approval, OQ-1, OQ-2) | 1 shared |
| tester | none (no new tests, research D13) | 0 |
| **Total** | T000–T012 | **13** |
