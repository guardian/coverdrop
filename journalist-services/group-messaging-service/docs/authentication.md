# OpenMLS Authentication: Signing and Verification

This document traces how OpenMLS signs and verifies three core MLS types: **Key Packages**, **Welcome Messages** (via GroupInfo), and **Application Messages** (via FramedContent). All three use the same underlying `Signable`/`Verifiable` trait system defined in [`signable.rs`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/ciphersuite/signable.rs).

All references link to commit [`195f841`](https://github.com/openmls/openmls/tree/195f841c378807254efe6c93d0b82e9ad13f8577) of the OpenMLS repository.

---

## Common Signing Infrastructure

OpenMLS uses a trait-based type-state pattern for signing and verification:

- **`Signable`** — implemented by TBS (to-be-signed) structs. Provides `unsigned_payload()` and a `label()` string. The default [`Signable::sign()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/ciphersuite/signable.rs#L71) method serializes the payload into a `SignContent` with the label, then calls the `Signer` to produce a signature.
- **`Verifiable`** — implemented by incoming (unverified) structs. Provides `unsigned_payload()`, `signature()`, and `label()`. The default [`Verifiable::verify_no_out()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/ciphersuite/signable.rs#L144) method re-serializes the payload as `SignContent` with the label and calls `crypto.verify_signature()`.

The `SignContent` wraps a label + payload for domain separation, matching the MLS spec's `SignWithLabel` construction.

In the context of group messaging in Sentinel, the client's credential is their `SentinelIdentity` and their `Signer` is their latest `SentinelIdKeyPair`.

---

## 1. Key Packages

### Signed payload

`KeyPackageTBS` — the unsigned key package payload containing protocol version, ciphersuite, init key, leaf node, and extensions.

**Label:** `"KeyPackageTBS"`

### Signing

[`KeyPackageTbs` implements `Signable`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/key_packages/mod.rs#L157) with label [`"KeyPackageTBS"`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/key_packages/mod.rs#L220).

Signing occurs in `KeyPackage::new_from_keys()`, which builds a `KeyPackageTbs` and calls [`key_package_tbs.sign(signer)`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/key_packages/mod.rs#L368).

This calls the generic `Signable::sign()` which:
1. Serializes the `KeyPackageTbs` via `unsigned_payload()` (TLS serialization)
2. Wraps it in `SignContent` with label `"KeyPackageTBS"`
3. TLS-serializes the `SignContent` and passes it to `signer.sign()`
4. Produces a `KeyPackage` (the `SignedStruct`)

### Verification

[`VerifiableKeyPackage` implements `Verifiable`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/key_packages/key_package_in.rs#L36) with the same label `"KeyPackageTBS"`.

Verification is triggered by [`KeyPackageIn::validate()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/key_packages/key_package_in.rs#L136):

The `validate()` method:
1. First verifies the **leaf node** signature inside the key package ([valn0108](https://validation.openmls.tech/#valn0108))
2. Checks protocol version ([valn0201](https://validation.openmls.tech/#valn0201)) and that init key ≠ encryption key ([valn0204](https://validation.openmls.tech/#valn0204))
3. Creates a `VerifiableKeyPackage` and calls `.verify(crypto, signature_key)` ([valn0203](https://validation.openmls.tech/#valn0203))
4. The signing public key is extracted from the key package's own leaf node

`verify()` calls [`verify_no_out()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/ciphersuite/signable.rs#L144) which:
1. TLS-serializes the `KeyPackageTbs` payload
2. Wraps it in `SignContent` with label `"KeyPackageTBS"`
3. Calls `crypto.verify_signature()` against the leaf node's signature key

---

## 2. Welcome Messages

### Structure

A [`Welcome`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/messages/mod.rs#L73) message contains three fields:

```rust
pub struct Welcome {
    cipher_suite: Ciphersuite,
    secrets: Vec<EncryptedGroupSecrets>,
    encrypted_group_info: VLBytes,
}
```

- **`encrypted_group_info`** — a signed `GroupInfo` symmetrically encrypted with a key derived from the welcome secret
- **`secrets`** — a list of [`EncryptedGroupSecrets`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/messages/mod.rs#L140), one per invited member, each containing:
  - `new_member` — a `KeyPackageRef` identifying which member the secret is for
  - `encrypted_group_secrets` — the `GroupSecrets` (joiner secret, optional path secret, PSKs) encrypted via HPKE to that member's init key

Both components are authenticated, but through different mechanisms: `GroupInfo` uses an asymmetric signature, while `EncryptedGroupSecrets` are bound to the `encrypted_group_info` via HPKE.

### GroupInfo: Asymmetric Signature

#### Signed payload

[`GroupInfoTBS`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/messages/group_info.rs#L249) — contains the group context, extensions, confirmation tag, and signer leaf index.


#### Signing

[`GroupInfoTBS` implements `Signable`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/messages/group_info.rs#L291) with label [`"GroupInfoTBS"`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/messages/group_info.rs#L24).

Signing occurs during commit creation when a Welcome is needed. In [`CommitBuilder::build()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/mls_group/commit_builder.rs#L420):
1. A `GroupInfoTBS` is [constructed](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/mls_group/commit_builder.rs#L775) with the new group context, extensions, confirmation tag, and the committer's leaf index
2. [`group_info_tbs.sign(old_signer)`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/mls_group/commit_builder.rs#L783) is called to produce a signed `GroupInfo`
3. The signed `GroupInfo` is [symmetrically encrypted](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/mls_group/commit_builder.rs#L789) with the welcome key/nonce to produce `encrypted_group_info`

#### Verification

[`VerifiableGroupInfo` implements `Verifiable`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/messages/group_info.rs#L317) with the same label `"GroupInfoTBS"`.

Verification occurs during Welcome processing in `PublicGroup::from_ratchet_tree()`:
1. The Welcome is decrypted to obtain a `VerifiableGroupInfo` via [`VerifiableGroupInfo::try_from_ciphertext()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/mls_group/creation.rs#L202)
2. The ratchet tree is reconstructed, and the signer's leaf node is looked up
3. The signer's signature public key is extracted from the tree
4. [`verifiable_group_info.verify()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/public_group/mod.rs#L251) is called ([valn1402](https://validation.openmls.tech/#valn1402))

`verify()` calls `verify_no_out()` which:
1. TLS-serializes the `GroupInfoTBS` payload
2. Wraps it in `SignContent` with label `"GroupInfoTBS"`
3. Calls `crypto.verify_signature()` against the signer's signature key from the ratchet tree

### EncryptedGroupSecrets: HPKE Binding

The `EncryptedGroupSecrets` are not signed, but are cryptographically bound to the `encrypted_group_info` via HPKE's `EncryptWithLabel` construction.

#### Encryption

Each invited member's `GroupSecrets` are encrypted in [`TreeSyncDiff::encrypt_group_secrets()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/treesync/treekem.rs#L168):

```
EncryptWithLabel(KeyPackage.init_key, "Welcome", encrypted_group_info, GroupSecrets)
```

[`hpke::encrypt_with_label()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/ciphersuite/hpke.rs#L119) constructs an [`EncryptContext`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/ciphersuite/hpke.rs#L77) from the label `"MLS 1.0 Welcome"` and the `encrypted_group_info` bytes. This context is passed as the HPKE `info` parameter to `SealBase`, meaning it is mixed into the HPKE key schedule. This binds the encrypted group secrets to the exact `encrypted_group_info` ciphertext — if either is tampered with, decryption will fail.

The result is wrapped in an [`EncryptedGroupSecrets`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/treesync/treekem.rs#L220) along with the member's `KeyPackageRef`, and the full set is then [assembled into a `Welcome`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/mls_group/commit_builder.rs#L815).

#### Decryption

During Welcome processing, [`GroupSecrets::try_from_ciphertext()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/messages/mod.rs#L480) calls [`hpke::decrypt_with_label()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/messages/mod.rs#L488) with the same label `"Welcome"` and [`welcome.encrypted_group_info()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/mls_group/creation.rs#L174) as the context, using the member's HPKE init private key. If the `encrypted_group_info` has been modified, the HPKE key schedule will derive different keys and decryption will fail.

---

## 3. Application Messages (FramedContent)

Application messages, proposals, and commits all use the same signing mechanism: they are wrapped in `FramedContent`, signed as `FramedContentTBS`, and transmitted as `AuthenticatedContent` (either encrypted as `PrivateMessage` or as `PublicMessage`).

### Signed payload

`FramedContentTbs` — contains the protocol version, wire format, the `FramedContent` (group ID, epoch, sender, authenticated data, and message body), and optionally a serialized group context.

**Label:** `"FramedContentTBS"`

### Signing

[`FramedContentTbs` implements `Signable`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/mls_content.rs#L154) with label `"FramedContentTBS"`.

For application messages specifically, signing goes through:
1. [`MlsGroup::create_message()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/mls_group/application.rs#L16) calls [`AuthenticatedContent::new_application()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/mls_auth_content.rs#L122)
2. `new_application()` calls the internal [`new_and_sign()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/mls_auth_content.rs#L92) helper
3. `new_and_sign()` builds a `FramedContentTbs`, attaches the serialized group context (for member senders), and calls `content_tbs.sign(signer)` passing the client's signing key.
4. The result is an `AuthenticatedContent` containing the `FramedContent` body + `FramedContentAuthData` (signature and, for commits, a confirmation tag). `create_message` then calls [`MlsGroup::encrypt()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/mls_group/mod.rs#L646) which calls [`PrivateMessage::try_from_authenticated_content()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/private_message.rs#L71), which:
   - [Serializes the content body + auth data (signature) + padding](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/private_message.rs#L170) into a plaintext
   - Derives a ratchet key/nonce from the [sender's secret tree](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/private_message.rs#L178), XORed with a random reuse guard
   - [AEAD-encrypts](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/private_message.rs#L191) the plaintext with a [`PrivateContentAad`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/private_message.rs#L315) (group ID, epoch, content type, authenticated data) as the AAD
   - [Encrypts the sender data](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/private_message.rs#L238) (leaf index, generation, reuse guard) separately, keyed from the sender data secret and the content ciphertext

The same flow is used for proposals and commits (via `member_proposal()`, `commit()`, etc.), with different wire formats and content types.

### Verification

[`VerifiableAuthenticatedContentIn` implements `Verifiable`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/mls_auth_content_in.rs#L202) with the same label `"FramedContentTBS"`.

Verification occurs during message processing in [`MlsGroup::process_message()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/mls_group/processing.rs#L119), which calls [`unprotect_message()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/mls_group/processing.rs#L149) → [`decrypt_message()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/mls_group/processing.rs#L656). For `PrivateMessage` inputs, `decrypt_message()` delegates to [`DecryptedMessage::from_inbound_ciphertext()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/validation.rs#L96), which performs the following steps:

1. **Sender data decryption** — [`PrivateMessageIn::sender_data()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/private_message_in.rs#L57) derives a sender data key and nonce from the `sender_data_secret` and the content ciphertext bytes, then AEAD-opens the `encrypted_sender_data` with an `MlsSenderDataAad` (group ID, epoch, content type) as AAD. This yields the sender's leaf index, generation number, and reuse guard.

2. **Content decryption** — [`PrivateMessageIn::to_verifiable_content()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/private_message_in.rs#L153) retrieves the ratchet key/nonce from the [secret tree](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/private_message_in.rs#L166) for the sender's leaf index and generation, then XORs the nonce with the reuse guard. It then calls [`decrypt()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/private_message_in.rs#L105), which constructs a [`PrivateContentAad`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/private_message.rs#L315) (group ID, epoch, content type, authenticated data) as AAD and [AEAD-opens the ciphertext](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/private_message_in.rs#L128). The plaintext is deserialized into the content body + auth data (signature, and for commits, a confirmation tag). Finally, a [`VerifiableAuthenticatedContentIn` is reconstructed](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/private_message_in.rs#L193) with the wire format, `FramedContentIn`, serialized group context, and auth data.

3. **Validation** — The `DecryptedMessage` is checked for structural correctness (e.g. application messages must be encrypted, commits must have confirmation tags) in [`DecryptedMessage::from_verifiable_content()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/validation.rs#L132).

**Credential lookup** `unprotect_message` then [calls](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/mls_group/processing.rs#L199) [`PublicGroup::parse_message()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/public_group/process.rs#L44) which looks up the sender's credential and signature key from the ratchet tree (or past trees for old-epoch messages).

**Signature verification** `process_message` [calls](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/mls_group/processing.rs#L138) [`MlsGroup::process_unverified_message`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/group/mls_group/processing.rs#L408) which calls [`UnverifiedMessage::verify()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/validation.rs#L250) which calls [`self.verifiable_content.verify(crypto, &self.sender_pk)`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/validation.rs#L258). [`verify()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/framing/mls_auth_content_in.rs#L217) calls [`verify_no_out()`](https://github.com/openmls/openmls/blob/195f841c378807254efe6c93d0b82e9ad13f8577/openmls/src/ciphersuite/signable.rs#L144) which:
1. TLS-serializes the `FramedContentTbsIn` (including the group context if present)
2. Wraps it in `SignContent` with label `"FramedContentTBS"`
3. Calls `crypto.verify_signature()` against the sender's signature key

---

## Summary

All three message types rely on the same `SignWithLabel` / `VerifyWithLabel` pattern (via the `Signable` and `Verifiable` traits), but differ in their TBS struct, label, key source, and the additional encryption layers that protect the signed payload in transit.

| Type | TBS Struct | Label | Signing Key | Verification Key Source |
|------|-----------|-------|-------------|------------------------|
| Key Package | `KeyPackageTbs` | `"KeyPackageTBS"` | Client's signing key | Leaf node inside the key package |
| Welcome — GroupInfo | `GroupInfoTBS` | `"GroupInfoTBS"` | Committer's signing key | Signer's leaf node in the ratchet tree |
| Welcome — GroupSecrets | n/a (HPKE) | `"Welcome"` | n/a | Member's HPKE init key (from key package) |
| Application Message | `FramedContentTbs` | `"FramedContentTBS"` | Sender's signing key | Sender's leaf node in the ratchet tree |

**Key Packages** are the simplest case: the signature is verified directly against the leaf node's public key embedded in the key package itself.

**Welcome Messages** layer two mechanisms. The `GroupInfo` is signed with `SignWithLabel` and then symmetrically encrypted (AEAD) with a key derived from the welcome secret. The `EncryptedGroupSecrets` are not signed but are cryptographically bound to the `encrypted_group_info` ciphertext via HPKE's `EncryptWithLabel` — the ciphertext is mixed into the HPKE `info` parameter so that tampering with either component causes decryption to fail.

**Application Messages** have the most complex protection. The `FramedContent` is signed with `SignWithLabel`, then the signed content (body + signature + padding) is AEAD-encrypted with a ratchet key from the sender's secret tree to produce a `PrivateMessage`. The sender metadata is encrypted separately using the sender data secret. On the receiving side, the process reverses: sender data is decrypted first to identify the sender and retrieve the correct ratchet key, the content is AEAD-decrypted and deserialized, and finally the signature is verified against the sender's public key from the ratchet tree.
