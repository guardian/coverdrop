-- Add dead_drop_created_at column and unique constraint for U2J message deduplication.
-- Deduplication is based on (user_pk, dead_drop_created_at, message).

ALTER TABLE u2j_messages
RENAME TO u2j_messages_old;

CREATE TABLE u2j_messages (
    id                   INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_pk              BLOB NOT NULL,
    message              BLOB NOT NULL, -- PaddedCompressedString
    received_at          TEXT NOT NULL, -- ISO formatted date
    read                 INTEGER NOT NULL DEFAULT 0,
    dead_drop_id         INTEGER NOT NULL,
    dead_drop_created_at TEXT NOT NULL, -- ISO formatted date
    custom_expiry        TEXT, -- ISO formatted date
    FOREIGN KEY(user_pk) REFERENCES users(user_pk),
    UNIQUE(user_pk, dead_drop_created_at, message)
);

INSERT OR IGNORE INTO u2j_messages (id, user_pk, message, received_at, read, dead_drop_id, dead_drop_created_at, custom_expiry)
SELECT
    id,
    user_pk,
    message,
    received_at,
    read,
    dead_drop_id,
    received_at AS dead_drop_created_at, -- backfill dead_drop_created_at with received_at timestamp for existing records
    custom_expiry
FROM u2j_messages_old;

DROP TABLE u2j_messages_old;
