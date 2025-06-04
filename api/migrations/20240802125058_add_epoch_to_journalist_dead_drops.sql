ALTER TABLE journalist_dead_drops ADD COLUMN epoch INTEGER;

-- Increment the sequence to initialize the sequence in case it has yet to be used.
SELECT nextval('epoch_seq');

UPDATE journalist_dead_drops SET epoch = currval('epoch_seq');

ALTER TABLE journalist_dead_drops ALTER COLUMN epoch SET NOT NULL;
