# Feature Specification: M7 About Page, Updater, Pre-Migration Backup

**Feature Branch**: `007-m7-updater` (spec directory; work happens on the current branch)

**Created**: 2026-10-29

**Status**: Draft

**Input**: User description: "Milestone M7 from docs/spec.md §13.1: rules R26–R30, tests T31–T33.
Do not add requirements."

**Source of truth**: `docs/spec.md` §1 (rows "Network" and "Updates (M7)"), §3 (`setting` default
`check_updates_on_start=1`), §7 (M7 row), §13.1 (R26–R30, the release-workflow note, the About
page), and §13.5 (T31–T33). Everything in §1–§12 still applies. This feature adds no rule, no
schema change, and no §5 method; it implements the §13.1 text as written.

## Human Decisions (2026-10-29, recorded; do not re-ask)

- **HD-1 Signing**: updates are minisign-signed with `cargo-packager`'s signer and verified by
  `cargo-packager-updater`. The public key is embedded in the app at build time. The private key
  and its password live only in two GitHub repository secrets that the human creates. The agent
  never generates, sees, or stores the private key; it tells the human the exact command and the
  secret names.
- **HD-2 When to check**: once on startup (setting `check_updates_on_start`, default on) and on
  demand with a "Check for updates" button on a new About page. Nothing is ever installed without
  the user's click.
- **HD-3 What is offered**: only published GitHub releases (not drafts, not pre-releases). The
  release workflow keeps `draft: true`; the human publishes each release by hand.
- **HD-4 Per-format install**: macOS `.app`, Windows NSIS, and Linux AppImage install themselves.
  A Linux `.deb` install does not self-install; its button opens the release page.
- **HD-5 Pre-migration backup** per R26: core reads no clock, so duplicate names get `-2`, `-3`,
  … suffixes. Only one real migration exists today, so T31/T32 run against a test-only migration
  list; the plan decides how core exposes that list without widening the public API more than
  needed.

## User Scenarios & Testing *(mandatory)*

The user is a student with an installed copy of Academic Planner (v0.1.1 or later) and a database
full of data they cannot afford to lose. They want new versions to reach them with one click and
want every schema upgrade to leave a copy of their old database behind.

### User Story 1 - Database upgrades leave a backup and never half-apply (Priority: P1)

When a newer app version opens an older database, it first saves a full copy of the old database
in the `backups` folder next to it, then runs the pending migrations, each one all-or-nothing.

**Why this priority**: Updates (Stories 2–3) deliver new schemas to real databases. Without this
safety net, a failed upgrade could cost the user all their data. It must ship before or with the
updater.

**Independent Test**: T31 and T32 (core tests with a test-only list of 2 migrations against a
database file in a temporary directory).

**Acceptance Scenarios**:

1. **Given** a database at schema v1 and an app whose migration list has 2 entries, **When** the
   database is opened, **Then** `{db_dir}/backups/pre-migrate-v1-to-v2.sqlite` exists, its
   contents equal the database as it was before the open, and the database is now at v2 (T31).
2. **Given** that upgraded database, **When** it is opened again, **Then** no new backup file is
   written (T31).
3. **Given** a fresh database (`user_version` 0), **When** it is opened and migrated, **Then** no
   backup is written (T31).
4. **Given** `pre-migrate-v1-to-v2.sqlite` already exists in `backups`, **When** another v1
   database in the same folder is upgraded, **Then** the copy is written as
   `pre-migrate-v1-to-v2-2.sqlite` (then `-3`, …); no existing file is overwritten (R26).
5. **Given** a v1 database and a migration list whose second migration fails, **When** the
   database is opened, **Then** the open fails, the database is still at v1 with unchanged
   contents, and the error message contains the backup file path (T32).
6. **Given** a database newer than the app, **When** it is opened, **Then** it is refused as
   before (R25), no backup is written, and nothing is downgraded.

---

### User Story 2 - Find out that an update exists (Priority: P1)

On launch (if the startup check is on) the app quietly checks GitHub for a newer published release
and shows a dismissible banner when one exists. On the About page the user can check at any time.

