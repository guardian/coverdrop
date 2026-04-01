CREATE TABLE clients (
    client_id TEXT PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE key_packages (
    -- the key package hash uniquely identifies it
    key_package_hash BYTEA PRIMARY KEY,
    client_id TEXT NOT NULL REFERENCES clients(client_id) ON DELETE CASCADE,
    published_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ,
    key_package BYTEA NOT NULL
);
CREATE INDEX idx_key_packages_client_unconsumed ON key_packages(client_id);

-- groups table tracks epoch state for handshake messages
CREATE TABLE groups (
    group_id BYTEA PRIMARY KEY,
    epoch BIGINT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE messages (
    message_id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    to_client_id TEXT NOT NULL REFERENCES clients(client_id) ON DELETE CASCADE,
    published_at TIMESTAMPTZ NOT NULL,
    content BYTEA NOT NULL
);
-- optimizes selecting messages for a client since a given message_id
CREATE INDEX idx_messages_client_id_message_id ON messages(to_client_id, message_id);
