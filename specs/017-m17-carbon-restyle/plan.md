# Implementation Plan: M17 Carbon Restyle

**Branch**: `017-m17-carbon-restyle` (spec directory; work happens on the current branch) | **Date**: 2026-10-30 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `specs/017-m17-carbon-restyle/spec.md`

## Summary

A visual-layer swap from DESIGN.md v1 to v2 (IBM Carbon, light and very dark navy dark, IBM Plex Sans),
done in small commits that each build and pass the gate:

1. **Fonts (R83).** Vendor Plex Sans 400 and 600 TTF and the OFL under `crates/app/ui/fonts/`,
   register them with Slint font imports in `app-window.slint`, and add the OFL text to
   `about.hbs` and `THIRD-PARTY-NOTICES.html` (research D1, D2).
2. **Theme (R82).** Rewrite `theme.slint` with every DESIGN.md v2 token, light and dark
   ([contracts/theme-tokens.md](contracts/theme-tokens.md)); `radius-sm`/`radius-md` become
   temporary aliases of `radius-none` so nothing breaks (research D4).
3. **Components (R84).** Restyle the shared components in `components.slint` and add
   `DangerButton` ([contracts/components.md](contracts/components.md)).
4. **Shell (R84, R85).** Sidebar 240px, nav items with the 3px active bar, square appearance
   switch, default font family, startup error screen.
5. **Pages (R84).** One commit per page group: Dashboard; Timetable; Courses; Assessments; Rules +
   Import; Settings + About.
6. **Cleanup and checks (R82, R86).** Remove aliases and retired tokens, theme grep, DESIGN.md
   lint, gate, 24 screenshots.
7. **Critique and approval (R86, §7).** `impeccable` on the screenshots, fix high-severity items,
   human approval.

`docs/spec.md` §13.12 is the plan: no rule, schema, core API, or view-model change. Behaviour,
layout structure, texts, and the sidebar stay (§13.12 intro). The only interpretations are in
research.md; two of them are open questions below (defaults used until answered).

## Dependencies and start condition

- **Order**: M17 is built first, before M18 and M12–M16 (human decision 2026-10-30, §13.12). It
  starts from v0.2.1 (last green gate 1966d7c, 544 tests).
- **M18** later uses `success`, `success-bg`, `rule-width`, and the `toast-*` sizes added here; R88
  says M18 adds them only if it is built before M17.
- **M13, M15, M16** later use `progress-*`, `numeral-md`, `progress-height`,
  `progress-target-width`, `calendar-midterm`, `calendar-final`, `calendar-public-holiday`, and
  `swatch-size` added here. They add no token.
- **Exam periods** are holidays until M15, so the dashboard keeps one "Holiday / exam" fill
  (`calendar-holiday`).

## Technical Context

**Language/Version**: Rust stable (rustc 1.90), edition 2021; Slint 1.16 markup.

**Primary Dependencies**: Slint 1.16 (`fluent` style, unchanged build config), no new crate. New
assets: two IBM Plex Sans TTF files (OFL 1.1). `cargo-about` 0.9.3 regenerates the notices.

**Storage**: none touched (no schema, migration, or settings change).

**Testing**: `cargo test --workspace` (existing suite, unchanged); the §12.3 theme grep; DESIGN.md
lint (`npx -y @google/design.md@0.4.0 lint DESIGN.md`); screenshots + `impeccable` critique + human
approval. No new tests (research D13).

**Target Platform**: macOS, Windows, Linux desktop (unchanged).

**Project Type**: desktop app (Cargo workspace: `crates/core`, `crates/app`).

**Performance Goals**: no change; two embedded TTF files add a few hundred KB to the binary.

**Constraints**: `Theme.*` only outside `theme.slint`; `.slint` holds bindings only; no hex/px
outside `theme.slint`; every intermediate commit builds and passes the gate; weights 400 and 600
only; window default 1100 × 720.

**Scale/Scope**: 12 `.slint` files under `crates/app/ui/` (about 4,500 lines; `types.slint` unchanged), `about.hbs`,
`THIRD-PARTY-NOTICES.html`, three new asset files. No Rust source change is expected; if Slint's
font import needs one (for example `default-font-family` cannot be set in markup), it stays in
`main.rs` setup code and is noted in the task.

