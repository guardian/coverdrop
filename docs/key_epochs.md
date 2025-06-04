### Key Epochs

In order the solve the issues outlined in the [key propagation docs](https://github.com/guardian/coverdrop/blob/main/docs/key_propagation.md), we needed to add a monotonically increasing epoch value to each key. We decided that the best approach to doing this was to add a new epoch column to each key tables in the api database (see [migration](https://github.com/guardian/coverdrop/blob/main/api/migrations/20240627090815_add_epoch_columns.sql) for schema changes ).
To enforce correctness we have a function `is_valid_key_epoch` which checks uniqueness of new epochs across all key tables. This stops someone accidentally rolling back the epoch sequence manually.

To generate a new epoch when a key is added, updated or deleted, we added a new trigger function `set_epoch` to each of the keys tables. This function uses a `pg_advisory_xact_lock` to block any other transactions that also call set_epoch.

This means that modification to any of the key tables cannot be done in parallel, as each modification sets the shared advisory lock, and guarantees that our epoch values are always consistent with the order in which the records were inserted / modified in the database and avoids the `2 before 1` problem described in this article https://mattjames.dev/auto-increment-ids-are-not-strictly-monotonic/
