#!/usr/bin/env bash

# Script to generate a Postgres URL in the form postgresql://user:password@host/dbname
# The URL is needed so that the API knows which database to talk to, and with which credentials.
# This script is only used in CI, and you should not need it in local development.
# However, if you want to test it locally, you can do so by running the script
# after importing credentials from Janus (using the "Export to shell" feature)

set -o errexit
set -o pipefail

USAGE='    Usage: ./get_db_url.sh <STAGE> <DB_NAME> [region. Default: eu-west-1]'

if [[ "${1-}" =~ ^-*h(elp)?$ ]]; then
    echo "This script generates a Postgres URL in the form postgresql://user:password@host/dbname"
    echo "It is only used in CI. If you want to test it locally, get shell credential from Janus"
    echo "$USAGE"
    exit
fi

STAGE=$1
DB_NAME=$2
REGION=${3-eu-west-1}

if [[ -z "$STAGE" ]]; then
    echo "Error: you need to specify a stage."
    echo "$USAGE"
    exit 1
fi

if [[ -z "$DB_NAME" ]]; then
    echo "Error: no database name provided".
    echo "$USAGE"
    exit 1
fi

DB_SECRET_ARN=$(aws resourcegroupstaggingapi get-resources          \
    --resource-type-filters secretsmanager                          \
    --region "$REGION" |                                            \
    jq -r  '.ResourceTagMappingList[] | 
    select (.ResourceARN | contains ("DatabaseSec")) |
    select (.Tags[].Key=="Stage" and .Tags[].Value=='\"$STAGE\"') |
    .ResourceARN')

if [[ -z "$DB_SECRET_ARN" ]]; then
    echo "Could not find a database secret for stage \"$STAGE\" in region \"$REGION\""
    exit 1
fi

DB_URL=$(aws secretsmanager get-secret-value        \
    --secret-id "$DB_SECRET_ARN"                    \
    --query SecretString                            \
    --output text                                   \
    --region "$REGION" |                            \
    jq -r '{ url: ("postgresql://" + .username + ":" + .password + "@" + .host + "/" + '\"$DB_NAME\"') } | .url')

echo "$DB_URL"
