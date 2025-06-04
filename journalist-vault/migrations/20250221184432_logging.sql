CREATE TABLE sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    session_started_at TEXT NOT NULL -- ISO formatted timestamp
);

CREATE TABLE log_entries (
    session_id INTEGER NOT NULL, -- Used to disciminate between different usages of the vault
    timestamp TEXT NOT NULL, -- ISO formatted timestamp
    level TEXT NOT NULL, -- Log level: INFO, ERROR, DEBUG, etc.
    target TEXT NOT NULL, -- The module that emitted the log message
    message TEXT NOT NULL, -- Message text
    FOREIGN KEY (session_id) REFERENCES sessions (id)
);

CREATE INDEX idx_log_entries_timestamp ON log_entries (timestamp);
