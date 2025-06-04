#!/usr/bin/env bash
set -e;

SCRIPT_PATH=$( cd "$(dirname "$0")" || exit 1 ; pwd -P )

DB_PATH="${SCRIPT_PATH}/../temp-covernode.db"

pushd "${SCRIPT_PATH}/.."

    if [ -f "$DB_PATH" ];
    then
        # The temporary DB is lying around from a previous failed run
        rm "$DB_PATH"
    fi

    # It is important that we use the regular `sqlite3` application to create
    # the sqlx json since the actual database created by the code will be encrypted.
    # Rather than fuss around with trying to pass in a password to `sqlx prepare`
    # we can just use a normal, unencrypted sqlite database.

    echo "VACUUM;" | sqlite3 "$DB_PATH" 

    export DATABASE_URL="sqlite://${DB_PATH}"

    cargo sqlx migrate run
    cargo sqlx prepare -- -p covernode-database --all-features

    rm "$DB_PATH"

popd
