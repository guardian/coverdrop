#!/usr/bin/env bash

# Creates typescript models representing the responses from the Tauri API.

SCRIPT_PATH=$(
	cd "$(dirname "$0")" || exit
	pwd -P
)

# Delete the ts-rs generated files, we use the default ts-rs bindings directory and
# then copy from it so that each defined type doesn't need to define it's output dir
rm -r "$SCRIPT_PATH"/../src/model/bindings/

cargo test export_bindings

mv "$SCRIPT_PATH"/../src-tauri/bindings "$SCRIPT_PATH"/../src/model/
mv "$SCRIPT_PATH"/../../common/bindings/* "$SCRIPT_PATH"/../src/model/bindings/
mv "$SCRIPT_PATH"/../../journalist-vault/bindings/* "$SCRIPT_PATH"/../src/model/bindings/

npx prettier --write "$SCRIPT_PATH/../src/model/bindings/*.ts"
