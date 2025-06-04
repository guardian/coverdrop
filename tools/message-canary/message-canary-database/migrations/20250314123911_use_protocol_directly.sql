DELETE FROM users;

-- Don't want to manage user vaults specifically
ALTER TABLE users
DROP COLUMN mailbox_path,
DROP COLUMN password,
ADD COLUMN key_pair_json JSONB NOT NULL;

-- We don't really need to track the vault path
-- since we can just autodiscover the vaults in the
-- file system periodically.
ALTER TABLE journalists
DROP COLUMN vault_path;

-- We need to pull both types of dead drops now since
-- the signal-bridge is removed which means the message
-- canary will be operating against the CoverDrop protocol
-- directly.
ALTER TABLE processed_dead_drops
RENAME TO j2u_processed_dead_drops;

CREATE TABLE u2j_processed_dead_drops (
    dead_drop_id INTEGER NOT NULL,
    processed_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX ON u2j_processed_dead_drops (dead_drop_id);

-- We use UUIDs for the message and then match on the contents of a message
-- this makes it effectively a primary key. So let's enforce that constraint
-- It's also used in queries to filter so it's nice to have an index
ALTER TABLE user_to_journalist_messages ADD PRIMARY KEY (message);

ALTER TABLE journalist_to_user_messages ADD PRIMARY KEY (message);
