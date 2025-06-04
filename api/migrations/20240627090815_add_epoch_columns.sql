ALTER TABLE organization_pks ADD COLUMN epoch INTEGER;
ALTER TABLE covernode_provisioning_pks ADD COLUMN epoch INTEGER;
ALTER TABLE covernode_id_pks ADD COLUMN epoch INTEGER;
ALTER TABLE covernode_msg_pks ADD COLUMN epoch INTEGER;
ALTER TABLE journalist_provisioning_pks ADD COLUMN epoch INTEGER;
ALTER TABLE journalist_id_pks ADD COLUMN epoch INTEGER;
ALTER TABLE journalist_msg_pks ADD COLUMN epoch INTEGER;

CREATE sequence epoch_seq;

-- Ensure that we don't introduce invalid epoch values
-- (even if the value of epoch_seq is changed manually)
CREATE OR REPLACE FUNCTION is_valid_key_epoch(potential_epoch INTEGER) 
RETURNS BOOLEAN
as $$
DECLARE
    max_epoch INTEGER;
BEGIN
    SELECT MAX(epoch) INTO max_epoch
    FROM (
        SELECT MAX(epoch) AS epoch FROM organization_pks
        UNION
        SELECT MAX(epoch) AS epoch FROM covernode_provisioning_pks
        UNION
        SELECT MAX(epoch) AS epoch FROM covernode_id_pks
        UNION
        SELECT MAX(epoch) AS epoch FROM covernode_msg_pks
        UNION
        SELECT MAX(epoch) AS epoch FROM journalist_provisioning_pks
        UNION
        SELECT MAX(epoch) AS epoch FROM journalist_id_pks
        UNION
        SELECT MAX(epoch) AS epoch FROM journalist_msg_pks
    ) x;

    RETURN potential_epoch > max_epoch;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION set_epoch()
RETURNS TRIGGER AS $$
DECLARE 
    new_epoch INTEGER;
BEGIN
    PERFORM pg_advisory_xact_lock(1);
    new_epoch := nextval('epoch_seq');
    IF NOT is_valid_key_epoch(new_epoch) THEN
      RAISE 'Duplicate epoch value %', new_epoch USING ERRCODE = 'unique_violation';
    END IF;
    NEW.epoch := new_epoch;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


-- Create indexes on epoch columns to make is_valid_key_epoch fast
CREATE UNIQUE INDEX ON organization_pks (epoch);
CREATE UNIQUE INDEX ON covernode_provisioning_pks (epoch);
CREATE UNIQUE INDEX ON covernode_id_pks (epoch);
CREATE UNIQUE INDEX ON covernode_msg_pks (epoch);
CREATE UNIQUE INDEX ON journalist_provisioning_pks (epoch);
CREATE UNIQUE INDEX ON journalist_id_pks (epoch);
CREATE UNIQUE INDEX ON journalist_msg_pks (epoch);

-- update epoch value whenever we write to a key table (including deletions)
CREATE TRIGGER set_epoch BEFORE INSERT OR UPDATE OR DELETE ON organization_pks 
    FOR EACH ROW EXECUTE FUNCTION set_epoch();
CREATE TRIGGER set_epoch BEFORE INSERT OR UPDATE OR DELETE ON covernode_provisioning_pks 
    FOR EACH ROW EXECUTE FUNCTION set_epoch();
CREATE TRIGGER set_epoch BEFORE INSERT OR UPDATE OR DELETE ON covernode_id_pks 
    FOR EACH ROW EXECUTE FUNCTION set_epoch();
CREATE TRIGGER set_epoch BEFORE INSERT OR UPDATE OR DELETE ON covernode_msg_pks 
    FOR EACH ROW EXECUTE FUNCTION set_epoch();
CREATE TRIGGER set_epoch BEFORE INSERT OR UPDATE OR DELETE ON journalist_provisioning_pks 
    FOR EACH ROW EXECUTE FUNCTION set_epoch();
CREATE TRIGGER set_epoch BEFORE INSERT OR UPDATE OR DELETE ON journalist_id_pks 
    FOR EACH ROW EXECUTE FUNCTION set_epoch();
CREATE TRIGGER set_epoch BEFORE INSERT OR UPDATE OR DELETE ON journalist_msg_pks 
    FOR EACH ROW EXECUTE FUNCTION set_epoch();

-- backfill
UPDATE organization_pks 
    SET epoch = DEFAULT; -- default value is ignored - the trigger function sets epoch equal to nextval for each row
UPDATE covernode_provisioning_pks 
    SET epoch = DEFAULT;
UPDATE covernode_id_pks 
    SET epoch = DEFAULT;
UPDATE covernode_msg_pks 
    SET epoch = DEFAULT;
UPDATE journalist_provisioning_pks 
    SET epoch = DEFAULT;
UPDATE journalist_id_pks 
    SET epoch = DEFAULT;
UPDATE journalist_msg_pks 
    SET epoch = DEFAULT;

-- set epoch not null
ALTER TABLE organization_pks 
    ALTER COLUMN epoch SET NOT NULL;
ALTER TABLE covernode_id_pks 
    ALTER COLUMN epoch SET NOT NULL;
ALTER TABLE covernode_msg_pks 
    ALTER COLUMN epoch SET NOT NULL;
ALTER TABLE journalist_provisioning_pks 
    ALTER COLUMN epoch SET NOT NULL;
ALTER TABLE journalist_id_pks 
    ALTER COLUMN epoch SET NOT NULL;
ALTER TABLE journalist_msg_pks 
    ALTER COLUMN epoch SET NOT NULL;