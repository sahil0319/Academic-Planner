# Research: M17 Carbon Restyle

Every decision below stays inside `docs/spec.md` §13.12 and DESIGN.md v2. None changes a rule, the
schema, the core API, or a view-model text. Items D9 and D10 fill gaps DESIGN.md leaves open and
are listed as open questions in [plan.md](plan.md) with the default used until the human answers.

## D1. Font embedding

- **Decision**: Vendor `IBMPlexSans-Regular.ttf` and `IBMPlexSans-SemiBold.ttf` from the official
  IBM Plex release (github.com/IBM/plex, the `@ibm/plex-sans` package, `fonts/complete/ttf/`) into
  `crates/app/ui/fonts/`, together with the release's `LICENSE.txt` as `crates/app/ui/fonts/OFL.txt`.
  Register them with Slint font imports at the top of `crates/app/ui/app-window.slint`:
  `import "fonts/IBMPlexSans-Regular.ttf";` and `import "fonts/IBMPlexSans-SemiBold.ttf";`, and set
  `default-font-family: Theme.font-body;` on the window. Record the release tag and the SHA-256 of
  each file in a comment next to the imports.
- **Rationale**: Slint 1.16 registers `.ttf` imports as custom fonts (`custom_fonts` in the
  compiler's object tree), and `slint-build` embeds imported resources in the binary, so the
  fonts load from any working directory and in installed bundles (spec Edge Cases). Two files, as
  R83 says ("no other weights"). Slint matches weight 600 to the SemiBold file of the same family.
- **Alternatives considered**: loading fonts at runtime from a path (breaks in bundles); the
  variable font (one file, but R83 names the two static weights); Plex Sans from a CDN (no network
  at startup).
- **Check**: the builder runs the app from a different working directory and confirms Plex is
  used (the router confirms visually in task 11's screenshots).
- **Network**: if the builder cannot download the release, the router fetches the two files and
  the licence and hands them over; the task does not substitute another source.

## D2. OFL text in the notices

- **Decision**: Add a static "Fonts" section to `about.hbs` (after the intro paragraph, before the
  crate overview): "IBM Plex Sans (Regular, SemiBold), Copyright © 2017 IBM Corp. with Reserved
  Font Name "Plex", licensed under the SIL Open Font License 1.1", followed by the full OFL 1.1
  text in a `<pre>`. Then regenerate `THIRD-PARTY-NOTICES.html` locally with the pinned
  `cargo-about` 0.9.3 (`target/tools/bin/cargo-about`), using the same command as the release workflow
  (`cargo about generate --workspace --locked --fail -o THIRD-PARTY-NOTICES.html about.hbs`). If the
  tool is not available, edit `THIRD-PARTY-NOTICES.html` by hand so its new section is identical to
  the template's.
- **Rationale**: the release workflow regenerates the file from `about.hbs` on every tag
  (`.github/workflows/release.yml`), so a hand edit of the HTML alone would be lost. The copyright
  line is taken from the vendored `OFL.txt`, not typed from memory.
- **Alternatives considered**: a separate `FONTS-NOTICE` file (R83 names `THIRD-PARTY-NOTICES.html`).

## D3. Token names and the light/dark pattern

- **Decision**: Keep the M4 pattern: each colour is `out property <color> name: dark ? <dark-*> :
  <light>;`, names exactly as DESIGN.md (`colors.surface-hover` → `Theme.surface-hover`).
  Typography roles keep the `<role>-size` / `<role>-weight` pair with the line height as a comment
  (Slint `Text` has no line height). Spacing keeps `space-*`; sizes keep their DESIGN.md names.
  Component sizes that DESIGN.md states in the front matter or prose get a token now (R82: "every
  value"; §13.12: later milestones add none). The full list is
  [contracts/theme-tokens.md](contracts/theme-tokens.md).
- **Rationale**: unchanged mechanism (System mode detection is untouched, spec US2-2); names that
  match DESIGN.md make review a lookup.

## D4. Temporary radius aliases

- **Decision**: Task 2 defines `radius-none: 0px` and redefines `radius-sm` and `radius-md` as
  `radius-none` (comment: "temporary alias, removed in M17 task 11"). Every page is square from
  task 2 on. Tasks 3–10 replace the aliases in the files they touch; task 11 deletes them and any
  leftover use.
- **Rationale**: every intermediate commit builds and passes the gate (human decision); square
  from the start means the screenshots of each task already match DESIGN.md Shapes.

## D5. Retired v1-only tokens

- **Decision**: `calendar-holiday-highlight` (a v1 translucent terracotta) is replaced by
  `calendar-holiday` in task 5 and deleted in task 11. `calendar-today` (the v1 22px circle) is no
  longer needed because today fills its whole square cell (D7); task 5 stops using it and task 11
  deletes it if nothing references it. The label weights change from 600 to 400 (DESIGN.md v2
  `label-md`, `label-sm`).
- **Rationale**: R82 says the theme takes its values from DESIGN.md v2; these two have no v2
  source. Removing them in the cleanup task keeps intermediate commits building.

## D6. Carbon field and focus rings in Slint

- **Decision**: Implement the field as a `Rectangle` with the field fill, an inner bottom
  `Rectangle` of `control-edge-width` in `border-strong`, and an overlay `Rectangle` with
  `border-width: focus-ring-width`, `border-color: focus` (or `error` when invalid) shown on focus
  (or when invalid). Filled buttons draw the 2px focus ring plus a 1px `on-primary` inner ring.
- **Rationale**: a Slint `Rectangle` draws its border inside its bounds, which gives Carbon's inset
  ring without changing the 32px height.
- **Alternatives considered**: an outset ring (shifts layout, not Carbon).

## D7. Mini calendar

- **Decision**: Day cells are `calendar-cell` (36px) squares with `radius-none`. Fill priority:
  today (primary fill, `on-primary` number) > holiday (`calendar-holiday`) > weekend
  (`surface-hover`) > none. Selected day: an inset 2px `focus` outline; when the selected day is
  today, a 1px `on-primary` inner ring as for filled buttons, so the outline stays visible on blue.
  The due marker is a `calendar-marker` (4px) square in `error` under the number. The legend keeps
  its three static entries and texts ("Holiday / exam", "Weekend", "Assessment due"); their
  swatches become `swatch-size` (12px) squares in `calendar-holiday` and `surface-hover`, and the
  due entry shows the 4px `error` square. The legend wraps when needed.
- **Rationale**: DESIGN.md Mini calendar. Exam weeks are holidays until M15 (human decision), so
  they use the holiday fill. The midterm, final, and public-holiday fills are tokens only.
- **Note**: the legend texts are unchanged (no new string); M17 changes only swatches and fills.
  M15 renames "Holiday / exam" when exam periods become their own kinds.

## D8. Standard Slint widgets

- **Decision**: Keep `CheckBox`, `Switch`, `ComboBox`, `ScrollView`, and `ProgressIndicator` from
  the `fluent` style. In Slint 1.16 only `Palette.color-scheme` is writable; the fluent accent and
  radii are `out` properties, so they cannot take `Theme.*` values. They follow the light/dark
  scheme as today.
- **Rationale**: DESIGN.md says "the platform-neutral Slint widgets, recoloured with these tokens";
  Slint 1.16 allows only the scheme. Replacing them with custom widgets is not in the human's task
  list for M17 (task 3 lists buttons, fields, cards, chips, banner, rows).
- **Open question OQ-2** in plan.md: the fluent `CheckBox` (4px corners) and `Switch` (pill) are
  not square as R84 asks. Default: keep them and let the task 12 critique rate it.

## D9. Update banner colours

- **Decision (default, OQ-1)**: The update banner (About, shown above every page) becomes a square
  `surface` card with a 3px `border-strong` left rule, body text in on-surface, the "View" primary
  button and the "Dismiss" secondary button unchanged.
- **Rationale**: R84 gives banners a 3px left rule; DESIGN.md names only the below-target banner's
  colours. Blue is reserved (DESIGN.md Do's: primary button, nav bar, links, focus, today, target
  marker), so a blue Carbon "info" rule would add a use of blue. `border-strong` is the neutral
  control edge.
- **Alternatives considered**: a `primary` rule (Carbon info notification; conflicts with the blue
  reservation); no rule (conflicts with R84).

## D10. Danger buttons in place today

- **Decision**: Add `DangerButton` to `components.slint`. Use it for the Courses delete button
  (shown when courses are selected), the delete button of the Courses confirm panel (today a
  `PrimaryButton`), and the restore confirm "Replace all data" (today a `PrimaryButton`). Assessment
  row "Delete" and rule set "Delete" stay secondary.
- **Rationale**: R84 names course delete, restore, and archive term; DESIGN.md names "Delete
  selected" and the confirm of a destructive action. Archive term does not exist until M14. The
  ask button hides while the confirm shows, so there is never more than one danger button in a
  panel.

## D11. Selection and tabs

- **Decision**: Selected or active states that exist today (active nav item, the Assessments
  filter tabs, the Timetable Week/Rules tabs, the AM/PM toggle) use `surface-selected` plus weight
  600; the active nav item also has the 3px `primary` bar (R84). Tabs and toggle segments are
  square.
- **Rationale**: DESIGN.md Colors: "Selection is a tonal fill plus weight 600 and a 3px blue bar,
  not colour alone." The 3px bar is required for nav items (R84); for tabs the fill plus weight
  already avoids colour-only meaning, so no extra blue is added.

## D12. Screenshots

- **Decision**: 12 screens × 2 themes = 24 PNG files in
  `specs/017-m17-carbon-restyle/screenshots/`, named `NN-<screen>-<light|dark>.png` (NN = 01–12 in
  the FR-016 order). Taken at the default 1100 × 720 window on a scratch database
  (`ACADEMIC_PLANNER_DB="$PWD/target/scratch/m17.sqlite"`) filled through the Import panel with
  `docs/import/template.json` plus a few attendance marks, a holiday range, and assessments (see
  [quickstart.md](quickstart.md)). Screen 02 (empty state) uses a second fresh scratch file. The
  router takes them (computer use, or macOS `screencapture -l <window id>`), because the builder
  cannot see the window.
- **Rationale**: R86 and §7 ask for every page in both themes; the set covers each page and the
  two danger confirms, which carry the only new component.

## D13. No new tests

- **Decision**: M17 adds no Rust test. `docs/spec.md` defines no T-test for M17 and the feature
  changes no Rust code path. The existing suite (544 tests at v0.2.1) must pass unchanged at every
  task. The visual checks are the theme grep, the DESIGN.md lint, the screenshots, the critique,
  and the human approval.
- **Rationale**: constitution II asks for T-tests before rules; M17 has no rule with behaviour to
  test. Adding UI tests would be a new requirement.
