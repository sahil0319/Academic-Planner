# Data Model: M17 Carbon Restyle

M17 changes no stored data: no schema change, no migration, no change to `planner-core` types,
`SCHEMA_VERSION`, backups, CSV, the import format, settings keys, or the Slint structs and
globals in `ui/types.slint` and the page `*State` globals that `src/pages/*.rs` binds.

The only "entities" are UI-side:

- **Theme token** (`Theme.*` in `crates/app/ui/theme.slint`): name, light value, dark value (for
  colours), status (kept, new, alias, removed). Complete list:
  [contracts/theme-tokens.md](contracts/theme-tokens.md). Invariants: every colour has both values;
  no other `.slint` file holds a raw hex or px value (§12.3 grep); after task 11 no `radius-sm`,
  `radius-md`, `calendar-holiday-highlight`, or unused `calendar-today` remains.
- **Component style**: the DESIGN.md v2 component each shared Slint component implements, with its
  unchanged properties and callbacks: [contracts/components.md](contracts/components.md).
- **Embedded font**: two TTF files (Plex Sans 400 and 600) plus `OFL.txt` under
  `crates/app/ui/fonts/`, registered by Slint font imports in `app-window.slint` (research D1).
- **Screenshot**: `specs/017-m17-carbon-restyle/screenshots/NN-<screen>-<light|dark>.png`, 24 files
  (research D12).