**Why this priority**: Every later milestone (M8–M10) reaches installed copies through this path.

**Independent Test**: T33 (app unit test of the version decision) plus the About page states,
which are formatted by unit-tested view-model functions.

**Acceptance Scenarios**:

1. **Given** the app runs 0.1.9, **When** the manifest says 0.1.10, **Then** the update is
   offered; 0.1.9 is not offered; 0.2.0-beta is not offered (T33).
2. **Given** `check_updates_on_start` is on, **When** the app starts, **Then** one background
   check runs; the window opens and works without waiting for it (R29).
3. **Given** the startup check finds an update, **Then** a dismissible banner says an update is
   available; dismissing it hides it for the rest of this launch (R29).
4. **Given** `check_updates_on_start` is off, **When** the app starts, **Then** no check runs
   and no network request is made (R29).
5. **Given** the user presses "Check for updates", **Then** the button area shows "checking",
   then one of: "up to date", "update available" with an Install button, or "Couldn't check for
   updates" with a Retry button (R27, §13.1 UI).
6. **Given** no network, an HTTP error, or a manifest that does not parse, **Then** the result is
   "Couldn't check for updates" with Retry, and the rest of the app keeps working (R27).
7. **Given** a release is still a draft or is marked pre-release on GitHub, **Then** installed apps
   are never offered it (R27, HD-3).

---

### User Story 3 - Install an update with one click (Priority: P1)

After the user chooses Install, a dialog shows the new version and its release notes. On confirm,
the app downloads the update with visible progress, verifies its signature, installs it, and
relaunches. A `.deb` install instead opens the release page.

**Why this priority**: This is the point of the updater; M7 is done only when a published test
release updates an installed copy on macOS (§7).

**Independent Test**: Manual: publish a test release and update an installed macOS copy (§7 Done
when). The signature check and per-format branching are exercised by that run; the version
decision is covered by T33.

**Acceptance Scenarios**:

1. **Given** an update is available, **When** the user presses Install, **Then** a confirm dialog
   shows the new version and its release notes; Cancel changes nothing (R28).
2. **Given** the user confirms, **Then** the About page shows "downloading" with progress; when the
   download is complete and its minisign signature matches the embedded public key, the app
   installs the update and relaunches into the new version (R28).
3. **Given** a download whose signature is bad or missing, **Then** installation stops, the
   installed app is unchanged, and the user sees an error with Retry (R28).
4. **Given** the app runs as a Linux `.deb` install (not from an AppImage), **When** an update is
   available, **Then** the dialog's button opens the release page in the browser instead of
   installing (R28, HD-4).
5. **Given** the new version starts after installing, **Then** its first database open applies
   R25/R26 (backup then migrate); the updater itself never reads or writes user data (R30).

---

### User Story 4 - About page (Priority: P2)

A sixth navigation page, About, shows who made the app and where things are, and holds the update
controls.

**Why this priority**: It is the home of the update button and switch; its static content is
small.

**Independent Test**: Run the app, open About, and check each item; view-model formatting is unit
tested.

**Acceptance Scenarios**:

1. **Given** the About page, **Then** it shows the app name, the running version, a license line,
   a link to the GitHub repository, a link to `THIRD-PARTY-NOTICES.html`, the database file
   location, the "Check for updates" button with its states, and the "Check for updates on
   startup" switch (§13.1 UI).
2. **Given** the switch is turned off, **Then** `check_updates_on_start` is saved as off at once
   and the next launch makes no check (R29, §1 Saving).
3. **Given** a JSON backup, **Then** it includes `check_updates_on_start`; a backup without that
   key restores as on (§13.1 UI).

---

### User Story 5 - Release builds publish signed updates (Priority: P2)

A tag push builds, as before, the installers, and now also the updater artifacts (macOS
`.app.tar.gz`, Windows setup `.exe`, `.AppImage`), their minisign signatures, and the updater
manifest. The release stays a draft until the human publishes it.

**Why this priority**: Installed apps can only update to releases built this way; it is needed
for the §7 Done-when check.

