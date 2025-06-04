--  This query builds 2 CTEs, the first gets all the available keys order by epoch id
-- the keys_ordered_by_transaction_timestamp_cond get all keys ordered by transaction_commit_timestamp
-- each query then assigns each row a row number based on the ordering
-- we then do a full outer join across both results keys_ordered_by_transaction_timestamp_ts which will only return results missing from either table.
-- We expect the ordering by commit timestamp and the ordering by epoch to be the keys_ordered_by_transaction_timestamp_me
-- keys_ordered_by_transaction_timestamp_ the query keys_ordered_by_transaction_timestamp_ould return no results.

WITH all_keys AS (
    SELECT id,
        pk_json,
        xmin,
        pg_xact_commit_timestamp(xmin) as transaction_timestamp,
        epoch
    FROM organization_pks
    UNION ALL
    SELECT id,
        pk_json,
        xmin,
        pg_xact_commit_timestamp(xmin) as transaction_timestamp,
        epoch
    FROM covernode_provisioning_pks
    UNION ALL
    SELECT id,
        pk_json,
        xmin,
        pg_xact_commit_timestamp(xmin) as transaction_timestamp,
        epoch
    FROM covernode_id_pks
    UNION ALL
    SELECT id,
        pk_json,
        xmin,
        pg_xact_commit_timestamp(xmin) as transaction_timestamp,
        epoch
    FROM covernode_msg_pks
    UNION ALL
    SELECT id,
        pk_json,
        xmin,
        pg_xact_commit_timestamp(xmin) as transaction_timestamp,
        epoch
    FROM journalist_provisioning_pks
    UNION ALL
    SELECT id,
        pk_json,
        xmin,
        pg_xact_commit_timestamp(xmin) as transaction_timestamp,
        epoch
    FROM journalist_id_pks
    UNION ALL
    SELECT id,
        pk_json,
        xmin,
        pg_xact_commit_timestamp(xmin) as transaction_timestamp,
        epoch
    FROM journalist_msg_pks
),
keys_ordered_by_epoch AS (
    SELECT *,
        ROW_NUMBER() OVER (
            ORDER BY epoch
        ) AS row_num
    FROM all_keys
),
keys_ordered_by_transaction_timestamp AS (
    SELECT *,
        ROW_NUMBER() OVER (
            ORDER BY transaction_timestamp
        ) AS row_num
    FROM all_keys
)
SELECT keys_ordered_by_epoch.id AS keys_ordered_by_epoch_id,
    keys_ordered_by_epoch.row_num AS keys_ordered_by_epoch_row_num,
    keys_ordered_by_epoch.transaction_timestamp AS keys_ordered_by_epoch_transaction_timestamp,
    keys_ordered_by_epoch.epoch AS keys_ordered_by_epoch_epoch,
    keys_ordered_by_epoch.pk_json AS keys_ordered_by_epoch_pk_json,
    keys_ordered_by_transaction_timestamp.id AS keys_ordered_by_transaction_timestamp_id,
    keys_ordered_by_transaction_timestamp.row_num AS keys_ordered_by_transaction_timestamp_row_num,
    keys_ordered_by_transaction_timestamp.transaction_timestamp AS keys_ordered_by_transaction_timestamp_transaction_timestamp,
    keys_ordered_by_transaction_timestamp.epoch AS keys_ordered_by_transaction_timestamp_epoch,
    keys_ordered_by_transaction_timestamp.pk_json AS keys_ordered_by_transaction_timestamp_pk_json
FROM keys_ordered_by_epoch keys_ordered_by_epoch
    FULL OUTER JOIN keys_ordered_by_transaction_timestamp keys_ordered_by_transaction_timestamp ON keys_ordered_by_epoch.row_num = keys_ordered_by_transaction_timestamp.row_num
    AND keys_ordered_by_epoch.id = keys_ordered_by_transaction_timestamp.id
    AND keys_ordered_by_epoch.transaction_timestamp = keys_ordered_by_transaction_timestamp.transaction_timestamp
    AND keys_ordered_by_epoch.epoch = keys_ordered_by_transaction_timestamp.epoch
    AND keys_ordered_by_epoch.row_num = keys_ordered_by_transaction_timestamp.row_num
    AND keys_ordered_by_epoch.pk_json = keys_ordered_by_transaction_timestamp.pk_json
WHERE keys_ordered_by_epoch.row_num is NULL
    OR keys_ordered_by_transaction_timestamp.row_num is NULL;
