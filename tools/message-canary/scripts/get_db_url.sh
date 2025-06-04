#!/usr/bin/env bash

# Script to generate a Postgres URL in the form postgresql://user:password@host/dbname
# The URL is needed so that the message canary knows which database to talk to, and with which credentials.
# This script is only used in CI, and you should not need it in local development.
# However, if you want to test it locally, you can do so by running the script
# after importing credentials from Janus (using the "Export to shell" feature)

set -o errexit
set -o pipefail

USAGE='    Usage: ./get_db_url.sh <STAGE> <DB_NAME> <DB_SECRET_ARN> [region. Default: eu-west-1]'

if [[ "${1-}" =~ ^-*h(elp)?$ ]]; then
    echo "This script generates a Postgres URL in the form postgresql://user:password@host/dbname"
    echo "It is only used in CI. If you want to test it locally, get shell credential from Janus"
    echo "$USAGE"
    exit
fi

STAGE=$1
DB_NAME=$2
DB_SECRET_ARN=$3
REGION=${4-eu-west-1}

if [[ -z "$STAGE" ]]; then
    echo "Error: you need to specify a stage."
    echo "$USAGE"
    exit 1
fi

if [[ -z "$DB_SECRET_ARN" ]]; then
    echo "Error: no database name provided".
    echo "$USAGE"
    exit 1
fi

DB_URL=$(aws secretsmanager get-secret-value        \
    --secret-id "$DB_SECRET_ARN"                    \
    --query SecretString                            \
    --output text                                   \
    --region "$REGION" |                            \
    jq -r '{ url: ("postgresql://" + .username + ":" + .password + "@" + .host + "/" + '\"$DB_NAME\"') } | .url')

echo "$DB_URL"
