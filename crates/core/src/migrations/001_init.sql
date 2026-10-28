-- Migration 001: the docs/spec.md §3 schema verbatim, plus the §3 setting defaults.
-- Never edit after release; schema changes go in a new numbered file.

CREATE TABLE term (
  id          INTEGER PRIMARY KEY,
  name        TEXT NOT NULL,
  start_date  TEXT NOT NULL,
  end_date    TEXT NOT NULL,
  is_active   INTEGER NOT NULL DEFAULT 0,
  CHECK (end_date >= start_date)
);
CREATE UNIQUE INDEX one_active_term ON term(is_active) WHERE is_active = 1;

CREATE TABLE holiday (
  term_id  INTEGER NOT NULL REFERENCES term(id) ON DELETE CASCADE,
  date     TEXT NOT NULL,
  label    TEXT,
  PRIMARY KEY (term_id, date)
);

CREATE TABLE course (
  id                  INTEGER PRIMARY KEY,
  term_id             INTEGER NOT NULL REFERENCES term(id),
  code                TEXT NOT NULL,
  name                TEXT NOT NULL,
  instructor          TEXT,
  room                TEXT,
  notes               TEXT,
  min_attendance_pct  INTEGER CHECK (min_attendance_pct BETWEEN 0 AND 100),
  archived            INTEGER NOT NULL DEFAULT 0,
  UNIQUE (term_id, code)
);

CREATE TABLE slot (                         -- recurring timetable rule
  id          INTEGER PRIMARY KEY,
  course_id   INTEGER NOT NULL REFERENCES course(id),
  weekday     INTEGER NOT NULL CHECK (weekday BETWEEN 1 AND 7),
  start_time  TEXT NOT NULL,
  end_time    TEXT NOT NULL,
  room        TEXT,                         -- NULL = use course.room
  valid_from  TEXT NOT NULL,                -- inclusive
  valid_to    TEXT,                         -- inclusive; NULL = until term end
  CHECK (end_time > start_time)
);

CREATE TABLE session (                      -- one dated class occurrence
  id                   INTEGER PRIMARY KEY,
  course_id            INTEGER NOT NULL REFERENCES course(id),
  slot_id              INTEGER REFERENCES slot(id),          -- NULL for makeup
  kind                 TEXT NOT NULL CHECK (kind IN ('regular','makeup')),
  date                 TEXT NOT NULL,
  start_time           TEXT NOT NULL,
  end_time             TEXT NOT NULL,
  room                 TEXT,
  status               TEXT NOT NULL DEFAULT 'pending'
                       CHECK (status IN ('pending','attended','absent','cancelled')),
  replaces_session_id  INTEGER REFERENCES session(id),       -- set on a makeup that reschedules an original
  note                 TEXT,
  UNIQUE (slot_id, date),
  CHECK (end_time > start_time),
  CHECK ((kind = 'regular') = (slot_id IS NOT NULL))
);

CREATE TABLE assessment (
  id                      INTEGER PRIMARY KEY,
  course_id               INTEGER NOT NULL REFERENCES course(id),
  kind                    TEXT NOT NULL CHECK (kind IN ('quiz','exam','assignment','other')),
  title                   TEXT NOT NULL,
  due_at                  TEXT NOT NULL,
  notes                   TEXT,
  completed               INTEGER NOT NULL DEFAULT 0,
  remind_minutes_before   INTEGER CHECK (remind_minutes_before >= 0)  -- NULL = no reminder
);

CREATE TABLE setting (key TEXT PRIMARY KEY, value TEXT NOT NULL);
-- defaults: class_reminder_minutes=15, upcoming_window_days=7, notifications_enabled=1

CREATE TABLE reminder_log (                 -- M5: prevents duplicate notifications
  kind     TEXT NOT NULL CHECK (kind IN ('session','assessment')),
  ref_id   INTEGER NOT NULL,
  fire_at  TEXT NOT NULL,
  PRIMARY KEY (kind, ref_id, fire_at)
);

INSERT INTO setting (key, value) VALUES
  ('class_reminder_minutes', '15'),
  ('upcoming_window_days', '7'),
  ('notifications_enabled', '1');
