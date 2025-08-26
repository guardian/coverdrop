The Journalist Client is a desktop app which allows a journalist to send and
receive messages to and from CoverDrop users.

### Get local setup in a state for testing against staging

#### Create profile file

Create / edit the profile file in the following location:

MacOS `~/Library/Application\ Support/com.theguardian.coverdrop-journalist-client.DEV/profiles.json`

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

## Release

The [CI job](../.github/workflows/journalist-client.yaml) (which always checks format, linting and types) will also produce a build **IF** either

- the branch is main (i.e. PROD)
- or if the workflow is manually triggered (i.e. BETA)

...and the resulting artifact is named accordingly. We don't build the app for every push to a PR as the macos GHA runner is 10x the cost and takes at least 10mins - running the GHA to produce a beta release is a easy/reasonable compromise.

### Visual regression tests

This project uses [Storybook](https://storybook.js.org/) to visually document individual components. The Storybook server can be started by running:

```bash
npm run storybook
```

To help catch visual regressions, CI is set up to validate changes for any component with an associated Story.
Components without Story files won't be tested, so to ensure regressions are caught, make sure a file like `ComponentXYZ.stories.tsx` exists alongside `ComponentXYZ.tsx`.

When you modify a component, CI will only pass if the number of pixels affected by your change stays below a certain threshold. The threshold can be updated in [`storybook.spec.ts`](./.storybook/storybook.spec.ts)
If your change exceeds the threshold, you can update the screenshots by running:

```bash
npm run storybook:update
```

This command spins up a Docker container that uses Playwright to take fresh screenshots of all components with Story files, which you will then commit to version control.
If you forget to update the screenshots when they're needed, CI will give you a reminder by failing the check. In that case, the CI job will upload an artifact containing the Playwright report, so you can inspect in detail where exactly the regression occurred.

### Skipping tests

If you do not want to run visual regression tests for a specific component, you can add the `skip` tag to the Story, like so:

```ts
export const Default: Story = {
  args: {},
  tags: ["skip"], // <--- This test will be skipped
};
```
