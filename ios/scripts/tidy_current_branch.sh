#!/bin/bash
set -e;

if brew list swiftlint &>/dev/null; then
  brew upgrade swiftlint
else
  brew install swiftlint
fi

if brew list swiftformat &>/dev/null; then
  brew upgrade swiftformat
else
  brew install swiftformat
fi

echo "[ ] Tidying current branch."

# Navigate to the ios directory
cd "$(dirname "$0")/.."

swiftlint --config ./reference/.swiftlint.yml --fix
echo "[+] swiftlint finished."

git diff --name-only main | grep "..swift" | xargs -I % swiftformat ../% --config .swiftformat
echo "[+] swiftformat finished."
