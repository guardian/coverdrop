--
-- Each journalist will have a dedicated queue for outbound messages.
-- This is needed so that messages can be sent with a delay rather than
-- immediately, which allows the Signal Bridge to send cover traffic mixed
-- with messages picked from the end of the queue
--
CREATE TABLE outbound_queue(
    id      INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    message BLOB NOT NULL
);
