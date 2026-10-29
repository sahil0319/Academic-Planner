-- R33-R39: default rule sets, attendance bands, and grade scale (docs/spec.md §13.3).
-- Run by migration 002 and by a version-1 backup restore. Fixed ids.

INSERT INTO rule_set (id, name) VALUES (1, 'Theory'), (2, 'Lab');

INSERT INTO rule_component
  (id, rule_set_id, position, name, kind, weight_pct, item_count, best_count, item_max, is_final)
VALUES
  (1, 1, 0, 'Attendance', 'attendance', 10, NULL, NULL, NULL, 0),
  (2, 1, 1, 'Quizzes',    'items',      15, 4,    3,    15,   0),
  (3, 1, 2, 'Mid',        'items',      25, 1,    1,    NULL, 0),
  (4, 1, 3, 'Final',      'items',      50, 1,    1,    NULL, 1),
  (5, 2, 0, 'Attendance', 'attendance', 30, NULL, NULL, NULL, 0),
  (6, 2, 1, 'Lab work',   'items',      40, 1,    1,    NULL, 0),
  (7, 2, 2, 'Lab final',  'items',      30, 1,    1,    NULL, 1);

INSERT INTO attendance_band (component_id, min_pct, credit_pct) VALUES
  (1, 90, 100), (1, 85, 90), (1, 80, 80), (1, 75, 70), (1, 70, 60), (1, 0, 0),
  (5, 90, 100), (5, 85, 90), (5, 80, 80), (5, 75, 70), (5, 70, 60), (5, 0, 0);

INSERT INTO grade_scale (letter, min_pct, points) VALUES
  ('A+', 80, 4.00), ('A', 75, 3.75), ('A-', 70, 3.50), ('B+', 65, 3.25), ('B', 60, 3.00),
  ('B-', 55, 2.75), ('C+', 50, 2.50), ('C', 45, 2.25), ('D', 40, 2.00), ('F', 0, 0.00);
