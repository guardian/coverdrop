CREATE TABLE backup_types (
    backup_type TEXT PRIMARY KEY
);

INSERT INTO backup_types (backup_type) VALUES
    ('AUTOMATED'),
    ('MANUAL');

ALTER TABLE backup_history RENAME TO backup_history_old;

CREATE TABLE backup_history (
    backup_type  TEXT NOT NULL REFERENCES backup_types(backup_type),
    timestamp    TEXT NOT NULL, -- ISO formatted datetime
    path         TEXT,  -- Path to the backup file (for manual backups)
    recovery_contacts     JSONB  -- array of recovery contact journalist ids (for automated backups)
);

INSERT INTO backup_history (backup_type, timestamp, path, recovery_contacts)
SELECT
    'MANUAL' AS backup_type,
    timestamp,
    path,
    NULL AS recovery_contacts
FROM backup_history_old;

DROP TABLE backup_history_old;
