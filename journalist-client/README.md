The Journalist Client is a desktop app which allows a journalist to send and
receive messages to and from CoverDrop users.

### Get local setup in a state for testing against staging

#### Create profile file

Create / edit the profile file in the following location:

MacOS `~/Library/Application\ Support/com.theguardian.coverdrop-journalist-client/profiles.json`

Add the following content for staging

```
{
  "STAGING": {
    "apiUrl": "<staging-url>"
  }
}
```
And replace `<staging-url>` with the domain for the keys api (including `https://` and trailing `/`)

#### Create a journalist on staging 
Download the staging keys from s3 and set the file permissions `chmod 600 staging-keys/*`. 
You only need the journalist provisioning keys and organization keys.

`./infra/scripts/create-journalist.sh STAGING staging-keys/ "<journalist_identity>" "<description>"`

This will generate a local vault file and password file.

When running Sentinel for the first time, the app will handle the registration of your new journalist with the api server. 

### To test messaging
Create a local user, this will allow you to send messages from the cli to your new journalist.
`./infra/scripts/create-user.sh STAGING "<passphrase>"`
This will create a mailbox file locally

To send messages run 
`./infra/scripts/send-message.sh STAGING "<message>" user.mailbox "<user passphrase>" <journalist_identity>`


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
