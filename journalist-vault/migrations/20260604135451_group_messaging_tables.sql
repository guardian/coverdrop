CREATE TABLE groups (
    id TEXT PRIMARY KEY NOT NULL, -- UUID string
    -- display name and description can be null between receiving a Welcome message
    -- and the subsequent GroupCreated message which contains the full group info.
    display_name TEXT,
    description TEXT
);

CREATE TABLE j2j_messages (
    id TEXT PRIMARY KEY NOT NULL, -- UUID string
    group_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    message JSON NOT NULL,
    published_at TIMESTAMPTZ NOT NULL,
    read BOOLEAN NOT NULL DEFAULT FALSE,
    FOREIGN KEY (group_id) REFERENCES groups(id)
);
