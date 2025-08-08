ALTER TABLE u2j_messages
    ADD COLUMN custom_expiry TEXT; -- ISO formatted date or NULL;

ALTER TABLE j2u_messages
    ADD COLUMN custom_expiry TEXT; -- ISO formatted date or NULL;
