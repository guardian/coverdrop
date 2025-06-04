ALTER TABLE journalists
DROP COLUMN phone_number;

ALTER TABLE journalists
DROP COLUMN pin;

ALTER TABLE journalists
ADD COLUMN vault_path TEXT NOT NULL;
