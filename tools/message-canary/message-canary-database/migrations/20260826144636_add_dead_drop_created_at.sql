ALTER TABLE j2u_processed_dead_drops
DROP COLUMN dead_drop_id,
ADD COLUMN dead_drop_created_at TIMESTAMPTZ;

-- Backfill with processed_at as best approximation of the dead drop's creation time
UPDATE j2u_processed_dead_drops SET dead_drop_created_at = processed_at;

ALTER TABLE j2u_processed_dead_drops ALTER COLUMN dead_drop_created_at SET NOT NULL;
