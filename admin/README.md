## Admin CLI

A small CLI tool to generate various keys.

From the root folder, run `cargo run --bin admin -- <SUBCOMMAND>`

For example, to generate the organization key pair, run `cargo run --bin admin -- generate-organization-keys`

```
USAGE:
    admin <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    generate-covernode-keys
            Generate random keys for test purposes
    generate-journalist
            Generate keys for journalists or desks
    generate-mobile-constants-files
            Generates files for the mobile apps that contain the main constants from the common Rust
            files
    generate-organization-keys
            Generate the top-level key pair which will be used to sign all the other keys
    generate-test-vectors
            Generate test vectors for the currently compiled version. These are used for ensuring
            cross-platform and cross-version compatibility
    help
            Print this message or the help of the given subcommand(s)
```
