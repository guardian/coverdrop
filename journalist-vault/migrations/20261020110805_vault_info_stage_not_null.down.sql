-- Revert stage back to nullable
CREATE TABLE new_vault_info(
    journalist_id                   TEXT NOT NULL,
    max_dead_drop_id                INT NOT NULL,
    max_delivery_service_message_id INT NOT NULL DEFAULT 0,
    sentinel_id                     TEXT,
    stage                           TEXT,
    max_dead_drop_created_at        TEXT NOT NULL DEFAULT '1970-01-01T00:00:00Z'
);

INSERT INTO new_vault_info
    (journalist_id, max_dead_drop_id, max_delivery_service_message_id, sentinel_id, stage, max_dead_drop_created_at)
SELECT
    journalist_id, max_dead_drop_id, max_delivery_service_message_id, sentinel_id, stage, max_dead_drop_created_at
FROM vault_info;

DROP TABLE vault_info;

ALTER TABLE new_vault_info RENAME TO vault_info;

CREATE TRIGGER vault_info_is_unique
BEFORE INSERT ON vault_info
WHEN (SELECT COUNT(*) FROM vault_info) >= 1
BEGIN
    SELECT RAISE(FAIL, 'There can only be one vault info row');
END;
