-- Rename the existing `added_at` columns to `published_at`
ALTER TABLE journalist_id_key_pairs
    RENAME COLUMN added_at TO published_at;

-- Add a new `created_at` column that tracks when the previous candidate key
-- was originally created. Its value should be the same as the existing `published_at` value.
ALTER TABLE journalist_id_key_pairs
    ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT published_at;
