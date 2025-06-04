# CoverDrop REST API

Overview of the end-points for the REST API that serves both the client applications and the CoverNode.

The middle part of the path `.../user/...` or `.../journalist/...` refers to which side (realm) of the CN the endpoints
lie on the architectural diagram.
For example, a user client would only interact with those which are part of the `.../user/...` path group.

The API is versioned and all documented error codes are in addition to the general ones like 404 and 500 that are
handled by the framework.

# Table of contents

1. [Signed forms](#signed-forms)

2. [General endpoints](#general-endpoints)

- [GET `/v1/healthcheck`](#get-v1healthcheck)
- [GET `/v1/status`](#get-v1status)
- [POST `/v1/status`](#post-v1status)
- [POST `/v1/logging`](#post-v1logging)

3. [Public keys endpoints](#public-keys-endpoints)

- [GET `/v1/public-keys`](#get-v1public-keys)
- [POST `/v1/public-keys/journalists`](#post-v1public-keysjournalists)
- [DELETE `/v1/public-keys/journalists/delete`](#delete-v1public-keysjournalistsdelete)
- [PATCH `/v1/public-keys/journalists/update-profile`](#patch-v1public-keysjournalistsupdate-profile)
- [POST `/v1/public-keys/covernode/provisioning-public-key`](#post-v1public-keyscovernodeprovisioning-public-key)
- [POST `/v1/public-keys/covernode/identity-public-key`](#post-v1public-keyscovernodeidentity-public-key)
- [POST `/v1/public-keys/covernode/messaging-public-key`](#post-v1public-keyscovernodemessaging-public-key)
- [POST `/v1/public-keys/journalist/provisioning-public-key`](#post-v1public-keysjournalistprovisioning-public-key)
- [POST `/v1/public-keys/journalist/identity-public-key`](#post-v1public-keysjournalistidentity-public-key)
- [POST `/v1/public-keys/journalist/messaging-public-key`](#post-v1public-keysjournalistmessaging-public-key)
- [POST `/v1/status/public-key`](#post-v1statuspublic-key)

4. [User-facing endpoints](#user-facing-endpoints)

- [GET `/v1/user/dead-drops`](#get-v1userdead-dropsids_greater_thanint)
- [POST `/v1/user/dead-drops`](#post-v1userdead-drops)

5. [Journalist-facing endpoints](#journalist-facing-endpoints)

- [GET `/v1/journalist/dead-drops`](#get-v1journalistdead-dropsids_greater_thanint)
- [POST `/v1/journalist/dead-drops`](#post-v1journalistdead-drops)

## Signed forms

Any request that modifies a resource in the database must have a JSON body that conforms to the [`UnverifiedForm`](../common/src/unverified_form.rs) type.
An unverified form is a JSON object in the following shape:

```json
{
  "body": "<base64>",
  "signature": "<hex>",
  "timestamp": "<UTC timestamp>",
  "signing_pk": "<hex>"
}
```

where:

- `body` is a base64-encoded JSON object
- `signature` is the hex-encoded signature of the byte representation of `body` concatenated to the byte representation of the current timestamp
- `timestamp` is the current timestamp
- `signing_pk` is the public key that signed the body of the request

Each controller is then responsible for verifying the authenticity of the request and that the request is sufficiently fresh. The verification returns the following error codes:

#### 401 RESPONSE

```json
{
  "error": "signature verification failed"
}
```

#### 404 RESPONSE

```json
{
  "error": "Signing key not found in key repository, it is either expired or never existed"
}
```

## General endpoints

### GET `/v1/healthcheck`

Returns a simple JSON that indicates that everything is fine.
It includes commit and branch information to help debug the version that is currently deployed.
Can be used for heartbeat services.

#### REQUEST

```
<no arguments>
```

#### 200 RESPONSE

```json
{
  "name": "api",
  "status": "ok",
  "commit": "45b61edc158ac02908c88c2ce944556cc9d8f4df",
  "branch": "main"
}
```

### GET `/v1/status`

Returns the status of the service.
This endpoint is consumed by clients (iOS and Android) to determine whether to enable CoverDrop within the live app.
Possible values for the `status` field are defined [here](../common/src/api/models/general.rs). They are:

`AVAILABLE`: The system is up and running.
`UNAVAILABLE`: The system is down.
`DEGRADED_PERFORMANCE`: The system is up, but performance may be suboptimal.
`SCHEDULED_MAINTENANCE`: The system is down for planned maintenance.
`NO_INFORMATION`: There is no information about the status of the system. This happens when CoverDrop is first set up.

#### REQUEST

```
<no arguments>
```

#### 200 RESPONSE

```json
{
  "status": "AVAILABLE",
  "is_available": true,
  "description": "Up and running",
  "timestamp": "2023-11-14T13:28:28.245946Z"
}
```

### POST `/v1/status`

Updates the status of the system (see the equivalent `GET` endpoint for more information). The body of this request can be generated with the `admin` CLI. Note that the admin key pair needs to be present in the keys path for the the request to be generated correctly:

```shell
$ cargo run --bin admin -- update-system-status --keys-path ./infra/dev-keys --api-url http://127.0.0.1:55397 --status AVAILABLE --description "Up and running"
```

#### REQUEST

Sample JSON payload:

```json
{
  "body": "eyJzdGF0dXMiOnsic3RhdHVzIjoiQVZBSUxBQkxFIiwiZGVzY3JpcHRpb24iOiJVcCBhbmQgcnVubmluZyIsInRpbWVzdGFtcCI6IjIwMjQtMDEtMDRUMTc6MTk6MDEuODIwODc5WiJ9fQ",
  "signature": "c41127d0f54ea4f2432fab9e79c773d10e0b097b208b888abc3fa8d22e64b259afb064cced07f43a6cb6f781a1a3c5d1fcdb0390cb0fca45139d0bb3487c610c",
  "timestamp": "2024-01-04T17:19:01.820879Z",
  "signing_pk": "cb516f4dcf3f13def66d53fdbf9a01bbe93cd535166c7345ee2a213d8c832c90"
}
```

#### 200 RESPONSE

```
<no body>
```

#### 401 RESPONSE

See the [signed forms section](#401-response)

#### 404 RESPONSE

See the [signed forms section](#404-response)

### POST `/v1/logging`

Dynamically change the logging level of the API at runtime.
This is helpful when more detailed logs are needed to debug an issue without having to restart the application.
Calling this endpoint is done via the admin CLI. Note that the system admin key pair is needed in the `--keys-path` directory in order for the form to be generated.

```shell
cargo run --bin admin -- post-reload-logging-form --keys-path ./infra/dev-keys --api-url http://127.0.0.1:55397  --rust-log-directive debug
```

#### REQUEST

Sample JSON payload:

```json
{
  "body": "eyJzdGF0dXMiOnsic3RhdHVzIjoiQVZBSUxBQkxFIiwiZGVzY3JpcHRpb24iOiJVcCBhbmQgcnVubmluZyIsInRpbWVzdGFtcCI6IjIwMjQtMDEtMDRUMTc6MTk6MDEuODIwODc5WiJ9fQ",
  "signature": "c41127d0f54ea4f2432fab9e79c773d10e0b097b208b888abc3fa8d22e64b259afb064cced07f43a6cb6f781a1a3c5d1fcdb0390cb0fca45139d0bb3487c610c",
  "timestamp": "2024-01-04T17:19:01.820879Z",
  "signing_pk": "cb516f4dcf3f13def66d53fdbf9a01bbe93cd535166c7345ee2a213d8c832c90"
}
```

#### 200 RESPONSE

```
<no body>
```

#### 401 RESPONSE

See the [signed forms section](#401-response)

#### 404 RESPONSE

See the [signed forms section](#404-response)

#### 500 RESPONSE

```json
{
  "error": "Internal Server Error"
}
```

## Public keys endpoints

### GET `/v1/public-keys`

Returns a collection of public keys which include the organisation's public key, the CoverNode ID public key, the
CoverNode message public key, and all journalist public keys.
The endpoint also returns the journalists registered with CoverDrop as well as an optional default journalist.

#### REQUEST

```
<no arguments>
```

#### 200 RESPONSE

<details>
<summary>Click here to display a sample response</summary>

```json
{
  "journalist_profiles": [
    {
      "id": "rosalind_franklin",
      "display_name": "Rosalind Franklin",
      "sort_name": "franklin rosalind",
      "description": "Chemistry correspondent",
      "is_desk": false,
      "tag": "d6e0cc6c",
      "status": "HIDDEN_FROM_UI"
    }
  ],
  "default_journalist_id": null,
  "keys": [
    {
      "org_pk": {
        "key": "afd1a92edbafbd59547c1470ffa54ffa040ddaaa14364f0774899837819992c4",
        "certificate": "520ea2444521310bf9454fc89b0652d3d9df353aa53bc9f869506de94834355d2c6d72f9c0ee7d63070a8b1529e68e2f44e86b3916f5026a30670f70c0fd4a07",
        "not_valid_after": "2024-10-30T16:34:55.176353Z"
      },
      "covernodes": [
        {
          "provisioning_pk": {
            "key": "07fea6b015fd491993c08f1108d8df7561d426a505dbe68e60524850f897aafd",
            "certificate": "d0a410d69a7031d68cc4e11af6254249d5bcb8a3a6ed511b966f97faf2b837e78512ae00fedc7bb73ebdb33ce9b11fc38868cd523f5599519dacb6e2933fad05",
            "not_valid_after": "2024-04-17T16:37:47.346022Z"
          },
          "covernodes": {
            "covernode_001": [
              {
                "id_pk": {
                  "key": "7b8dc5c54416938aeaafb0bde1e26d282b5e94760a7d6b5c84ccbdb60fb3e3bd",
                  "certificate": "e6666476606d8cf91b9cb4adccacf4f55be9ab48a160d490d04ed0b16838e01427567fb7dbb5676c874f2ff74b55212f0e3c867a360bda43c16ebf56f7750b08",
                  "not_valid_after": "2024-01-25T18:04:30.905124191Z"
                },
                "msg_pks": [
                  {
                    "key": "eb6f6acf036b5499472120d00b1d72d006b64d1be018137b9ea594f98512ef66",
                    "certificate": "e958ae7c286a1cf7e14fc8e1c04de5f5734786395d211fea4f232531defd9624a238e75837a4d12a2fe746bf55babb88fece30ed923477b060bb6dd392589502",
                    "not_valid_after": "2024-01-19T16:14:32.472209812Z"
                  },
                  {
                    "key": "a00424589ac308b3e6d023cd2f79fb5b4e2ad1a518c032e95ab2aa279c6d7b19",
                    "certificate": "30daa5214dbc66b27edf6d48e87e333875de6d606d3623f03fad370d106563972506afc7679c2de220c5e0839779d3de7dc50f8a17c94b793569766c9c48c00f",
                    "not_valid_after": "2024-01-12T16:14:31.223201332Z"
                  }
                ]
              }
            ]
          }
        }
      ],
      "journalists": [
        {
          "provisioning_pk": {
            "key": "487c4f4194ae86c221535133b2e7efe9167b9204bdfcf2cf74bb791be891e74f",
            "certificate": "a75f2be2b24ccac37d1ea19b1c2ec49bd8b277fc159900c4b4f791b8ca60a7e0ed82e5dcdfde7fe460b44356cc1e658840cb4e74f1af137f73818d6f4e6fec0b",
            "not_valid_after": "2024-04-17T16:37:46.369032Z"
          },
          "journalists": {
            "rosalind_franklin": [
              {
                "id_pk": {
                  "key": "e2aadac1e7d7b933bd7b1ce8effd091f4a0de2b82db5280597c751f89ba8e8a4",
                  "certificate": "ee38a6f4a58c6f12e3e3a7c34d570479bff2b712a0308fe62134ce96a14704f9f5839b39c293b11f0401256e6023d2d5aef9c1e21562accce97499abf6b8df0a",
                  "not_valid_after": "2024-02-21T20:06:46.239812307Z"
                },
                "msg_pks": [
                  {
                    "key": "7c726f20cfff84f79ff8abd3febae7f7662b35d3d2a0a878a7c4492306225947",
                    "certificate": "30af4fe298f6369999bfdbc27d7435fe50d2ddc132d9f700a5f9e7a4670ed3a8b7ceb29cdd9f093f93492cc4fd7c2e1340739011557857e572ea6e267b9e970e",
                    "not_valid_after": "2024-01-12T23:57:36.912969652Z"
                  },
                  {
                    "key": "f96e48b1678464cab86b46d9f3fd161890586e8689d4f05ba300fd47be62493f",
                    "certificate": "e9f6ac7ccbe8343666c46b8721333a01a6feb0434f5e93506d69e895eabd3184409d94677975a7ad910ef77929211962a173c4a1678a3ba3dea77dd8b34ab909",
                    "not_valid_after": "2024-01-17T00:37:51.317864422Z"
                  },
                  {
                    "key": "78470dc9dcd8bf231d307ae8f211003981f36527466d3b0e6e2075cd27e37f33",
                    "certificate": "7584f1b9dce4374902c1fb98a0a54a4762824e57711314b13193ffd3f19f14e54d94b2b1fa6b6ccc8ce5ff29376ccb31a06824e04377356a1f44f4d9665d8907",
                    "not_valid_after": "2024-01-11T23:55:49.556428847Z"
                  }
                ]
              }
            ]
          }
        }
      ]
    }
  ],
  "max_epoch": 123456
}
```

</details>

### POST `/v1/public-keys/journalists`

Inserts journalist information into the database. The journalist will then show up in the [GET `/v1/public-keys`](#get-v1public-keys) endpoint.
A journalist provisioning key pair is needed to generate a valid body for this request.

#### REQUEST

```json
{
  "body": "<base64>",
  "signature": "<hex>",
  "timestamp": "2024-01-04T17:19:01.820879Z",
  "signing_pk": "<base64>"
}
```

#### 200 RESPONSE

```
<no body>
```

#### 400 RESPONSE

```json
{
  "error": "journalist description too long"
}
```

#### 401 RESPONSE

See the [signed forms section](#401-response)

#### 404 RESPONSE

See the [signed forms section](#404-response)

#### 500 RESPONSE

```json
{
  "error": "Internal Server Error"
}
```

### DELETE `/v1/public-keys/journalists/delete`

Deletes journalist information from the database.
A journalist provisioning key pair is needed to generate a valid body for this request.

#### REQUEST

```json
{
  "body": "<base64>",
  "signature": "<hex>",
  "timestamp": "2024-01-04T17:19:01.820879Z",
  "signing_pk": "<base64>"
}
```

#### 200 RESPONSE

```
<no body>
```

#### 401 RESPONSE

See the [signed forms section](#401-response)

#### 404 RESPONSE

See the [signed forms section](#404-response)

#### 500 RESPONSE

```json
{
  "error": "Internal Server Error"
}
```

### PATCH `/v1/public-keys/journalists/update-profile`

Updates journalist information.
A journalist provisioning key pair is needed to generate a valid body for this request.

#### REQUEST

```json
{
  "body": "<base64>",
  "signature": "<hex>",
  "timestamp": "2024-01-04T17:19:01.820879Z",
  "signing_pk": "<base64>"
}
```

#### 200 RESPONSE

```
<no body>
```

#### 401 RESPONSE

See the [signed forms section](#401-response)

#### 404 RESPONSE

See the [signed forms section](#404-response)

#### 500 RESPONSE

```json
{
  "error": "Internal Server Error"
}
```

### POST `/v1/public-keys/covernode/provisioning-public-key`

Inserts a CoverNode provisioning key into the database.
The organization key pair is needed in order to generate a valid body for this request.

#### REQUEST

```json
{
  "body": "<base64>",
  "signature": "<hex>",
  "timestamp": "2024-01-04T17:19:01.820879Z",
  "signing_pk": "<base64>"
}
```

#### 200 RESPONSE

```
<no body>
```

#### 400 RESPONSE

```json
{
  "error": "key has been rotated too recently"
}
```

#### 401 RESPONSE

See the [signed forms section](#401-response)

#### 404 RESPONSE

See the [signed forms section](#404-response)

#### 500 RESPONSE

```json
{
  "error": "Internal Server Error"
}
```

### POST `/v1/public-keys/covernode/identity-public-key`

Inserts a CoverNode identity key into the database.
The CoverNode provisioning key pair is needed in order to generate a valid body for this request.

#### REQUEST

```json
{
  "body": "<base64>",
  "signature": "<hex>",
  "timestamp": "2024-01-04T17:19:01.820879Z",
  "signing_pk": "<base64>"
}
```

#### 200 RESPONSE

```
<no body>
```

#### 400 RESPONSE

```json
{
  "error": "key has been rotated too recently"
}
```

#### 401 RESPONSE

See the [signed forms section](#401-response)

#### 404 RESPONSE

See the [signed forms section](#404-response)

### POST `/v1/public-keys/covernode/messaging-public-key`

Inserts a CoverNode messaging key into the database.
The CoverNode provisioning key pair is needed in order to generate a valid body for this request.

#### REQUEST

```json
{
  "body": "<base64>",
  "signature": "<hex>",
  "timestamp": "2024-01-04T17:19:01.820879Z",
  "signing_pk": "<base64>"
}
```

#### 200 RESPONSE

```
<no body>
```

#### 400 RESPONSE

```json
{
  "error": "key has been rotated too recently"
}
```

#### 401 RESPONSE

See the [signed forms section](#401-response)

#### 404 RESPONSE

See the [signed forms section](#404-response)

#### 500 RESPONSE

```json
{
  "error": "Internal Server Error"
}
```

### POST `/v1/public-keys/journalist/provisioning-public-key`

Inserts a journalist provisioning key into the database.
The organization key pair is needed in order to generate a valid body for this request.

#### REQUEST

```json
{
  "body": "<base64>",
  "signature": "<hex>",
  "timestamp": "2024-01-04T17:19:01.820879Z",
  "signing_pk": "<base64>"
}
```

#### 200 RESPONSE

```
<no body>
```

#### 400 RESPONSE

```json
{
  "error": "key has been rotated too recently"
}
```

#### 401 RESPONSE

See the [signed forms section](#401-response)

#### 404 RESPONSE

See the [signed forms section](#404-response)

#### 500 RESPONSE

```json
{
  "error": "Internal Server Error"
}
```

### POST `/v1/public-keys/journalist/identity-public-key`

Inserts a journalist identity key into the database.
The journalist provisioning key pair is needed in order to generate a valid body for this request.

#### REQUEST

```json
{
  "body": "<base64>",
  "signature": "<hex>",
  "timestamp": "2024-01-04T17:19:01.820879Z",
  "signing_pk": "<base64>"
}
```

#### 200 RESPONSE

```
<no body>
```

#### 400 RESPONSE

```json
{
  "error": "key has been rotated too recently"
}
```

#### 401 RESPONSE

See the [signed forms section](#401-response)

#### 404 RESPONSE

See the [signed forms section](#404-response)

### POST `/v1/public-keys/journalist/messaging-public-key`

Inserts a journalist messaging key into the database.
The journalist provisioning key pair is needed in order to generate a valid body for this request.

#### REQUEST

```json
{
  "body": "<base64>",
  "signature": "<hex>",
  "timestamp": "2024-01-04T17:19:01.820879Z",
  "signing_pk": "<base64>"
}
```

#### 200 RESPONSE

```
<no body>
```

#### 400 RESPONSE

```json
{
  "error": "key has been rotated too recently"
}
```

#### 401 RESPONSE

See the [signed forms section](#401-response)

#### 404 RESPONSE

See the [signed forms section](#404-response)

#### 500 RESPONSE

```json
{
  "error": "Internal Server Error"
}
```

### POST `/v1/status/public-key`

Inserts the admin key into the database.
The organization key pair is needed in order to generate a valid body for this request.

#### REQUEST

Sample JSON payload:

```json
{
  "body": "eyJzdGF0dXMiOnsic3RhdHVzIjoiQVZBSUxBQkxFIiwiZGVzY3JpcHRpb24iOiJVcCBhbmQgcnVubmluZyIsInRpbWVzdGFtcCI6IjIwMjQtMDEtMDRUMTc6MTk6MDEuODIwODc5WiJ9fQ",
  "signature": "c41127d0f54ea4f2432fab9e79c773d10e0b097b208b888abc3fa8d22e64b259afb064cced07f43a6cb6f781a1a3c5d1fcdb0390cb0fca45139d0bb3487c610c",
  "timestamp": "2024-01-04T17:19:01.820879Z",
  "signing_pk": "cb516f4dcf3f13def66d53fdbf9a01bbe93cd535166c7345ee2a213d8c832c90"
}
```

#### 200 RESPONSE

```
<no body>
```

#### 401 RESPONSE

See the [signed forms section](#401-response)

#### 404 RESPONSE

See the [signed forms section](#404-response)

## User-facing endpoints

### GET `/v1/user/dead-drops?ids_greater_than=<:int>`

Called by the user client. It returns all user-facing epochs that have ids that are greater or equal to the provided
identifier. The returned array might be empty.

#### REQUEST

```
<only query parameter>
```

#### 200 RESPONSE

```json
{
  "dead_drops": [
    {
      "id": 1,
      "created_at": "2023-12-29T15:50:55.363704Z",
      "data": "<base64>",
      "signature": "<hex>"
    },
    {
      "id": 2,
      "created_at": "2023-12-29T16:50:55.363704Z",
      "data": "<base64>",
      "signature": "<hex>"
    }
  ]
}
```

### 422 response

Deserialization errors are returned when the query parameter is malformed or missing:

```
Failed to deserialize query string: invalid digit found in string
```

```
Failed to deserialize query string: missing field `ids_greater_than`
```

### POST `/v1/user/dead-drops`

Called by the CoverNode. It adds the given dead drop to the list of user-facing dead drops.

#### REQUEST

```json
{
  "data": "<base64>",
  "cert": "<hex>"
}
```

#### 200 RESPONSE

```
<no body>
```

#### 401 RESPONSE

See the [signed forms section](#401-response)

#### 404 RESPONSE

See the [signed forms section](#404-response)

#### 500 RESPONSE

```json
{
  "error": "Internal Server Error"
}
```

## Journalist-facing endpoints

The journalist endpoints are currently mirroring the user endpoints. However, it’s worth keeping them separate as we are
likely to treat them differently.

### GET `/v1/journalist/dead-drops?ids_greater_than=<:int>`

Called by the journalist client. It returns all journalist-facing dead drops that have ids that are greater or equal to the
provided identifier. The returned array might be empty.

Response codes and messages are the same as the equivalent `user` endpoint.

### POST `/v1/journalist/dead-drops`

Called by the CoverNode. It adds the given dead drops to the list of journalist-facing dead drops.

Response codes and messages are the same as the equivalent `user` endpoint.
