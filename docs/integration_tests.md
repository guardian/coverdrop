# Integration tests

We use the [`testcontainers`](https://docs.rs/testcontainers/latest/testcontainers/) library for integration testings, which allows to easily spin up Docker containers from within our tests and removes them afterwards.

Containers need an image to run. In order to decide which image to run integration tests again, we use tags.

Containers can be built with `infra/scripts/build-all.sh` or individually using the scripts found in `infra/{cloud,on-premises}/scripts/build/*.sh`.

The `testcontainers` library provides a way to implement the `Image` trait on a container. The `name()` and `tag()` functions allows us to name and tag our images. In the example belows, the code looks for environment variables `KINESIS_IMAGE_NAME` and `KINESIS_IMAGE_TAG`. If they are not set, the tests will run against the container `coverdrop_kinesis:dev`, which corresponds to the Kinesis container built locally.

```rust
impl Image for Kinesis {
    // ...

    fn name(&self) -> String {
        env::var("KINESIS_IMAGE_NAME").unwrap_or("coverdrop_kinesis".into())
    }

    fn tag(&self) -> String {
        env::var("KINESIS_IMAGE_TAG").unwrap_or("dev".into())
    }

    fn ready_conditions(&self) -> Vec<WaitFor> {
       // ...
    }
}
```

## Continuous Integration

Because integration tests require Docker containers, building them in CI on every commit can be wasteful.
Whenever we make a change that affects containers running the test, we rebuild them and upload them to the [GitHub Container Registry](https://github.com/features/packages), and tag them with a specific hash.

The integration test [workflow](../.github/workflows/integration-tests.yaml) reconstructs the hash based on the files in the current working branch and sets the corresponding `XXX_IMAGE_NAME` and `XXX_IMAGE_TAG` environment variables.

This means that locally, tests that require the Kinesis container run against `coverdrop_kinesis:dev`, whereas in CI they run against `ghcr.io/guardian/coverdrop_kinesis:<hash>`.

## Testing new containers in CI

If you need to build new containers in CI:

1. Ensure the image of the container you want to test in CI is properly tagged with `dev` in [`docker-compose.yaml`](../docker-compose.yml)
2. Ensure the struct implementing the `Image` trait follows the same pattern in the `name` and `tag` functions (e.g. see [`kinesis.rs`](../integration-tests/src/images/kinesis.rs))
3. Add a new job for the image as well as the `XXX_IMAGE_NAME` and `XXX_IMAGE_TAG` environment variables to the [`integration-tests`](../.github/workflows/integration-tests.yaml) workflow.
4. Add a new job to the [`delete-old-packages`](../.github/workflows/delete-old-packages.yaml) workflow to delete old images from the registry. This is so we don't retain old images for longer than necessary, as storing private packages can be quite expensive.
