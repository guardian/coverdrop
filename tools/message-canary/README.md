## Overview

The Message Canary is a synthetic testing tool which sends non-cover user-to-journalist and journalist-to-user messages
through the CoverDrop system end-to-end in order to make sure that messages are
being delivered as expected.

### User to journalist messages
![Message Canary U2J](./assets/message_canary_u2j.png#gh-light-mode-only)
![Message Canary U2J](./assets/message_canary_u2j_dark.png#gh-dark-mode-only)


### Journalist to user messages
![Message Canary J2U](./assets/message_canary_j2u.png#gh-light-mode-only)
![Message Canary J2U](./assets/message_canary_j2u_dark.png#gh-dark-mode-only)

## Running stack locally

-   `docker compose up` to create db and signal cli containers
    -   the signal-cli image might need to be built with
        `./infra/k8s/on-premises/scripts/build/signal-cli.sh`

## Creating journalist

-   journalists need to be registered with the signal-bridge separately before
    being added to the message canary.
-   register the journalist with the message-canary signal cli with
    `./infra/scripts/register-signal-account.sh PROD <JOURNALIST PHONE NUMBER>
false 56780`
-   add journalist to canary db with `cargo run --bin message-canary --
--api-url=https://secure-messaging-api.guardianapis.com/
--db-url="postgresql://canary:canary@localhost:25432/canary"
--id <JOURNALIST ID> --phone-number <JOURNALIST PHONE NUMBER> --pin <SIGNAL PIN>`
-   to unregister the journalist from the signal cli, `./infra/scripts/dev-setup/on-premises/unregister-signal-account.sh <JOURNALIST PHONE NUMBER> 56780`

## Create users

`cargo run --bin message-canary --
--api-url=https://secure-messaging-api.guardianapis.com/
--db-url="postgresql://canary:canary@localhost:25432/canary"
create-users --num-users=1 --user-mailbox-dir=tools/message-canary/user-mailboxes/`

This will create `NUM_USERS` users (key pairs and mailboxes) and put them in the database

## Start the messaging canary service

`cargo run --bin message-canary -- \
--api-url=https://secure-messaging-api.guardianapis.com/ \
--db-url="postgresql://canary:canary@localhost:25432/canary" \
start --mph-u2j 1 --messaging-url=https://secure-messaging-msg.guardianapis.com`
