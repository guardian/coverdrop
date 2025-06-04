--
-- Organization
--

-- Organization keys are sync'd from the API's filesystem
CREATE TABLE organization_pks (
    id       INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    added_at TIMESTAMPTZ NOT NUll,
    pk_json  JSONB NOT NULL
);
CREATE UNIQUE INDEX ON organization_pks((pk_json->>'key'));

--
-- CoverNode
--

CREATE TABLE covernode_provisioning_pks (
    id              INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    org_pk_id       INTEGER REFERENCES organization_pks(id) NOT NULL,

    added_at        TIMESTAMPTZ NOT NUll,
    not_valid_after TIMESTAMPTZ NOT NULL, -- Denormalized to simplify key expiry
    pk_json         JSONB NOT NULL
);
CREATE UNIQUE INDEX ON covernode_provisioning_pks((pk_json->>'key'));

CREATE TABLE covernodes (
    id       TEXT PRIMARY KEY,
    added_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE covernode_id_pks (
    id                 INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    covernode_id       TEXT REFERENCES covernodes(id) NOT NULL,
    provisioning_pk_id INTEGER REFERENCES covernode_provisioning_pks(id) NOT NULL,

    added_at           TIMESTAMPTZ NOT NUll,
    not_valid_after    TIMESTAMPTZ NOT NULL, -- Denormalized to simplify key expiry
    pk_json            JSONB NOT NULL
);
CREATE UNIQUE INDEX ON covernode_id_pks((pk_json->>'key'));

CREATE TABLE covernode_msg_pks (
    id              INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    covernode_id    TEXT REFERENCES covernodes(id) NOT NULL,
    id_pk_id        INTEGER REFERENCES covernode_id_pks(id) NOT NULL,

    added_at        TIMESTAMPTZ NOT NUll,
    not_valid_after TIMESTAMPTZ NOT NULL, -- Denormalized to simplify key expiry
    pk_json         JSONB NOT NULL
);
CREATE UNIQUE INDEX ON covernode_msg_pks((pk_json->>'key'));

--
-- Journalist
--

CREATE TABLE journalist_provisioning_pks  (
    id              INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    org_pk_id       INTEGER REFERENCES organization_pks(id) NOT NULL,

    added_at        TIMESTAMPTZ NOT NUll,
    not_valid_after TIMESTAMPTZ NOT NULL, -- Denormalized to simplify key expiry
    pk_json         JSONB NOT NULL
);
CREATE UNIQUE INDEX ON journalist_provisioning_pks((pk_json->>'key'));

CREATE TABLE journalist_profiles (
    id            TEXT PRIMARY KEY,
    display_name  TEXT NOT NULL,    -- The display name of the journalist or desk
    sort_name     TEXT NOT NULL,    -- The name used for sorting the journalist, e.g. "Joe Bloggs" would be "bloggs joe"
    description   TEXT NOT NULL,    -- A short description for a reporter, for a desk it can be more detailed
    is_desk       BOOLEAN NOT NULL,  -- This "journalist" is actually a desk and should appear in that section in the UI
    added_at      TIMESTAMPTZ NOT NUll
);

CREATE TABLE journalist_id_pks (
    id                    INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    journalist_profile_id TEXT REFERENCES journalist_profiles(id) ON DELETE CASCADE NOT NULL,
    provisioning_pk_id    INTEGER REFERENCES journalist_provisioning_pks(id) NOT NULL,

    added_at              TIMESTAMPTZ NOT NUll,
    not_valid_after       TIMESTAMPTZ NOT NULL, -- Denormalized to simplify key expiry
    pk_json               JSONB NOT NULL
);
CREATE UNIQUE INDEX ON journalist_id_pks((pk_json->>'key'));

CREATE TABLE journalist_msg_pks (
    id                    INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    journalist_profile_id TEXT REFERENCES journalist_profiles(id) ON DELETE CASCADE NOT NULL,
    id_pk_id              INTEGER REFERENCES journalist_id_pks(id) NOT NULL,

    added_at              TIMESTAMPTZ NOT NUll,
    not_valid_after       TIMESTAMPTZ NOT NULL, -- Denormalized to simplify key expiry
    pk_json               JSONB NOT NULL
);
CREATE UNIQUE INDEX ON journalist_msg_pks((pk_json->>'key'));
