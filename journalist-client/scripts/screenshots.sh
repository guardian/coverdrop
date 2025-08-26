#! /usr/bin/env sh

# This scripts runs Playwright in a Docker container to run visual regression tests.
# This is so the generated screenshots are the same, irrespective of which machine the script is run from.
#
# Usage: ./screenshots.sh [optional playwright arguments]
# E.g.
# `./screenshots.sh` runs visual regression tests
# `./screenshots.sh --update-snapshots` updates the screenshots

set -eu

npm run storybook:build

PLAYWRIGHT_IMAGE=mcr.microsoft.com/playwright:v1.54.2-jammy

docker run \
    --rm \
    --network host \
    --volume "$(pwd)":/journalist-client/ \
    --workdir /journalist-client/ \
    --env CI \
    "$PLAYWRIGHT_IMAGE" \
    /bin/bash -c "npx playwright test --config=playwright.config.ts $@"
