#!/bin/bash
# This is the bootstrap script that systemd runs to start the message canary service
set -o errexit

# Structured JSON logs for ELK
export JSON_LOGS=1

echo "Bootstrap script starting message canary"

/usr/bin/message-canary \
    --stage=$STAGE \
    --db-url=$DB_URL \
    --api-url=$API_URL \
    --messaging-url=$MESSAGING_URL \
    --vaults-path=$VAULTS_PATH \
    --mph-u2j=1 \
    --mph-j2u=1 \
    --max-delivery-time-hours 3
