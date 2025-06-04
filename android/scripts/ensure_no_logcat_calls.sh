#!/bin/bash
set -e;

SCRIPT_PATH=$( cd $(dirname $0) ; pwd -P );
pushd "${SCRIPT_PATH}/..";

MATCHES=`grep -rI "Log\." app core ui 2> /dev/null | sort`;
diff scripts/ensure_no_logcat_calls_allowlist.txt <(echo "$MATCHES");

popd;
