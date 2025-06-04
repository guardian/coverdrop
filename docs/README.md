# Documentation

This folder contains living documentation of the CoverDrop design and implementation.
It focuses on implementation detail and architectural decisions that are not covered by the white paper.

## The protocol

We first introduce the cryptographic [primitives used in the protocol](cryptography.md).
These are based on methods from the LibSodium library and are available cross-platform.
With these we then build the [protocol messages](protocol_messages.md) that are used to communicate between the all the different components of the system.

The [key rotation](key_rotation.md) document explains how we provide forward security through key rotation for the messages sent between the source and the journalist.
As the keys propagate through the system, we need to ensure some causal ordering so that clients do not decrypt messages with keys that they do not know about.
We discuss the [key propagation](key_propagation.md) mechanism that ensures that progress in the system fulfills the causal ordering requirements.

## The client

The mobile clients (Android and iOS) store both public and private data.
Public data is accessible outside the secure session and considered non-critical, i.e. they must not reveal any information about active usage or exchanged messages.
Private data is stored inside an encrypted local storage and is only accessible when the user is logged in.
The encrypted storage is present for all users, whether they use CoverDrop or not, hence providing plausible deniability.
The [client state](client_state.md) document enumerates the different information stored on the client.
In particular, client implementation relies on two bespoke data structures: the [private sending queue](client_data_structures_and_algorithms.md#private-sending-queue) and the [encrypted local storage](client_data_structures_and_algorithms.md#encrypted-storage-with-plausible-deniability).

On both platforms we use the [Sloth](https://github.com/lambdapioneer/sloth) library for key stretching using the Secure Enclave.
For Android, we use its plausibly-deniable encryption scheme directly, while on iOS we have a custom implementation of the same scheme.
We discuss our choice of parameters for the passphrase stretching in the [client passphrase configurations](client_passphrase_configurations.md) document.

## Infrastructure

The CoverDrop infrastructure spans services hosted on third-party infrastructure (Fastly and AWS) as well as an on-premises cluster.
The [Fastly Compute@Edge](fastly_edge_service.md) service is used to route messages to the Kinesis stream.
As our configuration differs from other Guardian services, we document the specifics of our setup in the [Fastly configuration](fastly_cdn.md) document.

## Other

We also discuss how we run [integration tests](integration_tests.md) in the project and how they work in CI.

---
**Notes:** The diagrams within this documentation are created using https://asciiflow.com/legacy/ and https://app.diagrams.net/.
All are stored in the `assets` folder.
Those generated with diagrams.net include the editable `.drawio` data as part of the PNG metadata, i.e. they can be opened and edited online.