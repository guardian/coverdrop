CREATE TABLE organization_public_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    pk_json JSONB NOT NULL UNIQUE,
    created_at TEXT NOT NULL -- ISO formatted date time
);

CREATE TABLE journalist_provisioning_key_pairs (
    organization_pk_id INTEGER NOT NULL,
    key_pair_json JSONB NOT NULL UNIQUE,
    added_at TEXT NOT NULL, -- ISO formatted date time
    FOREIGN KEY (organization_pk_id) REFERENCES organization_public_keys (id)
);

CREATE TABLE covernode_provisioning_key_pairs (
    organization_pk_id INTEGER NOT NULL,
    key_pair_json JSONB NOT NULL UNIQUE,
    added_at TEXT NOT NULL, -- ISO formatted date time
    FOREIGN KEY (organization_pk_id) REFERENCES organization_public_keys (id)
);
