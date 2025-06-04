#!/bin/bash
# This is the bootstrap script that systemd runs to start the API service
set -o errexit

# Structured JSON logs for ELK
export JSON_LOGS=1

# If you update the cli args you also need to change in the
# /infra/k8s/cloud/base/u2j-appender-deployment.yaml script
/usr/bin/u2j-appender --stage $STAGE \
    --kinesis-endpoint $KINESIS_ENDPOINT \
    --kinesis-u2j-stream $KINESIS_USER_STREAM