## Constitution Check

*GATE: checked before Phase 0 and again after Phase 1.*

| Principle | Status | Note |
|---|---|---|
| I. Spec is source of truth; `Theme.*` only; no added requirements | Pass | Every value from DESIGN.md v2; FR-001–FR-019 map to R82–R86, §7, §12.3; gaps (update banner colours, std widgets) are open questions, not decisions that change a rule |
| II. Test-first, traceable | Pass | No T-tests exist for M17 and no Rust behaviour changes (D13); comments in `.slint` cite R82–R86 where a component implements one |
| III. Pure, clock-free core; bindings-only `.slint` | Pass | Core untouched; no logic moves into `.slint`; no new view-model text |
| IV. Append-only migrations | Pass | No migration |
| V. Gate before done | Pass | Gate after every task; task 11 pastes the summary |
| Agent rule: design skills only in M0/M4 | Exception (justified) | R86 (human decision) allows `impeccable` in M17, critique step only (task 12). The planner loads no design skill |

Post-design re-check: unchanged (Pass, with the R86 exception).

## Project Structure

### Documentation (this feature)

```text
specs/017-m17-carbon-restyle/
├── spec.md
├── plan.md              # this file
├── research.md          # D1–D13
├── data-model.md        # no data change; UI-side entities
├── quickstart.md        # checks, scratch data, screenshot list
├── contracts/
│   ├── theme-tokens.md  # every Theme token, light/dark, status
│   └── components.md    # component → DESIGN.md style, task
├── checklists/
│   ├── ui.md            # requirements-quality checklist (speckit-checklist)
│   └── design-review.md # task 12: critique findings and human approval (created then)
├── screenshots/         # task 11: 24 PNG files
└── tasks.md
```

### Source Code (repository root)

```text
crates/app/ui/
├── fonts/                       # new: IBMPlexSans-Regular.ttf, IBMPlexSans-SemiBold.ttf, OFL.txt
├── theme.slint                  # task 2 (and 11)
├── components.slint             # tasks 3, 4
├── app-window.slint             # tasks 1 (font imports), 4
├── dashboard.slint              # task 5
├── timetable.slint              # task 6
├── courses.slint                # task 7
├── assessments.slint            # task 8
├── rules.slint, import.slint    # task 9
├── settings.slint, about.slint  # task 10
└── types.slint                  # unchanged
about.hbs, THIRD-PARTY-NOTICES.html   # task 1
```

**Structure Decision**: existing layout; only `crates/app/ui/fonts/` is new.

## Open Questions (for the router / human; defaults used until answered)

- **OQ-1 Update banner colours** (research D9). DESIGN.md names no colours for the update banner;
  R84 gives banners a 3px left rule. Options: (a) `surface` fill + `border-strong` rule
  (recommended: neutral, keeps blue reserved); (b) `surface` fill + `primary` rule (Carbon info,
  adds a use of blue); (c) no rule (conflicts with R84). Default (a). Visual only; the human sees
  it in screenshot 12 and on any page while an update is offered.
- **OQ-2 Standard widgets not square** (research D8). Slint 1.16 fluent `CheckBox` has 4px corners
  and `Switch` is a pill; Slint lets the app set only the colour scheme, not the accent or radius.
  Options: (a) keep the fluent widgets in M17 and let the task 12 critique rate them
  (recommended: matches the human's task list, no new components); (b) add custom square
  `Checkbox` and `Toggle` components to `components.slint` in task 3 (more work, keyboard and
  accessibility behaviour must be rebuilt). Default (a).

Naive choices decided here (one line each): screenshots named `NN-<screen>-<theme>.png` (D12);
today fills its whole calendar cell (D7); tabs and toggles show selection by fill + weight 600
without a blue bar (D11); assessment and rule set "Delete" stay secondary (D10); prose-only sizes
get the token names in contracts/theme-tokens.md.

## Complexity Tracking

| Exception | Why needed | Simpler alternative rejected because |
|---|---|---|
| `impeccable` design skill outside M0/M4 | R86 and §7 Done-when require a critique of every page in both themes | Human review alone was not what the human decided; the exception is limited to task 12 |
