# Rotating public keys

In the CoverDrop protocol, when the source wants to start a new chat to a journalist (or desk) they need to know the public key of the endpoint they want to contact.
This can be achieved by maintaining a list of long-lived asymmetric keys for the different contact points within the organisations.
However, such an approach does not provide protections of future and past messages encrypted with a compromised public key (properties commonly referred to as forward security and post-compromise security, respectively).
We review in this document the considerations and steps we need to take to enable such protections in the face of our design's limitations.

## Desired properties
We consider the level of protection CoverDrop can provide against compromise of cryptographic keys, listed as a set of desired properties below:
- **Forward security** = Any messages sent in the past are safe from being read by an attacker who compromised the current keys. We consider forward security as a must-have property of CoverDrop.
- **Replay protection** = An attacker that captured traffic cannot re-send the same traffic towards the server. We consider replay protection a must-have property of CoverDrop.
- **Zero round-trip time (0-RTT)** = To enable the chosen protection and setup a secure channel we do not require any extra messages to be exchanged. We consider 0-RTT a must-have property for CoverDrop to reduce any further message delays.
- **Post-compromise security** =  Once a compromise of the keys has happened, can CoverDrop recover to a safe state? We consider post-compromise security a should-have property.

## Proposal: windowed rotated public keys
We implement windowed rotated public keys:
- The endpoint device (e.g. the journalist) generates new public keys daily with an expiration date 7*24 hours from the generation date (TTL).
- The public keys from the last 14*24 hours (the TTL of the dead drops) are delivered to all the app readers by downloading /public_keys/ once per day (e.g. on app start)
- When a source wants to start a new message thread with a journalist, their device chooses to encrypt the message contents with the public key with the longest TTL.
- Subtle detail: the client will need access to more than just the latest key for each journalist when trying to decrypt messages they receive as response since those are using [TwoPartyBox](../common/src/crypto/two_party_box.rs).
- All public keys in our scheme should be extended with a not_valid_after property that is checked together with the signature when we compute the “verified keys”.

### Advantages of the proposed approach
- Simple to implement
- 0RTT ensured since the public keys are already onto every possible source’s device; no pre-setup required
- The key is valid for a fixed period of time – 7 days – which is the period of time for which the messages encrypted with the public key are vulnerable. In reality, assuming the considerations above, new keys are published daily and the source's device looks for an optimised process by using the longest-to-live key, typically the one published that day. Therefore, we expect the worst-case scenario vulnerability-wise to be in the order of a day.
- Forward security is provided when keys rotate, only the messages sent with a compromised key will only be vulnerable, i.e. only those messages sent within the vulnerability window.
- Post-compromise security is also ensured through the key rotation, since by rotating keys we have changed the compromised pair altogether.
