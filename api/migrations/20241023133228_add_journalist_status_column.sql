CREATE TABLE journalist_statuses (
    id INTEGER PRIMARY KEY GENERATED ALWAYS AS IDENTITY,
    status TEXT UNIQUE NOT NULL
);

INSERT INTO journalist_statuses (status)
VALUES 
    ('VISIBLE'),
    ('HIDDEN_FROM_UI'),
    ('HIDDEN_FROM_RESPONSE');

ALTER TABLE journalist_profiles
ADD COLUMN status_id INTEGER REFERENCES journalist_statuses(id);

UPDATE journalist_profiles SET status_id = (
    SELECT id FROM journalist_statuses WHERE status = 'VISIBLE'
);

ALTER TABLE journalist_profiles
ALTER COLUMN status_id SET NOT NULL;
