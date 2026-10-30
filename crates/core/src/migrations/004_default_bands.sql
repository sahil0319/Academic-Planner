-- R93: data-only migration. An attendance component whose bands are exactly the retired
-- placeholder table (90/100, 85/90, 80/80, 75/70, 70/60, 0/0) is "not configured" and gets the
-- default table of R92 (95/100, 90/80, 85/60, 75/20, 70/10, 0/0). Any other bands, and every
-- other row, are left alone. Run by migration 003 and by a version-1 backup restore (after the
-- grading seed).

CREATE TEMP TABLE unconfigured_attendance AS
SELECT c.id AS component_id
FROM rule_component c
WHERE c.kind = 'attendance'
  AND (SELECT COUNT(*) FROM attendance_band b WHERE b.component_id = c.id) = 6
  AND (SELECT COUNT(*) FROM attendance_band b
       WHERE b.component_id = c.id
         AND ((b.min_pct = 90 AND b.credit_pct = 100)
           OR (b.min_pct = 85 AND b.credit_pct = 90)
           OR (b.min_pct = 80 AND b.credit_pct = 80)
           OR (b.min_pct = 75 AND b.credit_pct = 70)
           OR (b.min_pct = 70 AND b.credit_pct = 60)
           OR (b.min_pct = 0 AND b.credit_pct = 0))) = 6;

DELETE FROM attendance_band
WHERE component_id IN (SELECT component_id FROM unconfigured_attendance);

INSERT INTO attendance_band (component_id, min_pct, credit_pct)
SELECT u.component_id, d.min_pct, d.credit_pct
FROM unconfigured_attendance u
CROSS JOIN (
  SELECT 95 AS min_pct, 100 AS credit_pct
  UNION ALL SELECT 90, 80
  UNION ALL SELECT 85, 60
  UNION ALL SELECT 75, 20
  UNION ALL SELECT 70, 10
  UNION ALL SELECT 0, 0
) d;

DROP TABLE unconfigured_attendance;
