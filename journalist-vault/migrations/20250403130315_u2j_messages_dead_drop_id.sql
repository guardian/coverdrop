ALTER TABLE u2j_messages
RENAME TO u2j_messages_old;

CREATE TABLE u2j_messages (
    id           INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_pk      BLOB NOT NULL,
    message      BLOB NOT NULL, -- PaddedCompressedString
    received_at  TEXT NOT NULL, -- ISO formatted date
    read         INTEGER NOT NULL DEFAULT 0,
    dead_drop_id INTEGER NOT NULL,
    FOREIGN KEY(user_pk) REFERENCES users(user_pk)
);

INSERT INTO u2j_messages (id, user_pk, message, received_at, read, dead_drop_id)
SELECT id, user_pk, message, received_at, read, 0
FROM u2j_messages_old;

DROP TABLE u2j_messages_old;
