#!/bin/bash
set -e;
echo "[ ] Tidying current branch."

# Navigate to the ios directory
cd "$(dirname "$0")/.."

swiftlint --fix
echo "[+] swiftlint finished."

git diff --name-only main | grep "..swift" | xargs -I % swiftformat ../% --config .swiftformat
echo "[+] swiftformat finished."
