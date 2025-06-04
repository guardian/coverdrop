#!/usr/bin/env bash

set -e

SCRIPT_PATH=$( cd "$(dirname "$0")" ; pwd -P )

"$SCRIPT_PATH"/api.sh
"$SCRIPT_PATH"/covernode.sh
"$SCRIPT_PATH"/identity-api.sh
