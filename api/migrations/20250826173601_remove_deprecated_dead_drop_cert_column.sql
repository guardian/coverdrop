--- Removes the deprecated 'cert' column from the 'dead_drops' tables.

ALTER TABLE journalist_dead_drops
DROP COLUMN IF EXISTS cert;

ALTER TABLE user_dead_drops
DROP COLUMN IF EXISTS cert;
