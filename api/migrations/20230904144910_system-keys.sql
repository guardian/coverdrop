CREATE TABLE system_status_pks (
    id              INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    org_pk_id       INTEGER REFERENCES organization_pks(id) NOT NULL,
    not_valid_after TIMESTAMPTZ NOT NULL, -- Denormalized to simplify key expiry
    pk_json         JSONB NOT NULL
);
CREATE UNIQUE INDEX ON system_status_pks((pk_json->>'key'));
