-- Tracking of the max u2j dead drop id has moved from the canary database
-- into each journalist vault. See https://github.com/guardian/coverdrop-internal/pull/3815
DROP TABLE u2j_processed_dead_drops;
