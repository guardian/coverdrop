-- Mark all existing messages as read.
-- This is required because we only mark messages from users as unread, and thus
-- we only ever need to move from unread -> read for a U2J message. If we defaulted to
-- all messages being unread then J2U messages would be stuck as unread forever. Or we
-- would need to implement marking J2U messages as read just for the purposes of old messages.
ALTER TABLE messages
ADD COLUMN read INTEGER NOT NULL DEFAULT 1;
