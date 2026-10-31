-- M16 (R77, R80): cache of government public holidays per country. It is a cache: backups and CSV
-- leave it out, and a restore leaves its rows alone. Next free migration number (D1).
CREATE TABLE public_holiday (country TEXT NOT NULL, date TEXT NOT NULL, name TEXT NOT NULL,
  PRIMARY KEY (country, date, name));
