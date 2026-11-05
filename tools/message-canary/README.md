# Message Canary

The Message Canary is a synthetic testing tool which sends non-cover user-to-journalist (U2J) and
journalist-to-user (J2U) messages through the CoverDrop system end-to-end in order to make sure that messages are being
delivered as expected.

It runs on an EC2 instance in AWS (see [`cdk/lib/message-canary.ts`](../../cdk/lib/message-canary.ts)) and talks to the
CoverDrop protocol directly, replicating what the mobile apps and the journalist client do:

- **Synthetic users** are key pairs stored in the canary's Postgres database.
- **Synthetic journalists** are [journalist vaults](../../docs/journalist_vault.md) stored on disk (on EFS in AWS).
- Every sent message contains a random UUID. Sent and received messages are recorded in the database, so that the
  delivery time of each message can be calculated and undelivered messages can be reported as metrics.

## How it works

### User to journalist messages

A synthetic user sends a message to a random journalist via the U2J appender. The CoverNode picks the message up from
Kinesis, mixes it, and publishes it in a U2J dead drop on the API. The canary's journalist vaults pull and decrypt these
dead drops and match the received message against the database.

![Message Canary U2J message flow](./assets/message_canary_u2j.svg)

### Journalist to user messages

A synthetic journalist replies to a user that has previously written to it. The message is posted to the API, from where
it reaches the CoverNode via Kinesis. The CoverNode publishes it in a J2U dead drop on the API. The canary's users pull
and decrypt these dead drops and match the received message against the database.

![Message Canary J2U message flow](./assets/message_canary_j2u.svg)

### Background tasks

When started, the canary opens all vaults in `--vaults-path` and generates users up to `--num-users`. It then waits
10 minutes, so that a previous instance can shut down (vaults can only be opened by one process at a time), before
starting the following tasks. If any of them exits, the canary exits.

| Task                                 | What it does                                                                          |
| ------------------------------------ | ------------------------------------------------------------------------------------- |
| `send_u2j`                           | Sends a U2J message from each user to a random journalist, `--mph-u2j` times per hour |
| `send_j2u`                           | Sends a J2U message from the journalists to users that have written to them           |
| `receive_u2j`                        | Pulls and decrypts U2J dead drops using the journalist vaults                         |
| `receive_j2u`                        | Pulls J2U dead drops and tries to decrypt them for each user                          |
| `sync_journalist_provisioning_pks`   | Adds new journalist provisioning keys from the API to the vaults                      |
| `rotate_journalist_keys`             | Rotates the journalists' keys and cleans up the vaults                                |
| `create_undelivered_message_metrics` | Publishes the `UndeliveredU2JMessages` and `UndeliveredJ2UMessages` metrics           |

It also records the `U2JMessageDeliveryTime` and `J2UMessageDeliveryTime` histograms, and the `DuplicateU2JMessage` and
`DuplicateJ2UMessage` counters. A healthcheck is served on port 3050 at `/v1/healthcheck`.

## Running locally

Start the Postgres database:

```shell
docker compose up
```

The database is then available at `postgresql://canary:canary@localhost:25432/canary`. The migrations in
[`message-canary-database/migrations`](./message-canary-database/migrations) are applied when the canary starts.

## Creating journalists

The canary needs journalist vaults in `--vaults-path`. Each journalist consists of a `<journalist-id>.vault` file and a
matching `<journalist-id>.password` file. Create them with the `admin` tool, which needs the journalist provisioning key
pair for the stage:

```shell
cargo run --bin admin -- generate-journalist \
    --journalist-display-name "Canary Journalist" \
    --sentinel-display-name "Canary" \
    --sentinel-id canary \
    --description "Message canary" \
    --keys-path <PATH TO KEYS> \
    --vault-path <VAULTS PATH> \
    --stage <STAGE>
```

When the canary first opens a new vault it processes the vault setup bundle (which publishes the journalist's keys to the
API) and then waits a minute for the journalist to appear in the API.

## Running the canary

```shell
cargo run --bin message-canary -- \
    --stage <STAGE> \
    --db-url postgresql://canary:canary@localhost:25432/canary \
    --api-url <API URL> \
    --messaging-url <U2J APPENDER URL> \
    --vaults-path <VAULTS PATH> \
    --mph-u2j 1 \
    --mph-j2u 1 \
    --max-delivery-time-hours 3
```

Missing users are generated and stored in the database on start up. Messages that have not been received after
`--max-delivery-time-hours` are reported as undelivered. In AWS the canary is started by systemd using
[`bootstrap.sh`](./systemd/bootstrap.sh).

To connect to the canary's database in AWS use [`scripts/psql.sh`](./scripts/psql.sh) (requires Janus credentials).
