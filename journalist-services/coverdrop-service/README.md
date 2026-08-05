# coverdrop-service

Service layer for the core CoverDrop protocol operations in Sentinel.

## Responsibilities

- Pulling dead drops from the API, verifying them, decrypting messages, and storing them in the vault.
- Encrypting and sending journalist-to-user messages via the covernode.
- Generating cover traffic.
- Managing journalist identity and messaging key pair rotation.

## Usage

This crate is consumed by Sentinel, integration tests, and the message canary.
