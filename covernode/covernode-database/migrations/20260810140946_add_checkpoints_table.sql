CREATE TABLE checkpoints (
    stream_kind      TEXT NOT NULL PRIMARY KEY,
    checkpoints_json JSONB NOT NULL
);

INSERT INTO checkpoints (stream_kind, checkpoints_json) VALUES ('user_to_journalist', '{}');
INSERT INTO checkpoints (stream_kind, checkpoints_json) VALUES ('journalist_to_user', '{}');
