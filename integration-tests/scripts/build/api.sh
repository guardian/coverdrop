#!/usr/bin/env bash

# This script is used for building the test API docker image locally for testing
# The container we actually use is published by a GitHub Action

set -e

SCRIPT_PATH=$( cd "$(dirname "$0")" ; pwd -P )
ROOT_PATH="$SCRIPT_PATH/../../../"
IMAGES_PATH="$SCRIPT_PATH/../../images"

docker build                       \
  --tag=test_coverdrop_api:dev     \
  --progress plain                 \
  -f "$IMAGES_PATH/api.Dockerfile" \
  "$ROOT_PATH"
