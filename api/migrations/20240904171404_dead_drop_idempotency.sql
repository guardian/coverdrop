-- Connection issues might result in the CoverNode submitting the same dead drop to the API multiple times
-- for example if the POST request is accepted but for some reason we lose the response from the API to the CoverNode.
--
-- The dead drops don't yet have an ID that the CoverNode can use so we can't deduplicate on that. Instead we deduplicate
-- on the content of the dead drop by hashing messages. This means the CoverNode can happily resubmit the same dead drop
-- over and over and the API will not add it to the table containing all dead drops.
--
-- If the CoverNode suffers a crash after a dead drop has been added but without getting confirmation then it may submit
-- a new dead drop with some of the same messages, but a different overall set of messages. This would result in
-- both dead drops being comitted and multiples of the same messages being in the system.
--
-- This is an annoying but acceptable failure mode.

CREATE EXTENSION pgcrypto;

-- User dead drops
ALTER TABLE user_dead_drops ADD COLUMN data_hash TEXT;
CREATE UNIQUE INDEX user_dead_drop_data_hash_idx ON user_dead_drops(data_hash);

UPDATE user_dead_drops SET data_hash = digest(data, 'sha256');
ALTER TABLE user_dead_drops ALTER COLUMN data_hash SET NOT NULL;

-- Journalist dead drops
ALTER TABLE journalist_dead_drops ADD COLUMN data_hash TEXT;
CREATE UNIQUE INDEX journalist_dead_drop_data_hash_idx ON journalist_dead_drops(data_hash);

UPDATE journalist_dead_drops SET data_hash = digest(data, 'sha256');
ALTER TABLE journalist_dead_drops ALTER COLUMN data_hash SET NOT NULL;
