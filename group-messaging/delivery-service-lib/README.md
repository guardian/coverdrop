# Delivery Service Library

Client library and shared types for interacting with CoverDrop's MLS Delivery Service. This crate serves as the interface between MLS clients and the Delivery Service, providing both the API client and shared data structures.

## Overview

`delivery-service-lib` serves two purposes:

1. **Type Sharing**: Defines common data structures used by both clients and the delivery service server
2. **Client Interface**: Provides `DeliveryServiceClient` for making authenticated API calls to the delivery service

### Client

The `DeliveryServiceClient` provides methods for all delivery service operations including client registration, key package management, group operations, and message sending/receiving.

### Forms

Cryptographically signed request forms that authenticate all client operations. Each form is signed using the client's identity key to prove authenticity.
