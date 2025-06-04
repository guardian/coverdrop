-- Going to take this opportunity to make the keypairs into key_pairs like the
-- rest of the code base...
DROP INDEX journalist_id_keypairs_unqiue_pk_json;
DROP INDEX journalist_msg_keypairs_unqiue_pk_json;

ALTER TABLE journalist_id_keypairs RENAME TO journalist_id_key_pairs_old;
ALTER TABLE journalist_msg_keypairs RENAME TO journalist_msg_key_pairs_old;

--
-- Journalist identity key pairs
--

-- Because candidate identity keys are unsigned they do not have a parent relationship
-- to a journalist provisioning public key. Rather than make the foreign key nullable
-- it's easier to make a separate table to represent this difference.
CREATE TABLE candidate_journalist_id_key_pair(
    id                 INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    key_pair_json      JSONB NOT NULL,
    added_at           TEXT NOT NULL -- ISO formatted date
);

CREATE TRIGGER candidate_journalist_id_key_pair_is_unique
BEFORE INSERT ON candidate_journalist_id_key_pair
WHEN (SELECT COUNT(*) FROM candidate_journalist_id_key_pair) >= 1
BEGIN
    SELECT RAISE(FAIL, 'There can only be one candidate id key pair');
END;

CREATE TABLE journalist_id_key_pairs(
    id                 INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    provisioning_pk_id INTEGER NOT NULL,
    key_pair_json      JSONB NOT NULL,
    added_at           TEXT NOT NULL, -- ISO formatted date
    epoch              INTEGER NOT NULL,
    FOREIGN KEY (provisioning_pk_id) REFERENCES journalist_provisioning_pks(id)
);

CREATE UNIQUE INDEX journalist_id_key_pairs_unique_pk_json ON journalist_id_key_pairs(key_pair_json);

-- We can't meaningfully assign an epoch to existing keys so just set them to the highest
-- possible value (2^31 - 1).
--
-- This effectively means "can be a candidate key for all possible decryptions/verifications"
--
-- This will result in these keys being used for all decryption attempts until they expire.
INSERT INTO journalist_id_key_pairs
    SELECT
        id,
        provisioning_pk_id,
        keypair_json AS key_pair_json,
        added_at,
        2147483647 AS epoch
    FROM journalist_id_key_pairs_old;


--
-- Journalist messaging key pairs
--


CREATE TABLE journalist_msg_key_pairs(
    id             INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    id_key_pair_id INTEGER NOT NULL,
    key_pair_json  JSONB NOT NULL,
    added_at       TEXT NOT NULL, -- ISO formatted date
    epoch          INTEGER,
    FOREIGN KEY (id_key_pair_id) REFERENCES journalist_id_key_pairs(id)
);

CREATE UNIQUE INDEX journalist_msg_key_pairs_unique_pk_json ON journalist_msg_key_pairs(key_pair_json);

CREATE TRIGGER only_one_msg_candidate_key_pair
BEFORE INSERT ON journalist_msg_key_pairs
WHEN (SELECT COUNT(*) FROM journalist_msg_key_pairs WHERE epoch IS NULL) >= 1
BEGIN
    SELECT RAISE(FAIL, 'Can only have one candidate messaging key pair at one time');
END;

INSERT INTO journalist_msg_key_pairs
    SELECT
        id,
        id_keypair_id AS id_key_pair_id,
        keypair_json  AS key_pair_json,
        added_at,
        -- See the comment above for reasoning about this value.
        2147483647 AS epoch
    FROM journalist_msg_key_pairs_old;


-- Delete the old tables
DROP TABLE journalist_msg_key_pairs_old;
DROP TABLE journalist_id_key_pairs_old;
