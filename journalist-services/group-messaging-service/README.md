# group-messaging-service

Service layer for MLS-based group messaging between journalists in Sentinel.

## Responsibilities

- Registering clients and publishing key packages with the delivery service.
- Creating and modifying MLS groups.
- Sending and receiving encrypted group messages.
- Managing group membership (adding/removing members).

## Dependencies

Uses [OpenMLS](https://openmls.tech/) for the MLS protocol implementation, with SQLite-backed storage for MLS group state.

## Usage

This crate is consumed by Sentinel, integration tests, and the message canary.
