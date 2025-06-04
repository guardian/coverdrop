# README: CoverDrop Web API

This crate provides the REST API service executable.
It is the interface between the CoverNode and the outside world.

The REST API is defined in [docs/rest_api_specification.md](../docs/rest_api_specification.md).

## Running

Run the scripts in the [dev folder](../infra/cloud/scripts/dev/) folder to set up a local cloud stack.

Then, run the following to start the api:

```shell
$ kubectl rollout restart deployment api-deployment -n cloud
```

## Notes

In the absence of a `DATABASE_URL` environment variable, `sqlx` relies on a local JSON file (`sqlx-data.json`) to statically check the validity of the SQL syntax.
Whenever queries are modified, the following command will need to be run in order to generate new query metadata. This file needs to be checked into version control.

```shell
$ ./api/scripts/update_sqlx_json.sh
```
