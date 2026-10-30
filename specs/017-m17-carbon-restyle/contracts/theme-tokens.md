# Contract: `Theme` tokens after M17 (`crates/app/ui/theme.slint`)

Source: DESIGN.md v2 front matter and component prose. Pattern (research D3): every colour is
`out property <color> <name>: dark ? <dark value> : <light value>;`. `mode`, `dark`, and
`std-background` stay exactly as they are (System appearance detection is unchanged).

Status column: **kept** (name exists, value changes), **new**, **alias** (temporary, removed in
task 11), **removed** (in task 11).

## Colours

| Token | Light | Dark | Status |
|---|---|---|---|
| `primary` | #0F62FE | #0F62FE | kept |
| `primary-hover` | #0050E6 | #0050E6 | kept |
| `primary-active` | #002D9C | #002D9C | new |
| `on-primary` | #FFFFFF | #FFFFFF | kept |
| `neutral` | #FFFFFF | #161616 | kept |
| `surface` | #F4F4F4 | #262626 | kept |
| `surface-alt` | #FFFFFF | #161616 | kept |
| `surface-hover` | #E8E8E8 | #333333 | new |
| `surface-selected` | #E0E0E0 | #393939 | kept |
| `surface-inverse` | #161616 | #F4F4F4 | new |
| `on-surface-inverse` | #FFFFFF | #161616 | new |
| `field` | #FFFFFF | #393939 | new |
| `border` | #E0E0E0 | #393939 | kept |
| `border-strong` | #8D8D8D | #8D8D8D | new |
| `on-surface` | #161616 | #F4F4F4 | kept |
| `on-surface-muted` | #525252 | #C6C6C6 | kept |
| `on-surface-subtle` | #6F6F6F | #A8A8A8 | new |
| `link` | #0F62FE | #78A9FF | new |
| `focus` | #0F62FE | #FFFFFF | new |
| `error` | #DA1E28 | #FF8389 | kept |
| `status-attended` | #0E6027 | #6FDC8C | kept |
| `status-attended-bg` | #A7F0BA | #044317 | kept |
| `status-absent` | #A2191F | #FFB3B8 | kept |
| `status-absent-bg` | #FFD7D9 | #750E13 | kept |
| `status-cancelled` | #393939 | #E0E0E0 | kept |
| `status-cancelled-bg` | #E0E0E0 | #525252 | kept |
| `status-pending` | #8E6A00 | #F1C21B | kept |
| `status-pending-bg` | #FCF4D6 | #483700 | kept |
| `below-target` | #A2191F | #FFB3B8 | kept |
| `below-target-bg` | #FFF1F1 | #4A0C10 | kept |
| `progress-track` | #E0E0E0 | #393939 | new (used from M13) |
| `progress-fill` | #161616 | #F4F4F4 | new (used from M13) |
| `progress-target` | #0F62FE | #78A9FF | new (used from M13) |
| `calendar-holiday` | #D0E2FF | #002D9C | new |
| `calendar-midterm` | #D4BBFF | #6929C4 | new (used from M15) |
| `calendar-final` | #FFAFD2 | #9F1853 | new (used from M15) |
| `calendar-public-holiday` | #9EF0F0 | #005D5D | new (used from M16) |
| `success` | #24A148 | #24A148 | new (used from M18) |
| `success-bg` | #DEFBE6 | #071908 | new (used from M18) |
| `calendar-holiday-highlight` | — | — | removed (replaced by `calendar-holiday`, research D5) |

## Typography

`font-heading` and `font-body` are both `"IBM Plex Sans"` (R83). Line heights are comments.

| Role | `-size` | `-weight` | Line height (comment) | Status |
|---|---|---|---|---|
| `title-lg` | 24px | 400 | 32px | kept |
| `headline-md` | 16px | 600 | 22px | kept |
| `numeral-md` | 24px | 400 | 32px | new (used from M13) |
| `body-md` | 14px | 400 | 20px | kept |
| `body-emphasis` | 14px | 600 | 18px | new |
| `body-sm` | 12px | 400 | 16px | kept |
| `label-md` | 12px | 400 | 16px | kept |
| `label-sm` | 12px | 400 | 16px | kept |

## Radii

| Token | Value | Status |
|---|---|---|
| `radius-none` | 0px | kept |
| `radius-full` | 9999px | kept |
| `radius-sm` | `radius-none` | alias from task 2, removed in task 11 |
| `radius-md` | `radius-none` | alias from task 2, removed in task 11 |

## Spacing

`space-xs` 4px, `space-sm` 8px, `space-md` 12px, `space-lg` 16px, `space-xl` 24px, `space-xxl`
32px (kept), `space-xxxl` 48px (new).

## Sizes

| Token | Value | DESIGN.md source | Status |
|---|---|---|---|
| `nav-width` | 240px | spacing.nav-width | kept |
| `nav-item-height` | 32px | spacing.nav-item-height | new |
| `row-height` | 40px | spacing.row-height | kept |
| `input-height` | 32px | spacing.input-height | kept |
| `progress-height` | 8px | spacing.progress-height | new (used from M13) |
| `window-width` | 1100px | spacing.window-width | kept |
| `window-height` | 720px | spacing.window-height | kept |
| `divider-width` | 1px | components.divider | kept |
| `border-width` | 1px | card and control edges | kept |
| `control-edge-width` | 1px | components.control-edge | new |
| `focus-ring-width` | 2px | components.focus-ring | new |
| `focus-inner-width` | 1px | Components "Focus" (inner ring on filled buttons) | new |
| `rule-width` | 3px | components.success-rule; nav active bar, banner and toast left rule | new |
| `toast-width` | 320px | components.toast | new (used from M18) |
| `toast-close-size` | 24px | components.toast-close | new (used from M18) |
| `toast-timer-height` | 3px | components.toast-timer-* | new (used from M18) |
| `progress-target-width` | 2px | Components "Progress card" | new (used from M13) |
| `swatch-size` | 12px | Components "Mini calendar" legend, "Exam period cards" | new |
| `calendar-cell` | 36px | Components "Mini calendar" | kept |
| `calendar-marker` | 4px | Components "Mini calendar" (due square) | kept |
| `calendar-today` | — | v1 only | removed in task 11 if unused (research D5) |
| `appearance-segment` | 28px | Components "Appearance switch" | kept |
| `appearance-icon` | 16px | icon size inside a segment | kept |
| `appearance-icon-stroke` | 1.5px | icon stroke | kept |
| `link-icon` | 16px | Components "GitHub button" (16px mark) | kept |

Token names for sizes DESIGN.md states only in prose (`rule-width`, `focus-inner-width`,
`swatch-size`, `progress-target-width`, the `toast-*` sizes) are the builder's to keep exactly as
above; later milestones use these names.
