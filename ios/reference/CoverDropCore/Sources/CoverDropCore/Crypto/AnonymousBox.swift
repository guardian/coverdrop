import Foundation
import Sodium

/// Used for public key cryptography using ephemeral X25519 key exchange followed by XSalsa20Poly1305.
/// Internally uses `libsodium`'s `sealed_box` primative, for interoperability with other platforms.
///
/// The byte array contains the ciphertext, AEAD tag and ephemeral public key.
/// Nonces are not stored since they are created by hashing the ephemeral and recipient public keys using BLAKE2.
///
/// Like [`SecretBox`], `AnonymousBox` works with types that implement [`Encryptable`].
///
/// [`Encryptable`]: super::Encryptable
/// [`SecretBox`]: super::SecretBox
/// [`sealed_box`]: https://libsodium.gitbook.io/doc/public-key_cryptography/sealed_boxes
///
/// Please read `https://github.com/guardian/coverdrop/blob/main/docs/cryptography.md` for details on `AnonymousBox`
/// functions

public struct AnonymousBox<T: Encryptable>: Equatable {
    let pkTagAndCiphertext: [UInt8]
}

public extension AnonymousBox {
    /// Create a new `AnonymousBox` from a byte array without checking if it's valid.
    static func fromVecUnchecked(bytes: [UInt8]) -> AnonymousBox {
        return AnonymousBox<T>(
            pkTagAndCiphertext: bytes
        )
    }

    /// Returns the `AnonymousBox` data as bytes, as this is already stored at `[UInt8]` no transformation is required
    func asBytes() -> [UInt8] {
        pkTagAndCiphertext
    }

    /// Encrypts data with supplied `PublicEncryptionKey`
    static func encrypt<S: Encryptable, R: EncryptionKey>(
        recipientPk: R,
        data: S
    ) throws -> AnonymousBox<S> {
        let bytes = data.asUnencryptedBytes()

        let pkTagAndCiphertext = Sodium().box.seal(message: bytes, recipientPublicKey: recipientPk.key)

        return AnonymousBox<S>(pkTagAndCiphertext: pkTagAndCiphertext!)
    }

    /// Decrypts data with supplied `PublicEncryptionKey` and `SecretEncryptionKey`
    /// The `PublicEncryptionKey` has to be derived from the `SecretEncryptionKey` ie they need to be a related Key Pair
    /// `throws` a `EncryptionError.failedToDecrypt` if `open`ing the `AnonymousBox` fails,
    /// or if converting the resulting data to `T`  `fromUnencryptedBytes` fails
    ///
    internal static func decrypt<S: Encryptable, R: Role>(
        myPk: PublicEncryptionKey<R>,
        mySk: SecretEncryptionKey<R>,
        data: AnonymousBox<S>
    ) throws -> S {
        let ciphertext = data.pkTagAndCiphertext

        if let plaintextBytes = Sodium().box.open(
            anonymousCipherText: ciphertext,
            recipientPublicKey: myPk.key,
            recipientSecretKey: mySk.key
        ) {
            return try S.fromUnencryptedBytes(bytes: plaintextBytes) as! S

        } else {
            throw EncryptionError.failedToDecrypt
        }
    }
}
