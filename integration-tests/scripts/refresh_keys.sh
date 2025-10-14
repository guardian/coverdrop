#!/usr/bin/env bash

set -e
set -x

# This script regenerates the keys used by the integration tests
#
# You should only have to do this if the *format* of the keys change.
#
# You should never have to run this script due to a key expiring.
# The integration tests are supposed to manage time correctly.

SCRIPT_PATH=$(
	cd $(dirname $0)
	pwd -P
)

KEYS_PATH="${SCRIPT_PATH}/../keys"

# This URL is not actually used because of the `--do-not-upload-to-api` flags below
# but it is required to be set.
API_URL="http://fake-api-path"

pushd "${SCRIPT_PATH}/../.."

ls "${KEYS_PATH}"/*.json | grep -v user | xargs rm
rm "${KEYS_PATH}/keys_generated_at.txt" || true

date -u +"%Y-%m-%dT%H:%M:%SZ" >"${KEYS_PATH}/keys_generated_at.txt"

cargo run --quiet --bin admin -- generate-organization-key-pair --keys-path "$KEYS_PATH"
cargo run --quiet --bin admin -- generate-journalist-provisioning-key-pair --keys-path "$KEYS_PATH" --api-url "$API_URL" --do-not-upload-to-api
cargo run --quiet --bin admin -- generate-covernode-provisioning-key-pair --keys-path "$KEYS_PATH" --api-url "$API_URL" --do-not-upload-to-api
cargo run --quiet --bin admin -- generate-covernode-identity-key-pair --covernode-id covernode_001 --keys-path "$KEYS_PATH" --api-url "$API_URL" --do-not-upload-to-api
cargo run --quiet --bin admin -- generate-covernode-messaging-key-pair --keys-path "$KEYS_PATH" --api-url "$API_URL" --do-not-upload-to-api
cargo run --quiet --bin admin -- generate-backup-identity-key-pair --keys-path "$KEYS_PATH"
cargo run --quiet --bin admin -- generate-backup-messaging-key-pair --keys-path "$KEYS_PATH"
cargo run --quiet --bin admin -- generate-admin-key-pair --keys-path "$KEYS_PATH" --api-url "$API_URL" --do-not-upload-to-api

# Under normal conditions we don't want the journalist key pair to ever be available
# on-disk so we create it in an encrypted vault. Because of this, we need to jump through
# some hoops to extract the keys from this vault for integration testing.

export RUST_LOG=info

function get_key {
	local KEY_NAME=$1
	local TABLE_NAME=$2
	local ENTITY_NAME=$3

	local KEYPAIR=$(cargo run -q --bin coverup journalist-vault execute-vault-query \
		--vault-path "${KEYS_PATH}/static_test_journalist.vault" \
		--password-path "${KEYS_PATH}/static_test_journalist.password" \
		--sql-query "SELECT $KEY_NAME FROM $TABLE_NAME;")

	# The '8' here is also used in `common/src/crypto/keys/serde.rs`
	local PUBLIC_KEY_PREFIX=$(echo $KEYPAIR | jq -r .public_key.key | head -c 8)

	echo $KEYPAIR > "$KEYS_PATH/$ENTITY_NAME-$PUBLIC_KEY_PREFIX.keypair.json"
	chmod 600 "$KEYS_PATH/$ENTITY_NAME-$PUBLIC_KEY_PREFIX.keypair.json"
}

# Delete the vault if it currently exists
rm "$KEYS_PATH/static_test_journalist.vault" || true
rm "$KEYS_PATH/static_test_journalist.password" || true


cargo run --quiet --bin admin -- generate-journalist \
	--keys-path "$KEYS_PATH" \
	--vault-path "$KEYS_PATH" \
	--sort-name 'journalist static test' \
	--display-name 'static test journalist' \
	--description 'test journalist description'

PASSWORD=$(cat "$KEYS_PATH/static_test_journalist.password")

get_key 'keypair_json' 'vault_setup_bundle' 'journalist_id'

cargo run --features "integration-tests" --quiet --bin admin -- generate-journalist-messaging-keys-for-integration-test --keys-path "$KEYS_PATH"

rm "$KEYS_PATH/static_test_journalist.vault"

popd
