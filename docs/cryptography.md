# Cryptography

The CoverDrop internally uses the following cryptographic primitives for signing and encrypting data.
Most of them are based on [libsodium](https://libsodium.gitbook.io/doc/) which is available on many platforms.

## Keys

We use keys based on Curve25519 to perform Diffie-Hellman key agreement (called X25519).
The secret and public keys are 32 bytes in length each [[src]](https://cr.yp.to/ecdh/curve25519-20060209.pdf).

For Curve25519 (and others) there exist multiple public keys for each secret key that lead to the same derived
secret [[src]](https://www.rfc-editor.org/rfc/rfc7748#section-7):

> Designers using these curves should be aware that for each public
> key, there are several publicly computable public keys that are
> equivalent to it, i.e., they produce the same shared secrets. Thus
> using a public key as an identifier and knowledge of a shared secret
> as proof of ownership (without including the public keys in the key
> derivation) might lead to subtle vulnerabilities.

For the signatures we use twisted Edwards curves which can be converted to and from Curve25519 and share the same
security properties [[src]](https://libsodium.gitbook.io/doc/advanced/ed25519-curve25519). The DH and signature
algorithms use the different representations for performance optimizations.

The (encoded) representation of elliptic curves are not guaranteed to be indistinguishable from random. For instance,
Curve25519 uses a "fixed position for the leading 1 in the secret
key" [[src]](http://cr.yp.to/ecdh/curve25519-20060209.pdf).

### Implementations

- [x] [Android](../android/core/src/main/java/com/theguardian/coverdrop/core/crypto/EncryptionKeys.kt)
- [x] [iOS](../ios/reference/CoverDropCore/Sources/CoverDropCore/Crypto/Keys/encryption.swift)
- [x] [Rust](../common/src/crypto/keys)

### Test vectors

There are currently no test vectors for keys.

## Signatures

We use the Ed25519 signatures scheme which uses SHA-512 and Curve25519 internally.
It provides a security level of 128 bit [[src]](https://ed25519.cr.yp.to/).
Encoded Ed25519 signatures are 64 bytes long [[src]](https://ed25519.cr.yp.to/).

The (encoded) representation of Ed25519 signatures are not guaranteed to be indistinguishable from random.
For instance, some specific bits of valid signatures are always `0`
[[src]](https://docs.rs/ed25519/latest/src/ed25519/lib.rs.html#294-304).

### Implementations

- [x] [Android](../android/core/src/main/java/com/theguardian/coverdrop/core/crypto/Signature.kt)
- [x] [iOS](../ios/reference/CoverDropCore/Sources/CoverDropCore/Crypto/Keys/SignedKeys.swift)
- [x] [Rust](../common/src/crypto/signature.rs)

### Test vectors

There are currently no test vectors for signatures.

## Secret Box

Secret Box is an abstraction for symmetric authenticated encryption.
It internally uses [XChaCha20Poly1305](https://datatracker.ietf.org/doc/html/draft-irtf-cfrg-xchacha).
The `X` indicates an extended nonce of 192-bit compared to the 96-bit one of just ChaCha20.
This makes XChaCha20 safe to use with using randomly generated "
nonces" [[src]](https://libsodium.gitbook.io/doc/secret-key_cryptography/aead/chacha20-poly1305/xchacha20-poly1305_construction)
.
In our construction we append the nonce to the result.

The encoded representation looks like the following and has an overhead of 36 Bytes:

```
+-----+------------+-------+
| tag | ciphertext | nonce |
+-----+------------+-------+
   ^         ^         ^
   |         |         |
  16B    variable     24B
```

The tag comes before the ciphertext [[src]](https://datatracker.ietf.org/doc/html/rfc8439#section-2.8) and is
indistinguishable from random bytes.

### Implementations

- [ ] Android
- [ ] iOS
- [x] [Rust](../common/src/crypto/secret_box.rs)

### Test vectors

There are currently no test vectors for Secret Box.

## Anonymous Box

Anonymous Box is an abstraction for unauthenticated asymmetric encryption.
It internally uses [libsodium's Sealed Box](https://libsodium.gitbook.io/doc/public-key_cryptography/sealed_boxes)
primitive which is based on X25519 DH key agreement and XSalsa20-Poly1305.
The sender stays anonymous and their public key is not included.
Therefore, an ephemeral secret key is created and its public key `epk` is prepended to the result.

The encoded representation looks like the following and has an overhead of 48 Bytes:

```
+-----+-----+------------+
| epk | tag | ciphertext |
+-----+-----+------------+
   ^     ^        ^
   |     |        |
  32B   16B    variable
```

The key for the ciphertext is derived as `X25519(recipient public key, ephemeral secret key)`.
The nonce for the ciphertext is derived
as `Blake2(ephemeral public key || recipient public key)` [[src]](https://libsodium.gitbook.io/doc/public-key_cryptography/sealed_boxes#algorithm-details)
.

### Implementations

- [x] [Android](../android/core/src/main/java/com/theguardian/coverdrop/core/crypto/AnonymousBox.kt)
- [x] [iOS](../ios/reference/CoverDropCore/Sources/CoverDropCore/Crypto/anonymousBox.swift)
- [x] [Rust](../common/src/crypto/anonymous_box.rs)

### Test vectors

Test vectors for the Anonymous Box are available at [`common/tests/vectors/anonymous_box`](../common/tests/vectors/anonymous_box).
They are covered by the [`common/tests/verify_crypto_test_vectors.rs`](../common/tests/verify_crypto_test_vectors.rs) test file.

## Two Party Box

Two Party Box is an abstraction for authenticated asymmetric encryption.
It internally uses
[libsodium's Crypto Box](https://libsodium.gitbook.io/doc/public-key_cryptography/authenticated_encryption)
primitive which is based on X25519 DH key agreement and XSalsa20-Poly1305.
As the sender is known, their public key is known to the recipient and no ephemeral key needs to be included.
In our construction we append the nonce to the result.

The encrypted message is authenticated using public-key authenticators.
No additional signature is needed, unless public verifiability is required [[src]](https://nacl.cr.yp.to/box.html).

The encoded representation looks like the following and has an overhead of 36 Bytes:

```
+-----+------------+-------+
| tag | ciphertext | nonce |
+-----+------------+-------+
   ^        ^          ^
   |        |          |
  16B    variable     24B
```

The key for the ciphertext is derived as `X25519(recipient public key, sender secret key)`.
The nonce for the ciphertext is randomly generated.

### Implementations

- [x] [Android](../android/core/src/main/java/com/theguardian/coverdrop/core/crypto/TwoPartyBox.kt)
- [x] [iOS](../ios/reference/CoverDropCore/Sources/CoverDropCore/Crypto/twoPartyBox.swift)
- [x] [Rust](../common/src/crypto/two_party_box.rs)

### Test vectors

Test vectors for the Two Party Box are available at [`common/tests/vectors/two_party_box`](../common/tests/vectors/two_party_box).
They are covered by the [`common/tests/verify_crypto_test_vectors.rs`](../common/tests/verify_crypto_test_vectors.rs) test file.

## Multi Anonymous Box

Multi Anonymous Box is an abstraction for unauthenticated asymmetric encryption for multiple recipients.
It internally uses generates a fresh secret key `s` and then encrypts the payload under XSalsa20-Poly1305.
The secret key `s` is then encrypted for each recipient `r` using an Anonymous Box `ab_r` which are placed at the
beginning of the final output.
The sender stays anonymous and their public key is not included.

The encoded representation looks like the following and has an overhead of 16 Bytes for the tag and 80 Bytes for each
recipient:

```
+------+---+------+------------+-----+
| ab_1 |...| ab_r | ciphertext | tag |
+------+---+------+------------+-----+
   ^          ^         ^         ^
   |          |         |         |
  80B        80B     variable    16B
```

### Implementations

- [x] [Android](../android/core/src/main/java/com/theguardian/coverdrop/core/crypto/MultiAnonymousBox.kt)
- [x] [iOS](../ios/reference/CoverDropCore/Sources/CoverDropCore/Crypto/MultiAnonymousBox.swift)
- [x] [Rust](../common/src/crypto/multi_anonymous_box.rs)

### Test vectors

Test vectors for the Anonymous Box are available at [`common/tests/vectors/multi_anonymous_box`](../common/tests/vectors/multi_anonymous_box).
They are covered by the [`common/tests/verify_crypto_test_vectors.rs`](../common/tests/verify_crypto_test_vectors.rs) test file.

## Padded Compressed String

A Padded Compressed String takes a byte array, compresses it using GZip, and then pads the result to a given length.
The length of the compressed payload is prepended to allow later reconstruction.
No guarantees are given for the values used for padding.

The encoded representation looks like the following and has an overhead of at least 2 Bytes:

```
+-----+------------------------+
| len | compressed ... padding |
+-----+------------------------+
   ^               ^
   |               |
  2B           $len Bytes
```

### Implementations

- [x] [Android](../android/core/src/main/java/com/theguardian/coverdrop/core/models/PaddedCompressedString.kt)
- [x] [iOS](../ios/reference/CoverDropCore/Sources/CoverDropCore/PaddedCompressedString.swift)
- [x] [Rust](../common/src/padded_compressed_string.rs)

### Test vectors

There are currently no test vectors for Padded Compressed String.

## Implementation specific notes

### Rust

In the Rust implementation we preserve type information when applying cryptographic primitives.

For the encryption operations all types that can be encrypted implement the `Encryptable` trait.
This allows them to be directly passed to, for instance, the Secret Box encryption method.
When using `SecretBox::encrypt` on a type `T`, the output type is `SecretBox<T>`.
This prevents accidentally confusing different types after encryption.
It also prevents mixing plaintext and ciphertext byte arrays.

### Android

Not yet implemented.

### iOS

Not yet implemented.
