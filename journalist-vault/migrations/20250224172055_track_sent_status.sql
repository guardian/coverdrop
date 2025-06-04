-- Track the ID of the message in the outbound queue
-- if there is a message in the queue with that ID then the
-- message has yet to be sent.
--
-- Defaulting to NULL since existing messages don't match to any outbound queue id
ALTER TABLE messages
ADD COLUMN outbound_queue_id INTEGER;
