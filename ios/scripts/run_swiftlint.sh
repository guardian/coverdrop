#!/bin/sh
set -e

# Downloads the portable SwiftLint release pinned in reference/.swiftlint.yml,
# verifies its checksum, and prints the path to the extracted `swiftlint`
# binary on stdout. We use the portable build (statically linked SourceKit)
# rather than Homebrew's, since the latter can crash on Xcode Cloud when its
# sourcekitdInProc doesn't match the image's Xcode version.
#
# Usage: SWIFTLINT=$(./scripts/run_swiftlint.sh)

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
SWIFTLINT_YML="$SCRIPT_DIR/../reference/.swiftlint.yml"

SWIFTLINT_VERSION=$(grep '^swiftlint_version:' "$SWIFTLINT_YML" | sed -E 's/.*: *"?([^"]+)"?/\1/')
EXPECTED_SHA256=$(grep '^swiftlint_portable_zip_sha256:' "$SWIFTLINT_YML" | sed -E 's/.*: *"?([^"]+)"?/\1/')

if [ -z "$SWIFTLINT_VERSION" ] || [ -z "$EXPECTED_SHA256" ]; then
  echo "error: swiftlint_version / swiftlint_portable_zip_sha256 missing from $SWIFTLINT_YML" >&2
  exit 1
fi

SWIFTLINT_DIR="$(mktemp -d)"
ZIP_PATH="$SWIFTLINT_DIR/portable_swiftlint.zip"

curl -sL -o "$ZIP_PATH" \
  "https://github.com/realm/SwiftLint/releases/download/${SWIFTLINT_VERSION}/portable_swiftlint.zip" >&2

ACTUAL_SHA256=$(shasum -a 256 "$ZIP_PATH" | awk '{print $1}')
if [ "$ACTUAL_SHA256" != "$EXPECTED_SHA256" ]; then
  echo "error: portable_swiftlint.zip checksum mismatch" >&2
  echo "  expected: $EXPECTED_SHA256" >&2
  echo "  actual:   $ACTUAL_SHA256" >&2
  exit 1
fi

unzip -q "$ZIP_PATH" -d "$SWIFTLINT_DIR" >&2

echo "$SWIFTLINT_DIR/swiftlint"
