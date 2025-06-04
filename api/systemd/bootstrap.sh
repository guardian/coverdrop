#!/bin/bash
# This is the bootstrap script that systemd runs to start the API service
set -o errexit


# Structured JSON logs for ELK
export JSON_LOGS=1

# If you update the cli args you also need to change in the
# /infra/k8s/cloud/base/api-deployment.yaml script
/usr/bin/api --stage $STAGE \
    --default-journalist-id default_desk
