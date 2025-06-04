#!/bin/bash
# This is the bootstrap script that systemd runs to start the cover traffic service
set -o errexit

# Structured JSON logs for ELK
export JSON_LOGS=1

/usr/bin/cover-traffic continuous parameter-store
