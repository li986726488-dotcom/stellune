ALTER TABLE daily_readings
  ADD COLUMN profile_version INTEGER NOT NULL DEFAULT 1;

ALTER TABLE daily_readings
  ADD COLUMN profile_snapshot_json TEXT NOT NULL DEFAULT '{}';

ALTER TABLE daily_readings
  ADD COLUMN is_current INTEGER NOT NULL DEFAULT 1;

ALTER TABLE daily_readings
  ADD COLUMN superseded_at TEXT;

UPDATE daily_readings
SET
  is_current = 0,
  superseded_at = created_at
WHERE rowid NOT IN (
  SELECT newest.rowid
  FROM daily_readings AS newest
  WHERE newest.rowid = (
    SELECT candidate.rowid
    FROM daily_readings AS candidate
    WHERE
      candidate.local_date = newest.local_date
      AND candidate.profile_id = newest.profile_id
    ORDER BY candidate.created_at DESC, candidate.rowid DESC
    LIMIT 1
  )
);

CREATE INDEX IF NOT EXISTS idx_daily_readings_current_date
  ON daily_readings(profile_id, local_date, is_current, created_at);
