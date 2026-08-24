#!/bin/bash
set -e;

# Navigate to the ios directory
cd "$(dirname "$0")/.."

SWIFTLINT=$(./scripts/run_swiftlint.sh)

if brew list swiftformat &>/dev/null; then
  brew upgrade swiftformat
else
  brew install swiftformat
fi

echo "[ ] Tidying current branch."

"$SWIFTLINT" --config ./reference/.swiftlint.yml --fix
echo "[+] swiftlint finished."

git diff --name-only main | grep -E "\.swift$" | xargs -I % swiftformat ../% --config .swiftformat
echo "[+] swiftformat finished."
