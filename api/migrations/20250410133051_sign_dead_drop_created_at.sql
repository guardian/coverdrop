-- Create a new column for the version 2 signature. Previous versions of the protocol considered
-- the created_at date to be "just" metadata so didn't sign it. We more recently believe
-- that it is worth signing over this field as when a message was sent/received is important
-- enough to verify.
--
-- Journalist dead drops
--
ALTER TABLE journalist_dead_drops
ADD COLUMN published_at TIMESTAMPTZ,
ADD COLUMN signature BYTEA NOT NULL DEFAULT '\x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000';

UPDATE journalist_dead_drops
SET
    published_at = created_at;

ALTER TABLE journalist_dead_drops
ALTER COLUMN published_at
SET
    NOT NULL;

--
-- User dead drops
--
ALTER TABLE user_dead_drops
ADD COLUMN published_at TIMESTAMPTZ,
ADD COLUMN signature BYTEA NOT NULL DEFAULT '\x00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000';

UPDATE user_dead_drops
SET
    published_at = created_at;

ALTER TABLE user_dead_drops
ALTER COLUMN published_at
SET
    NOT NULL;
