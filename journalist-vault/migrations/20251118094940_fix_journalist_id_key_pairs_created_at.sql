-- 20251103125700_id_key_created_and_published_timestamps contains a bug where the default value
-- for created_at is set to the string 'published_at' instead of the value of the published_at column.
-- This migration fixes that by recreating the table with the corrected `created_at` column definition.

-- Create the new table with the corrected `created_at` column definition
--  * Change created_at to a TEXT column to be consistent with published_at
--  * remove the default value
-- NOTE: It's important to create the new table first, rather than rename the old table since according
-- to the docs "if foreign key constraints are enabled when a table is renamed, then any REFERENCES clauses
-- in any table ... that refer to the table being renamed are modified to refer to the renamed table by its new name."
-- We want to avoid journalist_msg_key_pairs referencing the old table.
CREATE TABLE new_journalist_id_key_pairs(
    id                 INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    provisioning_pk_id INTEGER NOT NULL,
    key_pair_json      JSONB NOT NULL,
    created_at         TEXT NOT NULL, -- ISO formatted datetime
    published_at       TEXT NOT NULL, -- ISO formatted datetime
    epoch              INTEGER NOT NULL,
    FOREIGN KEY (provisioning_pk_id) REFERENCES journalist_provisioning_pks(id) ON DELETE CASCADE
);

-- Migrate data from the old table to the new table
INSERT INTO new_journalist_id_key_pairs (id, provisioning_pk_id, key_pair_json, published_at, epoch, created_at)
SELECT
    id,
    provisioning_pk_id,
    key_pair_json,
    published_at,
    epoch,
    CASE WHEN created_at = 'published_at' THEN published_at ELSE created_at END AS created_at
FROM journalist_id_key_pairs;

-- Drop the old table
DROP TABLE journalist_id_key_pairs;

-- Rename the new table
ALTER TABLE new_journalist_id_key_pairs RENAME TO journalist_id_key_pairs;

-- Recreate index
CREATE UNIQUE INDEX journalist_id_key_pairs_unique_key_pair_json ON journalist_id_key_pairs(key_pair_json);
