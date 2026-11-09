-- Revert deduplication_id back to nullable
CREATE TABLE new_outbound_queue(
    id               INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    message          BLOB NOT NULL,
    deduplication_id TEXT -- UUID
);

INSERT INTO new_outbound_queue
    (id, message, deduplication_id)
SELECT
    id, message, deduplication_id
FROM outbound_queue;

DROP TABLE outbound_queue;

ALTER TABLE new_outbound_queue RENAME TO outbound_queue;
