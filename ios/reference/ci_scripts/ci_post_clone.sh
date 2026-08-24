#!/bin/sh
set -e

# This is run by Xcode Cloud on every CI run

# Homebrew's swiftlint links against the host Xcode's sourcekitdInProc,
# which can mismatch Xcode Cloud's image and crash (exit 132). Use the
# portable release build instead, version pinned in .swiftlint.yml.
SWIFTLINT=$(../../scripts/run_swiftlint.sh)
"$SWIFTLINT" --strict ../../

brew install swiftformat
swiftformat ../../ --config ../../.swiftformat
