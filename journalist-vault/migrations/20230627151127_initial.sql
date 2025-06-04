--
-- Info, only should be one row of this
--

CREATE TABLE vault_info(
    journalist_id    TEXT NOT NULL,
    max_dead_drop_id INT NOT NULL
);

-- Slightly verbose, but robust, way of ensuring there's only one vault_info row
CREATE TRIGGER vault_info_is_unique
BEFORE INSERT ON vault_info
WHEN (SELECT COUNT(*) FROM vault_info) >= 1
BEGIN
    SELECT RAISE(FAIL, 'There can only be one vault info row');
END;

--
-- Messages
--

CREATE TABLE messages(
    user_pk      BLOB NOT NULL,
    is_from_user BOOLEAN NOT NULL,
    message      BLOB NOT NULL, -- PaddedCompressedString
    received_at  TEXT NOT NULL  -- ISO formatted date
);

--
-- Key Hierarchy
--

CREATE TABLE trusted_organization_pks(
    id       INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    pk_json  TEXT NOT NULL,
    added_at TEXT NOT NULL -- ISO formatted date
);

CREATE TABLE journalist_provisioning_pks(
    id                 INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    organization_pk_id INTEGER NOT NULL,
    pk_json            TEXT NOT NULL,
    added_at           TEXT NOT NULL, -- ISO formatted date
    FOREIGN KEY (organization_pk_id) REFERENCES trusted_organization_pks(id)
);

CREATE TABLE journalist_id_keypairs(
    id                 INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    provisioning_pk_id INTEGER NOT NULL,
    keypair_json       TEXT NOT NULL,
    added_at           TEXT NOT NULL, -- ISO formatted date
    synced_at          TEXT NOT NULL, -- ISO formatted date
    FOREIGN KEY (provisioning_pk_id) REFERENCES journalist_id_keypairs(id)
);

CREATE TABLE journalist_msg_keypairs(
    id            INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    id_keypair_id INTEGER NOT NULL,
    keypair_json  TEXT NOT NULL,
    added_at      TEXT NOT NULL, -- ISO formatted date
    synced_at     TEXT,          -- ISO formatted date, nullable
    FOREIGN KEY (id_keypair_id) REFERENCES journalist_id_keypairs(id)
);

--
-- Unregistered ID keys
--

CREATE TABLE unregistered_journalist_id_keypairs(
    id            INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    keypair_json  TEXT NOT NULL,
    added_at      TEXT NOT NULL -- ISO formatted date
);