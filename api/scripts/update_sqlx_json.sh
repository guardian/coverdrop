#!/usr/bin/env bash
set -e

SCRIPT_PATH=$( cd $(dirname $0) ; pwd -P )

export PGPASSWORD=coverdrop

CONTAINER_ID=$( \
docker run                             \
      -e "POSTGRES_USER=coverdrop"     \
      -e "POSTGRES_PASSWORD=coverdrop" \
      -e "POSTGRES_DB=coverdrop"       \
      -p "127.0.0.1:15432:5432"        \
      -d postgres:14.10)

function cleanup() {
    docker kill "$CONTAINER_ID"
}

trap cleanup exit
sleep 2

pushd "${SCRIPT_PATH}/.."
cargo sqlx migrate run --database-url 'postgres://coverdrop:coverdrop@127.0.0.1:15432/coverdrop'
cargo sqlx prepare --database-url 'postgres://coverdrop:coverdrop@127.0.0.1:15432/coverdrop' -- --bin api 
popd
