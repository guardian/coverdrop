-- Rename journalist id columns for clarity
ALTER TABLE vault_setup_bundle RENAME COLUMN pk_upload_form_json TO journalist_id_pk_upload_form_json;
ALTER TABLE vault_setup_bundle RENAME COLUMN keypair_json TO journalist_id_keypair_json;

-- Add sentinel columns
ALTER TABLE vault_setup_bundle ADD COLUMN sentinel_id_pk_upload_form_json TEXT;
ALTER TABLE vault_setup_bundle ADD COLUMN sentinel_id_keypair_json TEXT;
ALTER TABLE vault_setup_bundle ADD COLUMN register_sentinel_profile_form_json TEXT;
