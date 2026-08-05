#!/usr/bin/env bash

# This script is used for building the varnish container locally for testing

set -e

SCRIPT_PATH=$( cd "$(dirname "$0")" ; pwd -P )
ROOT_PATH="$SCRIPT_PATH/../../../"
IMAGES_PATH="$SCRIPT_PATH/../../images"

docker build                             \
  --tag=test_coverdrop_varnish:dev       \
  --progress plain                       \
  -f "$IMAGES_PATH/varnish.Dockerfile"  \
  "$ROOT_PATH"
