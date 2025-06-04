# Test Vectors

This folder contains test vectors for our cryptographic primitives.
These allow to verify cross-platform and cross-version compatibility.
Everytime files in this folder are changed, we must ensure that they do not break older, live client implementations.

## Regenerating

When there are changes to our primitives or we have added new ones, the files in this directory should be re-generated.
Where possible, only commit the subsection that actually needs to change.

The files are regenerated using the `admin` CLI tool and then duplicated for the Android tests:

```shell
cargo run --bin admin generate-test-vectors
cp -rv common/tests/vectors/ android/core/src/androidTest/assets/
cp -rv common/tests/vectors/ ios/reference/CoverDropCore/Tests/CoverDropCoreTests/Resources/
```
note on OSX the `cp` required you remove the trailing slash from the source directory ie `cp -rv common/tests/vectors ios/reference/CoverDropCore/Tests/CoverDropCoreTests/Resources/`

## Testing

The test vectors are referenced by the platform-specific integration tests.
I.e. the usual `cargo test` will validate the Rust implementation against the test vectors.
