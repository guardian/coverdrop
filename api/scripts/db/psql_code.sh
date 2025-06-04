#!/usr/bin/env bash

# This script connects to the CODE Postgres database.
# Janus credentials are required.

set -o errexit
set -o pipefail

DB_ARN=$(aws resourcegroupstaggingapi get-resources \
  --resource-type-filters rds:db                    \
  --tag-filters Key=Stage,Values=CODE               \
                Key=App,Values=coverdrop            \
  --region eu-west-1                                \
  --profile secure-collaboration |                  \
  jq -r '.ResourceTagMappingList[0].ResourceARN')

DB_INSTANCE=$(aws rds describe-db-instances \
  --db-instance-identifier "$DB_ARN"        \
  --region eu-west-1                        \
  --profile secure-collaboration |          \
  jq -r '.DBInstances[0]')

DB_HOSTNAME=$(echo "$DB_INSTANCE" | jq -r '.Endpoint.Address')
DB_USERNAME=$(echo "$DB_INSTANCE" | jq -r '.MasterUsername')
DB_NAME=$(echo "$DB_INSTANCE" | jq -r '.DBName')

DB_SECRET_ARN=$(aws resourcegroupstaggingapi get-resources  \
  --resource-type-filters secretsmanager                    \
  --profile secure-collaboration                            \
  --region eu-west-1 |                                      \
  jq -r  '.ResourceTagMappingList[] |
  select (.ResourceARN | contains ("DatabaseSecret")) |
  select (.Tags[].Key=="Stage" and .Tags[].Value=="CODE") |
  select (.Tags[].Key=="App" and .Tags[].Value=="coverdrop") |
  .ResourceARN')

DB_PASSWORD=$(aws secretsmanager get-secret-value \
    --secret-id "$DB_SECRET_ARN"                  \
    --query SecretString                          \
    --output text                                 \
    --profile secure-collaboration                \
    --region eu-west-1 | jq -r '.password')

SSH_COMMAND=$(ssm ssh --raw -t secure-collaboration,api,CODE --oldest --profile secure-collaboration)

# -f to run in background
# sleep 10 to give time for tunnel to be established before psql connects.
#  Once psql connects, ssh will keep the tunnel open as long as there is
# at least one open connection, so until you exit psql.
eval "${SSH_COMMAND}" -L 15432:"$DB_HOSTNAME":5432 -o ExitOnForwardFailure=yes -f sleep 10

# For some reason the terminal gets messed up after running the SSH command
reset

PGPASSWORD=$DB_PASSWORD psql -h localhost -p 15432 -U "$DB_USERNAME" -d "$DB_NAME" "$@"
