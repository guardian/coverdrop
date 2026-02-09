-- Remove the anchor_organization_pks table and the organization_pk_id column
-- from journalist_provisioning_pks. Since organization_pk_id has a foreign key constraint,
-- we must recreate the table. This also requires recreating tables that reference it.

-- Drop old indexes first to avoid conflicts
DROP INDEX IF EXISTS journalist_provisioning_pks_unique_pk_json;
DROP INDEX IF EXISTS journalist_id_key_pairs_unique_key_pair_json;
DROP INDEX IF EXISTS journalist_msg_key_pairs_unique_key_pair_json;

-- Rename old tables
ALTER TABLE journalist_provisioning_pks
    RENAME TO journalist_provisioning_pks_old;
ALTER TABLE vault_setup_bundle
    RENAME TO vault_setup_bundle_old;
ALTER TABLE journalist_id_key_pairs
    RENAME TO journalist_id_key_pairs_old;
ALTER TABLE journalist_msg_key_pairs
    RENAME TO journalist_msg_key_pairs_old;

-- Recreate journalist_provisioning_pks without organization_pk_id and its foreign key
CREATE TABLE journalist_provisioning_pks(
    id       INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    pk_json  JSONB NOT NULL,
    added_at TEXT NOT NULL -- ISO formatted datetime
);

-- Recreate vault_setup_bundle with foreign key to new journalist_provisioning_pks
CREATE TABLE vault_setup_bundle(
    id                            INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    provisioning_pk_id            INTEGER NOT NULL,
    pk_upload_form_json           JSONB NOT NULL,
    keypair_json                  JSONB NOT NULL,
    register_journalist_form_json JSONB, -- Nullable, only used in the very first creation of a vault
    FOREIGN KEY (provisioning_pk_id) REFERENCES journalist_provisioning_pks(id) ON DELETE CASCADE
);

-- Recreate journalist_id_key_pairs with foreign key to new journalist_provisioning_pks
CREATE TABLE journalist_id_key_pairs(
    id                 INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    provisioning_pk_id INTEGER NOT NULL,
    key_pair_json      JSONB NOT NULL,
    created_at         TEXT NOT NULL, -- ISO formatted datetime
    published_at       TEXT NOT NULL, -- ISO formatted datetime
    epoch              INTEGER NOT NULL,
    FOREIGN KEY (provisioning_pk_id) REFERENCES journalist_provisioning_pks(id) ON DELETE CASCADE
);

-- Recreate journalist_msg_key_pairs with foreign key to new journalist_id_key_pairs
CREATE TABLE journalist_msg_key_pairs(
    id             INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    id_key_pair_id INTEGER NOT NULL,
    key_pair_json  JSONB NOT NULL,
    added_at       TEXT NOT NULL, -- ISO formatted datetime
    epoch          INTEGER,
    FOREIGN KEY (id_key_pair_id) REFERENCES journalist_id_key_pairs(id) ON DELETE CASCADE
);

-- Recreate indexes
CREATE UNIQUE INDEX journalist_provisioning_pks_unique_pk_json ON journalist_provisioning_pks(pk_json);
CREATE UNIQUE INDEX journalist_id_key_pairs_unique_key_pair_json ON journalist_id_key_pairs(key_pair_json);
CREATE UNIQUE INDEX journalist_msg_key_pairs_unique_key_pair_json ON journalist_msg_key_pairs(key_pair_json);

-- Copy data from old tables (excluding organization_pk_id)
INSERT INTO journalist_provisioning_pks (id, pk_json, added_at)
    SELECT id, pk_json, added_at FROM journalist_provisioning_pks_old;
INSERT INTO vault_setup_bundle (id, provisioning_pk_id, pk_upload_form_json, keypair_json, register_journalist_form_json)
    SELECT id, provisioning_pk_id, pk_upload_form_json, keypair_json, register_journalist_form_json FROM vault_setup_bundle_old;
INSERT INTO journalist_id_key_pairs (id, provisioning_pk_id, key_pair_json, created_at, published_at, epoch)
    SELECT id, provisioning_pk_id, key_pair_json, created_at, published_at, epoch FROM journalist_id_key_pairs_old;
INSERT INTO journalist_msg_key_pairs (id, id_key_pair_id, key_pair_json, added_at, epoch)
    SELECT id, id_key_pair_id, key_pair_json, added_at, epoch FROM journalist_msg_key_pairs_old;

-- Drop old tables
DROP TABLE journalist_msg_key_pairs_old;
DROP TABLE journalist_id_key_pairs_old;
DROP TABLE vault_setup_bundle_old;
DROP TABLE journalist_provisioning_pks_old;

-- Recreate trigger for vault_setup_bundle uniqueness
CREATE TRIGGER vault_setup_bundle_is_unique
BEFORE INSERT ON vault_setup_bundle
WHEN (SELECT COUNT(*) FROM vault_setup_bundle) >= 1
BEGIN
    SELECT RAISE(FAIL, 'There can only be one vault setup bundle row');
END;

-- Drop the anchor_organization_pks table
DROP TABLE anchor_organization_pks;
