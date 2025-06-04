-- Journalist identity keys must be signed by a provisioning key
-- these provisioning keys are present only on our on-premises machines.
-- Rather than have journalist clients call the identity-api directly
-- they upload their rotation form to the regular API which is polled by
-- the identity-api. The identity-api then verifies the form and signs
-- the key within it, and uploads the signed key to the API.
CREATE TABLE journalist_id_pk_rotation_queue (
    journalist_id TEXT REFERENCES journalist_profiles (id) PRIMARY KEY,
    form_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW ()
);
