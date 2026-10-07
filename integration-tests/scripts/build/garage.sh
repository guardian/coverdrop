#!/usr/bin/env bash

# This script is used for building the garage container locally for testing

set -e

SCRIPT_PATH=$( cd "$(dirname "$0")" ; pwd -P )
ROOT_PATH="$SCRIPT_PATH/../../../"
IMAGES_PATH="$SCRIPT_PATH/../../images"

docker build                           \
  --tag=test_coverdrop_garage:dev      \
  --progress plain                     \
  -f "$IMAGES_PATH/garage.Dockerfile"  \
  "$ROOT_PATH"
