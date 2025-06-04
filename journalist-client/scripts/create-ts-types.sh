#!/usr/bin/env bash

# Creates typescript models representing the responses from the Tauri API.

SCRIPT_PATH=$(
	cd "$(dirname "$0")" || exit
	pwd -P
)

# Delete the ts-rs generated files, we use the default ts-rs bindings directory and
# then copy from it so that each defined type doesn't need to define it's output dir
rm "$SCRIPT_PATH"/../src-tauri/bindings/* || true

# Delete the versions we've copied into the frontend model directory
rm -r "$SCRIPT_PATH"/../src/model/bindings

cargo test -p journalist-client export_bindings

npx prettier --write "$SCRIPT_PATH/../src-tauri/bindings/*.ts"

cp -r "$SCRIPT_PATH"/../src-tauri/bindings "$SCRIPT_PATH"/../src/model/.
