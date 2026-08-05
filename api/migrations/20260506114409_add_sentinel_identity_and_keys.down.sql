-- Revert is_valid_key_epoch to the version without sentinel_id_pks
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

DROP TRIGGER IF EXISTS set_epoch ON sentinel_id_pks;
DROP TABLE IF EXISTS sentinel_id_pks;
DROP TABLE IF EXISTS sentinel_profiles;

DROP TABLE IF EXISTS sentinel_id_pk_rotation_queue;
