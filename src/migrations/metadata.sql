CREATE TABLE IF NOT EXISTS metadata (
  name TEXT NOT NULL,
  path TEXT NOT NULL,
  size_bytes BIGINT,
  discovered_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  server_address TEXT NOT NULL,
  created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
  updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_metadata_name on metadata(name);
-- if you want add column, please use alter query field at below
