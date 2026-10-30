# Contract: shared components after M17 (`crates/app/ui/components.slint` and page-local ones)

Each component keeps its name, properties, and callbacks (no API change for the pages or for
`src/pages/*.rs`). Only the visual layer changes. New: `DangerButton`. All values are `Theme.*`
tokens from [theme-tokens.md](theme-tokens.md).

| Component | File | Task | DESIGN.md v2 style |
|---|---|---|---|
| `PrimaryButton` | components.slint | 3 | `input-height`, `radius-none`, padding `space-lg`, `primary` fill, `on-primary` body-md label; hover `primary-hover`, press `primary-active`; disabled `surface-selected` fill, `on-surface-subtle` text; focus: 2px `focus` inset ring plus 1px `on-primary` inner ring |
| `SecondaryButton` | components.slint | 3 | `field` fill, 1px `border-strong` edge, `on-surface` label; hover `surface-hover`; focus 2px `focus` inset ring |
| `DangerButton` (new) | components.slint | 3 | same shape as SecondaryButton; `field` fill, 1px `error` edge, `error` label; hover `surface-hover`; same `clicked` / `text` / `enabled` interface as SecondaryButton |
| `Field`, `TimeField` text box | components.slint | 3 | `field` fill, no side or top border, 1px `border-strong` bottom rule, `input-height`, inline padding `space-lg`; focus 2px inset `focus` ring; invalid 2px inset `error` ring; label above in label-md `on-surface-muted`; error text below in body-sm `error` |
| `ComboField` | components.slint | 3 | label and error text as `Field`; the fluent `ComboBox` itself stays (research D8) |
| `AmPmToggle` / `AmPmSegment` | components.slint | 3 | square segments, 1px `border-strong` edge; selected `surface-selected` + weight 600 (research D11) |
| `StatusChip` | components.slint | 3 | `radius-full` pill, label-sm text, `space-sm` horizontal padding, `status-*` / `status-*-bg` pairs; text always present |
| `SectionCard` | components.slint | 3 | `surface` fill, `radius-none`, no border, padding `space-lg`, title in headline-md |
| `ListRow` | components.slint | 3 | min height `row-height` (40px), `surface` fill, 1px `border` bottom divider; vertical inset keeps a 32px button inside 40px |
| `EmptyState` | components.slint | 3 | body-md `on-surface-muted` inside the card, padding `space-xl` |
| `Banner` (below target) | components.slint | 3 | `below-target-bg` fill, `radius-none`, 3px (`rule-width`) `error` left rule, padding `space-lg`, "Below target" label plus body-md lines in `below-target`; wraps |
| `PageTitle` | components.slint | 3 | title-lg in `on-surface`, `font-heading` |
| `NavItem` | components.slint | 4 | `nav-item-height`, left padding `space-lg`, `radius-none`, body-md `on-surface-muted` on `surface-alt`; hover `surface-hover` + `on-surface`; active `surface-selected`, body-emphasis `on-surface`, 3px (`rule-width`) `primary` bar on the left edge |
| `AppearanceSwitch` / `AppearanceSegment` | components.slint | 4 | square, three `appearance-segment` (28px) segments, 1px `border-strong` edge; selected `surface-inverse` fill with `on-surface-inverse` icon; others `field` fill (hover `surface-hover`) with `on-surface` icon; accessible labels unchanged |
| App shell | app-window.slint | 4 | sidebar `nav-width` (240px) on `surface-alt` with a 1px `border` right edge; content `neutral` with padding `space-xl`; nav order Dashboard, Timetable, Courses, Assessments, Rules, Settings, About; `default-font-family: Theme.font-body`; startup error screen in the same tokens |
| `DayCell`, legend | dashboard.slint | 5 | research D7 |
| Week/Rules tabs, `SessionPanel` | timetable.slint | 6 | square tabs, selected `surface-selected` + weight 600 (D11); session panel as a section card |
| Delete confirm panel | courses.slint | 7 | square `surface` panel; delete buttons are `DangerButton` (D10) |
| `FilterTab` | assessments.slint | 8 | square, selected `surface-selected` + weight 600, hover `surface-hover`, focus 2px inset ring |
| Rule set list and form, `Divider` | rules.slint | 9 | rows and fields as above; `Divider` uses `border` and `divider-width` |
| `ImportPanel` paste box | import.slint | 9 | Carbon field style for the multi-line `TextInput` (field fill, bottom rule, inset focus ring) |
| Restore confirm panel | settings.slint | 10 | square `surface` panel; "Replace all data" is `DangerButton` (D10) |
| `GitHubButton` | about.slint | 10 | `surface-inverse` fill, `on-surface-inverse` label and `link-icon` (16px) mark, `radius-none`, `input-height`; focus 2px inset `focus` ring plus 1px inner ring |
| `UpdateBanner` | about.slint | 10 | `surface` fill, `radius-none`, 3px `border-strong` left rule (research D9, OQ-1), body-md text, View (primary) and Dismiss (secondary) |
| Text links on About | about.slint | 10 | `link` colour, body-md |

Page-local text components (`Caption`, `Body`, `Meta`, `FormError`, `RowText`, `RowMeta`,
`BodyText`, `MutedText`, `LegendText`, `ResultLine`) keep their roles and pick the v2 values from
the tokens; where one used a v1 weight or `font-heading` for body text, the page task switches it to
the DESIGN.md role (Typography section).
