-- Add sentinel identity to vault_info
-- (nullable during migration to populate the column for existing vaults)
ALTER TABLE vault_info ADD COLUMN sentinel_id TEXT;

-- Sentinel identity key pairs (published, signed by provisioning key)
CREATE TABLE sentinel_id_key_pairs(
    id                 INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    provisioning_pk_id INTEGER NOT NULL,
    key_pair_json      JSONB NOT NULL,
    created_at         TEXT NOT NULL, -- ISO formatted date
    published_at       TEXT NOT NULL, -- ISO formatted date
    epoch              INTEGER NOT NULL,
    FOREIGN KEY (provisioning_pk_id) REFERENCES journalist_provisioning_pks(id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX sentinel_id_key_pairs_unique_pk_json ON sentinel_id_key_pairs(key_pair_json);

-- Candidate sentinel identity key pair (awaiting signature from journalist provisioning key)
-- Only one row should exist at a time, enforced by trigger
CREATE TABLE candidate_sentinel_id_key_pair(
    id                 INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    key_pair_json      JSONB NOT NULL,
    added_at           TEXT NOT NULL -- ISO formatted date
);

CREATE TRIGGER candidate_sentinel_id_key_pair_is_unique
BEFORE INSERT ON candidate_sentinel_id_key_pair
WHEN (SELECT COUNT(*) FROM candidate_sentinel_id_key_pair) >= 1
BEGIN
    SELECT RAISE(FAIL, 'There can only be one candidate sentinel id key pair');
END;
