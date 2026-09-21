CREATE TABLE seen_message_hashes (
    stream_kind TEXT NOT NULL, -- 'user_to_journalist' or 'journalist_to_user'
    hash        BLOB NOT NULL, -- SHA-256 of the encrypted inner message (U2J or J2U)
    expires_at  TEXT NOT NULL, -- ISO formatted date time
    PRIMARY KEY (stream_kind, hash)
);