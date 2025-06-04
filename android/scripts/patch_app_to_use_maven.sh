#!/bin/bash
sed -i "s/project(\":core\")/\"com.theguardian.coverdrop:core:0.0.1\"/g" app/build.gradle
sed -i "s/project(\":ui\")/\"com.theguardian.coverdrop:ui:0.0.1\"/g" app/build.gradle

echo "app/build.gradle now looks like this:"
echo "---"
cat app/build.gradle
echo "---"
