# Contract: third-party notices (FR-004)

Tool: `cargo-about` 0.9.3 (research R-2).

## `about.toml` (repo root)

```toml
# Licenses accepted for dependencies, in priority order (cargo-about picks the first one that
# satisfies each crate's SPDX expression). Slint resolves to GPL-3.0-only (see NOTICE).
accepted = [
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "ISC",
    "Zlib",
    "BSL-1.0",
    "Unicode-3.0",
    "MPL-2.0",
    "NCSA",
    "GPL-3.0-only",
    "GPL-3.0-or-later",
]
ignore-dev-dependencies = true
```

If a later `Cargo.lock` adds a license outside this list, generation fails and names the crate. Add a
license to the list only if it is GPL-3.0-compatible; otherwise stop and ask the human.

## `about.hbs` (repo root)

Minimal HTML, no inline styles. Required content:

- `<title>` and `<h1>`: "Academic Planner: third-party notices".
- One intro sentence: these are the licenses of the Rust crates compiled into Academic Planner; the
  app itself is GPL-3.0-or-later (see LICENSE and NOTICE).
- Overview list: `{{#each overview}}` → link to `#{{id}}`, `{{name}}`, `({{count}})`.
- Body: `{{#each licenses}}` → heading with `id="{{id}}"` on `first_of_kind`, the license name, a
  "Used by" list of `{{crate.name}} {{crate.version}}`, and `<pre>{{text}}</pre>`.

## Command

```sh
cargo about generate --workspace --locked --fail -o THIRD-PARTY-NOTICES.html about.hbs
```

Run from the repo root. Output is ignored by git (`/THIRD-PARTY-NOTICES.html` in `.gitignore`).

## `NOTICE` change (US3 scenario 3)

Replace the last paragraph ("Third-party dependency notices are generated at release time
(milestone M6).") with:

> Notices for the third-party Rust crates compiled into this program are in
> THIRD-PARTY-NOTICES.html. It is generated from Cargo.lock by cargo-about (configuration:
> about.toml, about.hbs), is bundled in every installer, and is attached to every GitHub release.
> To generate it locally: cargo about generate --workspace --locked -o THIRD-PARTY-NOTICES.html about.hbs

The rest of `NOTICE` is unchanged.
