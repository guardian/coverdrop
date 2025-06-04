-- Add migration to fix table journalist_id_keypairs foreign key.
-- The foreign key was pointing to the id of it's own table but 
-- it should point to the table journalist_provisioning_pks.
--
-- Since SQLite does not support ADD CONSTRAINT in the ALTER TABLE statement,
-- to modify the foreign key we have to create a new table with the correct 
-- foreign key and copy data from the original table into the new table.
--
-- The same process also needs to  be done for table journalist_msg_keypairs
-- because this table has a foreign key pointing to journalist_id_keypairs
-- so the foreign key of this table needs to be updated to point to the 
-- new table (journalist_id_keypairs)

DROP INDEX journalist_id_keypairs_unqiue_pk_json;
DROP INDEX journalist_msg_keypairs_unqiue_pk_json;

ALTER TABLE journalist_id_keypairs RENAME TO journalist_id_keypairs_old;
ALTER TABLE journalist_msg_keypairs RENAME TO journalist_msg_keypairs_old;

CREATE TABLE journalist_id_keypairs(
    id                 INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    provisioning_pk_id INTEGER NOT NULL,
    keypair_json       TEXT NOT NULL,
    added_at           TEXT NOT NULL, -- ISO formatted date
    FOREIGN KEY (provisioning_pk_id) REFERENCES journalist_provisioning_pks(id)
);

CREATE TABLE journalist_msg_keypairs(
    id            INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    id_keypair_id INTEGER NOT NULL,
    keypair_json  TEXT NOT NULL,
    added_at      TEXT NOT NULL, -- ISO formatted date
    FOREIGN KEY (id_keypair_id) REFERENCES journalist_id_keypairs(id)
);

CREATE UNIQUE INDEX journalist_id_keypairs_unqiue_pk_json ON journalist_id_keypairs(keypair_json);
CREATE UNIQUE INDEX journalist_msg_keypairs_unqiue_pk_json ON journalist_msg_keypairs(keypair_json);

INSERT INTO journalist_id_keypairs SELECT * FROM journalist_id_keypairs_old;
INSERT INTO journalist_msg_keypairs SELECT * FROM journalist_msg_keypairs_old;

DROP TABLE journalist_msg_keypairs_old;
DROP TABLE journalist_id_keypairs_old;
