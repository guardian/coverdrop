-- Make the stage column on vault_info NOT NULL now that all vaults have had it
-- backfilled on first open since 20260527145100_vault_info_stage.sql.
-- See: https://github.com/guardian/coverdrop-internal/issues/3910
--
-- SQLite cannot alter a column's nullability in place, so the table is recreated.
-- NOTE: the new table is created before the old one is renamed since, with foreign keys
-- enabled, renaming a table updates REFERENCES clauses in other tables to point at the new name.
CREATE TABLE new_vault_info(
    journalist_id                   TEXT NOT NULL,
    max_dead_drop_id                INT NOT NULL,
    max_delivery_service_message_id INT NOT NULL DEFAULT 0,
    sentinel_id                     TEXT,
    stage                           TEXT NOT NULL,
    max_dead_drop_created_at        TEXT NOT NULL DEFAULT '1970-01-01T00:00:00Z'
);

INSERT INTO new_vault_info
    (journalist_id, max_dead_drop_id, max_delivery_service_message_id, sentinel_id, stage, max_dead_drop_created_at)
SELECT
    journalist_id, max_dead_drop_id, max_delivery_service_message_id, sentinel_id, stage, max_dead_drop_created_at
FROM vault_info;

DROP TABLE vault_info;

ALTER TABLE new_vault_info RENAME TO vault_info;

-- Slightly verbose, but robust, way of ensuring there's only one vault_info row
CREATE TRIGGER vault_info_is_unique
BEFORE INSERT ON vault_info
WHEN (SELECT COUNT(*) FROM vault_info) >= 1
BEGIN
    SELECT RAISE(FAIL, 'There can only be one vault info row');
END;
