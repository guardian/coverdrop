#!/bin/sh
set -e

# This is run by Xcode Cloud on every CI run

brew install swiftlint
brew install swiftformat
swiftlint --strict ../../
swiftformat ../../ --config ../../.swiftformat
