#!/usr/bin/env bash
set -e;

SCRIPT_PATH=$( cd $(dirname $0) ; pwd -P )
ROOT_DIRECTORY="$SCRIPT_PATH/../../"

COVER_DROP_PROPERTIES_PATH="${SCRIPT_PATH}/../coverdrop-dev.properties"

pushd "${SCRIPT_PATH}/.."

if [ -f "${COVER_DROP_PROPERTIES_PATH}" ];
then
    rm "${COVER_DROP_PROPERTIES_PATH}"
fi

echo "# This file is generated using scripts/create_coverdrop_properties.sh" > "${COVER_DROP_PROPERTIES_PATH}";

SCRIPT_PATH=$( cd "$(dirname "$0")" || exit ; pwd -P )

NODE1_IP_ADDR=$("${ROOT_DIRECTORY}"/infra/multipass/scripts/get_node_ip_addr.sh coverdrop-node1)

API_URL="http://${NODE1_IP_ADDR}:30000"
MSG_URL="http://${NODE1_IP_ADDR}:30767"

# Set local IP that maps to the docker containers as base url
echo "coverdrop.local_test.api_base_url=\"${API_URL}\"" >> "${COVER_DROP_PROPERTIES_PATH}";
echo "coverdrop.local_test.messaging_base_url=\"${MSG_URL}\"" >> "${COVER_DROP_PROPERTIES_PATH}";

# Set local organisation keys used with the docker containers as a trusted public key
ORG_PK=$(cat "${ROOT_DIRECTORY}/infra/multipass/keys/org*.json" | jq ".key" -r)
 
echo "coverdrop.local_test.trusted_org_pks=\"${ORG_PK}\"" >> "${COVER_DROP_PROPERTIES_PATH}";

# By default always disable local test mode (can be overwritten later manually by the developer)
echo "coverdrop.local_test_mode_enabled=true" >> "${COVER_DROP_PROPERTIES_PATH}";

popd
