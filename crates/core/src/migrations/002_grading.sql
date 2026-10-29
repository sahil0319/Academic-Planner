ALTER TABLE course ADD COLUMN credits REAL CHECK (credits IS NULL OR credits > 0);   -- NULL = not graded
ALTER TABLE course ADD COLUMN rule_set_id INTEGER REFERENCES rule_set(id);          -- NULL = not graded

CREATE TABLE rule_set (id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE);

CREATE TABLE rule_component (
  id           INTEGER PRIMARY KEY,
  rule_set_id  INTEGER NOT NULL REFERENCES rule_set(id) ON DELETE CASCADE,
  position     INTEGER NOT NULL,
  name         TEXT NOT NULL,
  kind         TEXT NOT NULL CHECK (kind IN ('attendance','items')),
  weight_pct   REAL NOT NULL CHECK (weight_pct > 0 AND weight_pct <= 100),
  item_count   INTEGER CHECK (item_count >= 1),          -- items: N items held
  best_count   INTEGER CHECK (best_count >= 1),          -- items: best K counted
  item_max     REAL CHECK (item_max IS NULL OR item_max > 0),  -- items: marks each item is out of; NULL = the component's marks for that course
  is_final     INTEGER NOT NULL DEFAULT 0,
  UNIQUE (rule_set_id, name)
);

CREATE TABLE attendance_band (
  component_id  INTEGER NOT NULL REFERENCES rule_component(id) ON DELETE CASCADE,
  min_pct       REAL NOT NULL CHECK (min_pct BETWEEN 0 AND 100),
  credit_pct    REAL NOT NULL CHECK (credit_pct BETWEEN 0 AND 100),
  PRIMARY KEY (component_id, min_pct)
);

CREATE TABLE grade_scale (
  letter   TEXT PRIMARY KEY,
  min_pct  REAL NOT NULL UNIQUE CHECK (min_pct BETWEEN 0 AND 100),
  points   REAL NOT NULL CHECK (points >= 0)
);

ALTER TABLE assessment ADD COLUMN component_id INTEGER REFERENCES rule_component(id);  -- NULL = not counted in marks
ALTER TABLE assessment ADD COLUMN score REAL CHECK (score IS NULL OR score >= 0);