**Independent Test**: Push a test tag; the draft release lists the installers, the three updater
artifacts with `.sig` files, and the manifest; after the human publishes, an installed copy is
offered the update.

**Acceptance Scenarios**:

1. **Given** the two signing secrets exist, **When** a `v*` tag is pushed, **Then** the draft
   release contains the updater artifacts, their signatures, and the manifest, whose download URLs
   point at that release's assets.
2. **Given** a signing secret is missing, **Then** the release workflow fails with a clear error
   before creating the draft (no unsigned update is ever published).
3. **Given** the draft is not yet published, **Then** installed apps see no update (HD-3).

---

### Edge Cases

- A database file opened through `open_in_memory` is always fresh, so it never gets a backup.
- The `backups` folder does not exist yet: it is created before the copy is written.
- The backup cannot be written (disk full, no permission): the open fails with an error and no
  migration runs, so the database stays at its old version.
- The first of two pending migrations succeeds and the second fails: the database stays at the
  version the first produced (the last good version), and the error names the backup path (R26).
- The manifest version equals the running version, or is lower: not offered.
- The manifest version is a semver pre-release (e.g. `0.2.0-beta`) even if its number is higher:
  not offered (T33).
- The manifest has no entry for this platform (e.g. Intel macOS): treated as a check failure,
  "Couldn't check for updates".
- The user presses "Check for updates" while a check or download is already running: the button
  is disabled during "checking" and "downloading".
- The user closes the app during a download: nothing is installed; the next check starts over.
- The startup check fails silently: no banner, no error dialog; the About page shows the failure
  only if the user opens it (R27: never blocks the app).
- A development build run with `cargo run` is not a packaged install; Install opens the release
  page, like a `.deb`, so a dev build never overwrites its own build folder.

## Requirements *(mandatory)*

### Functional Requirements

**Migrations (core)**

- **FR-001** (R26): When `open` finds an existing database with `user_version` ≥ 1 and below
  `SCHEMA_VERSION`, it MUST write a consistent copy of it to
  `{db_dir}/backups/pre-migrate-v{from}-to-v{to}.sqlite` before any migration runs, where `from`
  is the found version and `to` is `SCHEMA_VERSION`.
- **FR-002** (R26): If that file name exists, the copy MUST use the first free name among
  `pre-migrate-v{from}-to-v{to}-2.sqlite`, `-3`, …; no existing file is overwritten. Core MUST
  NOT read the clock to name the file.
- **FR-003** (R26): A fresh database (`user_version` 0) and an already-current database MUST NOT
  get a copy.
- **FR-004** (R26): Each migration MUST run in one transaction together with its `user_version`
  bump. If a migration fails, the database MUST stay at the last good version and the returned
  error MUST name the backup path.
- **FR-005** (R25, R26): A database newer than the app MUST still be refused before anything is
  written, with no backup and no downgrade.
- **FR-006** (HD-5): T31 and T32 MUST be able to run `open` against a test-only migration list of
  2 entries without that list or its entry point appearing in the public core API used by the app.

**Updates (app crate only; §1 Network)**

- **FR-007** (§1, R27): The only network access in the app MUST be the updater reading GitHub
  Releases of `sahil0319/Academic-Planner`. Core MUST contain no network code.
- **FR-008** (R27, HD-3): The update source MUST be the updater manifest attached to the latest
  published GitHub release; drafts and pre-releases MUST never be offered.
- **FR-009** (R27, T33): An update MUST be offered only if its version is greater than the running
  version by semver and is not a semver pre-release (0.1.10 > 0.1.9 is offered; 0.1.9 and
  0.2.0-beta are not).
- **FR-010** (R27): Any network or parse failure MUST show "Couldn't check for updates" with a
  Retry and MUST NOT block or freeze the app.
- **FR-011** (R28): Installing MUST first show a confirm dialog with the new version and its
  release notes; nothing happens without the user's confirmation.
- **FR-012** (R28, HD-1): The download MUST be verified against the minisign public key embedded
  at build time; a bad or missing signature MUST abort with the installed app unchanged.
