### Integration tests

## Running locally

The integration tests use the `testcontainers` library to orchestrate the starting of Docker containers.
`testcontainers` uses unix sockets to communicate with Docker.
On MacOS the path of the unix socket is not in the standard location, so to allow these tests to run correctly you need to change the setting in your Docker desktop setup.
This is done by setting `Settings > Advanced > Allow the default Docker socket to be used` to true.

You also need to pull the postgres and varnish images with `docker pull postgres:14.5 && docker pull varnish:6.0`.
