# Contract: backup JSON version 2 (R41)

Amends R21/R22. CSV (R23) is unchanged.

## Export (`export_json(now)`)

```json
{
  "format": "academic-planner-backup",
  "schema_version": 2,
  "exported_at": "…",
  "terms": [], "holidays": [], "courses": [], "slots": [], "sessions": [], "assessments": [],
  "settings": {},
  "rule_sets": [],
  "rule_components": [],
  "attendance_bands": [],
  "grade_scale": []
}
```

- `courses[]` rows gain `credits` (number or null) and `rule_set_id` (integer or null).
- `assessments[]` rows gain `component_id` (integer or null) and `score` (number or null).
- `rule_sets[]`: `{id, name}` ordered by id.
- `rule_components[]`: `{id, rule_set_id, position, name, kind, weight_pct, item_count,
  best_count, item_max, is_final}` ordered by id; `kind` is `"attendance"` or `"items"`; nullable
  keys are always written.
- `attendance_bands[]`: `{component_id, min_pct, credit_pct}` ordered by (component_id, min_pct).
- `grade_scale[]`: `{letter, min_pct, points}` ordered by min_pct descending.
- IDs are preserved (R21).

## Restore (`restore_json(json)`)

- R22 still applies: `format` checked, `schema_version` above `SCHEMA_VERSION` (2) rejected, all
  data replaced in one transaction, DB unchanged on any error.
- `schema_version` 2: the four arrays are required (missing → the same error kind R22 uses for a
  malformed backup). Rows are inserted as given; DB constraints and foreign keys are the only
  checks (deferred foreign keys, as today).
- `schema_version` 1: the four arrays are ignored if present; course `credits`/`rule_set_id` and
  assessment `component_id`/`score` are stored as NULL; the four tables are cleared and
  `grading_seed.sql` runs, so they equal a fresh database (T42).
- New course/assessment keys default to null when absent.
- Tables cleared before insert: `session`, `assessment`, `slot`, `course`, `holiday`, `term`,
  `setting`, `attendance_band`, `rule_component`, `rule_set`, `grade_scale`.

## Tests

- T42: v2 export → restore → export equal except `exported_at` (with a graded course, a linked
  scored assessment, an edited rule set, and an edited grade scale); a v1 backup restores with
  ungraded courses and seeded rule sets, components, bands, and grade scale.
- Existing R21/R22 tests: pins move from 1 to 2 (research D14).