- **FR-013** (R28, HD-4): After verification, macOS MUST replace the `.app` bundle, Windows MUST
  run the NSIS installer silently, and a Linux AppImage MUST replace itself; then the app
  relaunches. A Linux install not running from an AppImage (`.deb`) MUST NOT self-install; the
  dialog's button opens the release page instead.
- **FR-014** (R29): If `check_updates_on_start` is on, the app MUST check once per launch in the
  background and show a dismissible banner when an update exists. It MUST NEVER install without
  the user's click.
- **FR-015** (R30): The updater MUST NOT read, write, move, or delete user data. The new version's
  first `open` applies R25/R26.

**Setting and About page**

- **FR-016** (§3, §13.1 UI): The `check_updates_on_start` setting MUST default to on (a missing row
  reads as on), MUST be part of `Settings`, MUST be included in JSON backups, and a backup without
  it MUST restore as on. No new migration is added (it is a `setting` row).
- **FR-017** (§13.1 UI, §6): A sixth navigation page, About, MUST show the app name, version,
  license line, links to the GitHub repository and `THIRD-PARTY-NOTICES.html`, the database file
  location, the "Check for updates" button with its states (checking / up to date / update
  available → Install / downloading with progress / error + Retry), and the "Check for updates on
  startup" switch.
- **FR-018** (§6, constitution III): About page text and states MUST be produced by view-model
  functions with unit tests; `.slint` files contain only bindings and use `Theme.*` only.

**Release workflow**

- **FR-019** (§13.1 release workflow, HD-1): The tag build MUST also produce the macOS
  `.app.tar.gz`, the Windows setup `.exe`, and the `.AppImage` as updater artifacts, their
  minisign signatures, and the updater manifest, signing with the private key and password from
  two GitHub secrets.
- **FR-020** (HD-3): The release MUST stay a draft (`draft: true`) until the human publishes it.
- **FR-021** (HD-1): The human MUST be told the exact key-generation command and the two secret
  names; the private key never enters the repository or the agent's context.

### Key Entities

- **Pre-migration backup**: a SQLite file in `{db_dir}/backups/`, named
  `pre-migrate-v{from}-to-v{to}[-N].sqlite`, equal to the database before migration.
- **Setting `check_updates_on_start`**: a `setting` row, `1` (default) or `0`; a field of
  `Settings`.
- **Updater manifest**: a JSON file attached to each release: version, release notes, and per
  platform (macOS, Windows, Linux AppImage) the download URL, signature, and install format.
- **Update state** (app only, not stored): idle, checking, up to date, update available (version,
  notes), downloading (progress), error; plus whether the startup banner was dismissed.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: The gate passes and T31, T32, T33 pass (§7).
- **SC-002**: A published test release updates an installed copy on macOS: the user clicks
  Install, confirms, and the app relaunches as the new version with all data present (§7).
- **SC-003**: Upgrading a database with pending migrations always leaves exactly one new backup
  copy equal to the old database; opening an up-to-date or fresh database never creates one.
- **SC-004**: With no network, the app opens and every page works; the About page shows
  "Couldn't check for updates" with Retry.
- **SC-005**: No unsigned or wrongly signed download is ever installed.
- **SC-006**: A draft or pre-release on GitHub is never offered to an installed copy.

## Assumptions

- No schema change in M7: `SCHEMA_VERSION` stays 1; the test-only migration list exists only for
  T31/T32.
- T31/T32 need a database file because the backup is a file next to the database; they use a
  temporary directory, never the real user database (constitution II intent).
- The update check reads the manifest from GitHub's "latest release" download URL, which GitHub
  resolves only to the newest published, non-draft, non-pre-release release (HD-3).
- The startup banner's dismissal lasts for the current launch only; it is not stored.
- Release notes shown in the confirm dialog are the manifest's notes, taken from
  `docs/releases/<tag>.md`.
- The `THIRD-PARTY-NOTICES.html` link opens the copy attached to the running version's GitHub
  release (README: it is attached to each release).
- macOS builds are Apple Silicon only, as in M6; the manifest lists the platforms the release
  workflow builds.
- Users of v0.1.0 and v0.1.1 have no updater; they install the first M7 release by hand, after
  which updates arrive in-app.
