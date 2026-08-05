ALTER TABLE outbound_queue
ADD COLUMN deduplication_id TEXT; -- UUID
