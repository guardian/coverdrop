# Delivery Service

An MLS (Messaging Layer Security) [Delivery Service](https://datatracker.ietf.org/doc/html/draft-ietf-mls-architecture-15#name-delivery-service)
for CoverDrop's group messaging functionality. This service provides a centralized server that handles client registration, key package management, group operations, and message distribution.

## Overview

The Delivery Service is an Axum-based HTTP server with a PostgreSQL backend that enables secure group messaging using the MLS protocol. It authenticates client messages through cryptographic signatures and manages group state and message delivery.

Currently uses `JournalistIdentity` as the MLS client credential and `JournalistIdKey` for signing.

## Architecture

This service acts as the central coordination point for MLS group messaging:

- **Client Registration**: Tracks MLS clients and their identities
- **Key Package Management**: Stores and distributes key material for adding members to groups
- **Group State**: Manages group epoch state for handshake messages
- **Message Fanout**: Receives encrypted messages and queues them for delivery to recipients

All client operations require cryptographic signatures verified against identities and identity keys pulled from the public API.

