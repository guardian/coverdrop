-- Add stage column to vault_info table
-- The stage is nullable initially for migration purposes.
-- When opening a vault, if the stage is NULL, it will be set to the current session's stage.
-- Subsequently, opening a vault in a different stage will be rejected.

ALTER TABLE vault_info ADD COLUMN stage TEXT;

