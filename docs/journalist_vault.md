# Journalist Vault

Journalist data is stored in a secure "vault", which is a SQLCipher database. SQLCipher is an extension of SQLite which encrypts data before writing it to disk.

Data stored in the vault includes:

- Trusted root organization keys
- Messages to and from the journalist
- Key pairs used for signatures and encryption

## Vault setup bundle

We want to be able to create and manipulate vaults in an offline context so that we can create them without exposing secret key material.

The main time this is important is during the initial creation of the journalist vault by a member of editorial staff. The member of
staff who is creating the journalist's vaults should not be expected to be a security expert and to know how to safely manage secret keys.
To make things easier, we would like to provide a laptop without a WiFi adapter or other way of connecting to the internet to prevent accidental
disclosures.

Since the laptop is offline, it won't be able to upload the public keys to the API synchronously with the vaults creation.

To get around this, when we first create a vault, a setup bundle is included within it. The journalist's client tools will periodically
poll the vaults, and if a setup bundle is detected, the vault is bootstrapped and the journalist information and keys are posted to the API. When the process is complete, the setup bundle is deleted from the vault.

## Journalist Vault Spec

### Creation

Journalists vaults are created by a member of staff with access to a valid journalist provisioning key pair.

The vault is created against a particular key hierarchy and is seeded with a “preregistered” identity key for the journalist. Subsequent identity keys can be rotated by the journalist.

Since the creation of the vault is performed offline so as to not expose the journalist provisioning key pair, the creation ceremony will also emit a form to register the initial identity key with the CoverDrop API.

### Adding a new identity key pair

When a journalist wants to rotate their identity key they generate a new key, and post it to the identity API.
The identity API will return a signed version of the public key to the client which will then save it to the vault.

Previously we stored the key pair locally before syncing to the API, but this caused an unnecessary amount of state management.

### Adding a new messaging key pair

When a journalist wants to rotate their messaging key it’s very similar to when they rotate an identity key.

### Debugging the vault

Each journalist vault is a separate SQLCipher database file. SQLCipher is an open-source extension to SQLite that adds 256-bit AES encryption to a database. It works by transparently encrypting the database pages as they’re sent to the disk. This allows the client applications to write normal SQL queries with little thought to the encryption other than setting the key when the database is opened. The key is generated upon creating the vault.

Vaults are marked with the `.vault` extension.
Reading the content of a `.vault` file is possible by using the sqlcipher [command line tool](https://www.zetetic.net/sqlcipher/). We recommend using version `3.41.2 2023-03-22 11:56:21 (SQLCipher 4.5.4 community)` (hash: `0d1fc92f94cb6b76bffe3ec34d69cffde2924203304e8ffc4155597af0c1alt1`) which guarantees encryption/decryption is performed with the same number of iterations.
We cannot guarantee that vaults encrypted with a specific version of SQLcipher can be decrypted with an older/newer version.

The syntax for opening a vault file is

```shell
$ sqlcipher /path/to/file.vault

$ sqlcipher journalist.vault
SQLite version 3.41.2 2023-03-22 11:56:21 (SQLCipher 4.5.4 community)
Enter ".help" for usage hints.
sqlite>
```

Because each vault is encrypted, a password (or key, to use SQLcipher terminology) is needed to decrypt it. This is normally stored in a `.password` file upon vault creation. The password file is a simple txt file with the passphrase stored in cleartext.
In order to avoid plaintext `.password` files in the file system we recommend a journalist rekeys their vault.
By default sqlcipher uses a weak password-based key derivation function so we
use a stronger Argon2 configuration instead. In order to manually open a
journalist vault, the Argon2 key needs to be derived. You can open a vault using
the following coverup command which will derive the key, shell into sqlcipher,
and decrypt the vault with a `pragma` statement:

```shell
cargo run --bin coverup journalist-vault open-vault --vault-path default_journalist.vault --password-path default_journalist.password
```

Once the key PRAGMA has been set, sqlcipher will print out ok, signalling that SQL queries can now be run. E.g.

```shell
sqlite> SELECT journalist_id FROM vault_info;
mario_savarese
```

To enable headers, type in:

```shell
sqlite> .headers on
```

This will display column headers, which can be helpful when the query returns several columns.

```shell
sqlite> SELECT journalist_id FROM vault_info;
journalist_id
mario_savarese
```

To see all tables stored in the vault, you can run the following:

```shell
sqlite> .tables
```
