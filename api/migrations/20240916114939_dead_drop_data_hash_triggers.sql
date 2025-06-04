CREATE FUNCTION set_data_hash_trigger() RETURNS TRIGGER AS $$
BEGIN
    IF new.data_hash IS NULL THEN
        new.data_hash := digest(new.data, 'sha256');
    END IF;
    RETURN new;
END
$$ LANGUAGE plpgsql;

CREATE TRIGGER set_user_dead_drop_data_hash BEFORE INSERT OR UPDATE ON user_dead_drops 
FOR EACH ROW EXECUTE FUNCTION set_data_hash_trigger();

CREATE TRIGGER set_journalist_dead_drop_data_hash BEFORE INSERT OR UPDATE ON journalist_dead_drops 
FOR EACH ROW EXECUTE FUNCTION set_data_hash_trigger();