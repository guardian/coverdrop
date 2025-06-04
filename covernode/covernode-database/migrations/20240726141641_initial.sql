--
-- Initial setup table
--

CREATE TABLE setup_bundle(
    pk_upload_form_json JSONB NOT NULL,
    key_pair_json       JSONB NOT NULL,
    created_at          TEXT NOT NULL -- ISO formatted date time
);

CREATE TRIGGER setup_bundle_is_unique
BEFORE INSERT ON setup_bundle
WHEN (SELECT COUNT(*) FROM setup_bundle) >= 1
BEGIN
    SELECT RAISE(FAIL, 'There can only be one database setup bundle row');
END;

--
-- CoverNode ID key pairs
--

CREATE TABLE covernode_id_key_pairs (
    epoch         INTEGER,
    -- If there's no epoch this is the "UnregisteredCoverNodeIdKeyPair" type.
    -- Once the epoch is set this gets changed to include a signature so it is a "CoverNodeIdKeyPair"
    key_pair_json JSONB NOT NULL,
    created_at    TEXT NOT NULL -- ISO formatted date time
);

CREATE TRIGGER only_one_id_candidate_key_pair
BEFORE INSERT ON covernode_id_key_pairs
WHEN (SELECT COUNT(*) FROM covernode_id_key_pairs WHERE epoch IS NULL) = 1
BEGIN
    SELECT RAISE(FAIL, 'Can only have one candidate id key pair at one time');
END;

-- there should never be more than one copy of the same key pair
CREATE UNIQUE INDEX covernode_id_key_pairs_unique_key_pair_json ON covernode_id_key_pairs(json_extract(key_pair_json, '$.secret_key'));
--
-- CoverNode messaging key pairs
--

CREATE TABLE covernode_msg_key_pairs (
    epoch         INTEGER,
    key_pair_json JSONB NOT NULL,
    created_at    TEXT NOT NULL -- ISO formatted date time
);

CREATE TRIGGER only_one_msg_candidate_key_pair
BEFORE INSERT ON covernode_msg_key_pairs
WHEN (SELECT COUNT(*) FROM covernode_msg_key_pairs WHERE epoch IS NULL) = 1
BEGIN
    SELECT RAISE(FAIL, 'Can only have one candidate messaging key pair at one time');
END;

-- there should never be more than one copy of the same key pair
CREATE UNIQUE INDEX covernode_msg_key_pairs_unique_key_pair_json ON covernode_msg_key_pairs(json_extract(key_pair_json, '$.secret_key'));
