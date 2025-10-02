CREATE TABLE backup_contacts (
    journalist_id TEXT NOT NULL
);
CREATE UNIQUE INDEX idx_backup_contacts_journalist_id ON backup_contacts(journalist_id);
