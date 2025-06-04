The Journalist Client is a desktop app which allows a journalist to send and
receive messages to and from CoverDrop users.

### Run locally

`cd journalist-client`

`npm install`

`npm run tauri dev`

### Generate TS types

The journalist client uses [ts-rs](https://github.com/Aleph-Alpha/ts-rs) to
generate TypeScript types from Rust types.

To generate TypeScript types, run `./scripts/create-ts-types.sh`

### Manually create production release

`npm run tauri build`
