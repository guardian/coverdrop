-- Make the deduplication_id column on outbound_queue NOT NULL now that all queued
-- messages created before the column was added (20260722153843, 3 months ago) have
-- long since been drained from the FIFO queue.
-- See: https://github.com/guardian/coverdrop-internal/issues/4087
--
-- SQLite cannot alter a column's nullability in place, so the table is recreated.
CREATE TABLE new_outbound_queue(
    id               INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    message          BLOB NOT NULL,
    deduplication_id TEXT NOT NULL -- UUID
);

INSERT INTO new_outbound_queue
    (id, message, deduplication_id)
SELECT
    id, message, deduplication_id
FROM outbound_queue;

DROP TABLE outbound_queue;

ALTER TABLE new_outbound_queue RENAME TO outbound_queue;

