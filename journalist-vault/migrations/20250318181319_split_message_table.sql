CREATE TABLE u2j_messages (
    id           INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_pk      BLOB NOT NULL,
    message      BLOB NOT NULL, -- PaddedCompressedString
    received_at  TEXT NOT NULL, -- ISO formatted date
    read         INTEGER NOT NULL DEFAULT 0, 
    FOREIGN KEY(user_pk) REFERENCES users(user_pk)
);

CREATE TABLE j2u_messages (
    id                  INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    user_pk             BLOB NOT NULL,
    message             BLOB NOT NULL, -- PaddedCompressedString
    sent_at             TEXT NOT NULL, -- ISO formatted date
    outbound_queue_id   INTEGER,
    FOREIGN KEY(user_pk) REFERENCES users(user_pk)
);

INSERT INTO u2j_messages (user_pk, message, received_at, read)
SELECT user_pk, message, received_at, read
FROM messages
WHERE is_from_user = 1;

INSERT INTO j2u_messages (user_pk, message, sent_at, outbound_queue_id)
SELECT user_pk, message, received_at, outbound_queue_id
FROM messages
WHERE is_from_user = 0;

DROP TABLE messages;
