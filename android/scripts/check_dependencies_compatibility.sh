#!/usr/bin/env bash
set -e;

WORK_DIR=/tmp/coverdrop_dependency_check;
rm -rf "$WORK_DIR" || true;
mkdir -p "$WORK_DIR";

# Checkout the repositories
pushd $WORK_DIR;
echo "[ ] Cloning main-app-repository...";
git clone --depth=1 git@github.com:guardian/android-news-app.git;
popd;

# Export the relevant lines from each libs.versions.toml file
echo "[ ] Extracting critical version numbers...";
DEPS=(androidx-hilt compose-activity compose-bom compose-constraintLayout google-hilt navigation);

for dep in ${DEPS[@]}; do
    cat "gradle/libs.versions.toml" | grep -E "^$dep = \"" >> $WORK_DIR/coverdrop.deps;
    cat "$WORK_DIR/android-news-app/gradle/libs.versions.toml" | grep -E "^$dep = \"" >> $WORK_DIR/android-news-app.deps;
done;

echo "[ ] CoverDrop dependencies:";
cat $WORK_DIR/coverdrop.deps;

echo "[ ] Android-news-app dependencies:";
cat $WORK_DIR/android-news-app.deps;

echo "[ ] Looking for differences:";
diff $WORK_DIR/coverdrop.deps $WORK_DIR/android-news-app.deps;

echo "[+] No differences found";
