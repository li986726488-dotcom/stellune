CREATE TABLE IF NOT EXISTS profiles (
  id TEXT PRIMARY KEY,
  profile_json TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS sky_snapshots (
  id TEXT PRIMARY KEY,
  snapshot_key TEXT NOT NULL UNIQUE,
  provider TEXT NOT NULL,
  endpoint TEXT NOT NULL,
  request_json TEXT NOT NULL,
  raw_response_json TEXT NOT NULL,
  normalized_json TEXT NOT NULL,
  captured_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS daily_readings (
  id TEXT PRIMARY KEY,
  cache_key TEXT NOT NULL UNIQUE,
  local_date TEXT NOT NULL,
  profile_id TEXT NOT NULL,
  reading_json TEXT NOT NULL,
  trace_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY(profile_id) REFERENCES profiles(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_daily_readings_date
  ON daily_readings(local_date);

CREATE TABLE IF NOT EXISTS mood_entries (
  id TEXT PRIMARY KEY,
  local_date TEXT NOT NULL UNIQUE,
  mood TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS daily_fortunes (
  id TEXT PRIMARY KEY,
  local_date TEXT NOT NULL,
  guest_id TEXT NOT NULL,
  rules_version TEXT NOT NULL,
  fortune_json TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE(local_date, guest_id, rules_version)
);
