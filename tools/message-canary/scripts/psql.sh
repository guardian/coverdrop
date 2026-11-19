#!/usr/bin/env bash

# This script connects to the Postgres databases for the message canary.
# Janus credentials are required.

set -o errexit
set -o pipefail

STAGE=$1

if [ -z "$STAGE" ]; then
  echo "Usage: psql.sh <STAGE>"
  exit 1
fi

DB_ARN=$(aws resourcegroupstaggingapi get-resources \
  --resource-type-filters rds:db                    \
  --tag-filters Key=Stage,Values=$STAGE               \
                Key=App,Values=message-canary       \
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
  select (.Tags[].Key=="App" and .Tags[].Value=="message-canary") |
  select (.Tags[].Key=="Stage" and .Tags[].Value=='\"$STAGE\"') | .ResourceARN')

DB_PASSWORD=$(aws secretsmanager get-secret-value \
    --secret-id "$DB_SECRET_ARN"                  \
    --query SecretString                          \
    --output text                                 \
    --profile secure-collaboration                \
    --region eu-west-1 | jq -r '.password')

INSTANCE_ID=$(aws ec2 describe-instances                                      \
  --filters Name=tag:Stack,Values=secure-collaboration                        \
            Name=tag:App,Values=message-canary                                \
            Name=tag:Stage,Values="$STAGE"                                    \
            Name=instance-state-name,Values=running                           \
  --query 'sort_by(Reservations[].Instances[], &LaunchTime)[0].InstanceId'    \
  --output text                                                               \
  --region eu-west-1                                                          \
  --profile secure-collaboration)

if [ -z "$INSTANCE_ID" ] || [ "$INSTANCE_ID" = "None" ]; then
  echo "No running instance found to tunnel through"
  exit 1
fi

# Open a port forwarding session through the oldest matching instance, in the background.
# Requires the session-manager-plugin (brew install session-manager-plugin).
aws ssm start-session                                                         \
  --target "$INSTANCE_ID"                                                     \
  --document-name AWS-StartPortForwardingSessionToRemoteHost                  \
  --parameters "host=$DB_HOSTNAME,portNumber=5432,localPortNumber=15432"      \
  --region eu-west-1                                                          \
  --profile secure-collaboration > /dev/null &
SSM_PID=$!
trap 'pkill -P $SSM_PID 2> /dev/null || true; kill $SSM_PID 2> /dev/null || true' EXIT

# Wait for the tunnel to be established before psql connects.
for _ in $(seq 1 30); do
  nc -z localhost 15432 2> /dev/null && break
  sleep 1
done

PGPASSWORD=$DB_PASSWORD psql -h localhost -p 15432 -U "$DB_USERNAME" -d "$DB_NAME" "$@"
