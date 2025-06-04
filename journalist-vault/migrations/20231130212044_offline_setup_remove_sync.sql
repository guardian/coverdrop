-- This migration removes the previous 'sync' strategy were new keys 
-- were generated and stored in an unsynchronised state, to be synchronised later
--
-- The idea behind this was that a vault could be created in an offline setting
-- and the keys could be uploaded later on, once a network connection is available.
-- The regular journalist identity key rotation used this mechanism too.
--
-- In reality this didn't play out very well since storing a key for rotation 
-- isn't a very useful thing outside of the initial ID key since you either end up 
-- accumulating multiple keys to rotate or you end up with an aging key being
-- stored offline.
--
-- So after this migration we store initial ID keypairs in a separate table and 
-- special case them. Regular ID key rotation will only insert the new key into
-- the vault once the PKI has confirmed it has been saved.
-- For the initial vault creation the provisioning party will also insert a 
-- copy of the journalist registration form, since this cannot be done offline.
-- The journalist will then upload this form the first time they go online.

ALTER TABLE journalist_id_keypairs DROP COLUMN synced_at;
ALTER TABLE journalist_msg_keypairs DROP COLUMN synced_at;

DROP TABLE unregistered_journalist_id_keypairs;

-- Contains forms signed by a journalist provisioning key which can be uploaded 
-- to the identity API. Can also contain a signed form to register the journalist
-- which is used only for vault creation, not subsequent re-seeding.
CREATE TABLE vault_setup_bundle(
    id                            INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    provisioning_pk_id            INTEGER NOT NULL,
    pk_upload_form_json           TEXT NOT NULL,
    keypair_json                  TEXT NOT NULL,
    register_journalist_form_json TEXT, -- Nullable, only used in the very first creation of a vault
    FOREIGN KEY (provisioning_pk_id) REFERENCES journalist_provisioning_pks(id)
);

-- Again, a slightly verbose, but robust, way of ensuring there's only one row in vault_setup_bundle.
-- We want to prevent multiple vault_setup_bundle rows because then we'd need a way of choosing which one to use.
CREATE TRIGGER vault_setup_bundle_is_unique
BEFORE INSERT ON vault_setup_bundle
WHEN (SELECT COUNT(*) FROM vault_setup_bundle) >= 1
BEGIN
    SELECT RAISE(FAIL, 'There can only be one vault setup bundle row');
END;
