CREATE TABLE user_statuses (
    status TEXT PRIMARY KEY NOT NULL
);

INSERT INTO user_statuses (status) VALUES ('ACTIVE'), ('MUTED');

CREATE TABLE users (
    user_pk                 BLOB PRIMARY KEY NOT NULL,
    status                  TEXT NOT NULL DEFAULT 'ACTIVE',
    status_updated_at       TEXT NOT NULL, -- ISO formatted date
    FOREIGN KEY (status)    REFERENCES user_statuses(status)
);

INSERT INTO users (user_pk, status, status_updated_at)
SELECT 
    user_pk,
    'ACTIVE',
    MIN(received_at)
FROM messages
GROUP BY 1;
