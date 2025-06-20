#!/bin/bash
# Remove any existing artefacts from the `maven` directory
rm -rvf maven/com

# Build both the `core` and `ui` artefacts
./gradlew :core:clean :core:publishAllPublicationsToMavenRepository
./gradlew :ui:clean :ui:publishAllPublicationsToMavenRepository

# Copy over the generated aar files to `maven`
cp -rv core/build/repo/* maven/
cp -rv ui/build/repo/* maven/
