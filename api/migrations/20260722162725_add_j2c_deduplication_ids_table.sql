-- Keeps track of deduplication IDs for journalist-to-covernode messages to prevent
-- duplicate messages from being forwarded to the CoverNode.
-- These are deleted after 14 days, the validity period of CoverNode msg keys.
CREATE TABLE j2c_deduplication_ids (
    deduplication_id UUID PRIMARY KEY,
    created_at TIMESTAMPTZ NOT NULL
);
