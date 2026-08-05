CREATE TABLE sentinel_profiles (
    id TEXT PRIMARY KEY,
    display_name  TEXT NOT NULL, -- The display name of the Sentinel user
    added_at      TIMESTAMPTZ NOT NULL
);

CREATE TABLE sentinel_id_pks (
    id                    INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    sentinel_id           TEXT REFERENCES sentinel_profiles(id) ON DELETE CASCADE NOT NULL,
    provisioning_pk_id    INTEGER REFERENCES journalist_provisioning_pks(id) NOT NULL,
    added_at              TIMESTAMPTZ NOT NUll,
    not_valid_after       TIMESTAMPTZ NOT NULL, -- Denormalized to simplify key expiry
    pk_json               JSONB NOT NULL,
    epoch                 INTEGER NOT NULL
);
CREATE UNIQUE INDEX ON sentinel_id_pks((pk_json->>'key'));
CREATE UNIQUE INDEX ON sentinel_id_pks(epoch);

-- Redefine is_valid_key_epoch originally defined in 20240627090815_add_epoch_columns.sql
-- to include sentinel_id_pks in the max epoch check
CREATE OR REPLACE FUNCTION is_valid_key_epoch(potential_epoch INTEGER)
RETURNS BOOLEAN
as $$
DECLARE
    max_epoch INTEGER;
BEGIN
    SELECT MAX(epoch) INTO max_epoch
    FROM (
        SELECT MAX(epoch) AS epoch FROM organization_pks
        UNION
        SELECT MAX(epoch) AS epoch FROM covernode_provisioning_pks
        UNION
        SELECT MAX(epoch) AS epoch FROM covernode_id_pks
        UNION
        SELECT MAX(epoch) AS epoch FROM covernode_msg_pks
        UNION
        SELECT MAX(epoch) AS epoch FROM journalist_provisioning_pks
        UNION
        SELECT MAX(epoch) AS epoch FROM journalist_id_pks
        UNION
        SELECT MAX(epoch) AS epoch FROM journalist_msg_pks
        UNION
        SELECT MAX(epoch) AS epoch FROM sentinel_id_pks
    ) x;

    RETURN potential_epoch > max_epoch;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_epoch BEFORE INSERT OR UPDATE OR DELETE ON sentinel_id_pks
    FOR EACH ROW EXECUTE FUNCTION set_epoch();

-- Sentinel identity keys must be signed by a provisioning key.
-- These provisioning keys are present only on our on-premises machines.
-- Rather than have sentinel clients call the identity-api directly
-- they upload their rotation form to the regular API which is polled by
-- the identity-api. The identity-api then verifies the form and signs
-- the key within it, and uploads the signed key to the API.
CREATE TABLE sentinel_id_pk_rotation_queue (
    sentinel_id TEXT REFERENCES sentinel_profiles (id) PRIMARY KEY,
    form_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
);
