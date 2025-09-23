CREATE TABLE backups (
    id                  INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    created_at          TIMESTAMPTZ NOT NULL,
    data                BYTEA NOT NULL,
    data_hash           BYTEA GENERATED ALWAYS AS (digest(data, 'sha256')) STORED, -- SHA-256 hash of the data for integrity verification
    signature           BYTEA NOT NULL,
    signing_key_json    JSONB NOT NULL,
    journalist_id_pk_id INTEGER REFERENCES journalist_id_pks(id) ON DELETE CASCADE NOT NULL -- when deleting the identity key, also delete the backups since they can no longer be meaningfully verified
);
CREATE UNIQUE INDEX ON backups(data_hash);

CREATE TABLE backup_id_pks (
    id              INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    org_pk_id       INTEGER REFERENCES organization_pks(id) NOT NULL,
    not_valid_after TIMESTAMPTZ NOT NULL, -- Denormalized to simplify key expiry
    pk_json         JSONB NOT NULL
);
CREATE UNIQUE INDEX ON backup_id_pks((pk_json->>'key'));

CREATE TABLE backup_msg_pks (
    id                      INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    backup_id_pk_id         INTEGER REFERENCES backup_id_pks(id) NOT NULL,
    not_valid_after         TIMESTAMPTZ NOT NULL, -- Denormalized to simplify key expiry
    pk_json                 JSONB NOT NULL
);
CREATE UNIQUE INDEX ON backup_msg_pks((pk_json->>'key'));
