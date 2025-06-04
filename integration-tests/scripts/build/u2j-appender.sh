#!/usr/bin/env bash

# This script is used for building the test u2j-appender docker image locally for testing
# The container we actually use is published by a GitHub Action

set -e

SCRIPT_PATH=$( cd "$(dirname "$0")" ; pwd -P )
ROOT_PATH="$SCRIPT_PATH/../../../"
IMAGES_PATH="$SCRIPT_PATH/../../images"

docker build                                \
  --tag=test_coverdrop_u2j-appender:dev     \
  --progress plain                          \
  -f "$IMAGES_PATH/u2j-appender.Dockerfile" \
  "$ROOT_PATH"
