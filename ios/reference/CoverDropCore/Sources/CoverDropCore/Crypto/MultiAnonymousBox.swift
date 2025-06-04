import Foundation
import Sodium

let wrappedKeySize = Sodium().secretBox.KeyBytes + Sodium().box.SealBytes

enum MultiAnonymousBoxError: Error {
    case keyGenFailed, encryptWithSecretBoxFailed, badOutputLength, decryptWithSecretBoxFailed,
         missingRecipientPublicKeys
}

public struct MultiAnonymousBox<T>: Equatable, Hashable {
    var bytes: [UInt8]
}

public extension MultiAnonymousBox {
    static func fromVecUnchecked<U: Encryptable>(bytes: [UInt8]) -> MultiAnonymousBox<U> {
        return MultiAnonymousBox<U>(
            bytes: bytes
        )
    }

    func asBytes() -> [UInt8] {
        bytes
    }

    static func encrypt<U: Encryptable, R: Role>(
        recipientPks: [PublicEncryptionKey<R>],
        data: U
    ) throws -> MultiAnonymousBox<T> {
        let message = data.asUnencryptedBytes()

        let key = Sodium().secretBox.key()

        if recipientPks.isEmpty {
            throw MultiAnonymousBoxError.missingRecipientPublicKeys
        }

        guard let ciphertext = encryptWithSecretBox(
            key: key,
            // since we always use fresh keys for each message, we can choose a constant nonce
            nonce: Array(repeating: UInt8(0), count: Sodium().secretBox.NonceBytes),
            message: message
        ) else {
            throw MultiAnonymousBoxError.encryptWithSecretBoxFailed
        }

        let wrappedKey = try recipientPks.compactMap { recipientPk in
            try AnonymousBox<[UInt8]>.encrypt(
                recipientPk: recipientPk,
                data: key
            )
        }

        let outputCapacity = Int(wrappedKey.count * wrappedKeySize + ciphertext.count)
        var output = Data(capacity: outputCapacity)

        for key in wrappedKey {
            output.append(contentsOf: key.asBytes())
        }

        output.append(contentsOf: ciphertext)
        if output.count != outputCapacity {
            throw MultiAnonymousBoxError.badOutputLength
        }

        return MultiAnonymousBox<T>(bytes: Array(output))
    }

    static func decrypt<U: Encryptable, R: Role>(
        recipientPk: PublicEncryptionKey<R>,
        recipientSk: SecretEncryptionKey<R>,
        data: MultiAnonymousBox<U>,
        numRecipients: Int
    ) throws -> U {
        let bytes = data.bytes
        assert(bytes.count >= numRecipients * wrappedKeySize + Sodium().secretBox.MacBytes, "bad data.bytes length")

        let (wrappedKeys, ciphertext) = bytes.splitAt(offset: numRecipients * wrappedKeySize)

        let key: [UInt8]? = findKey(wrappedKeys: wrappedKeys, recipientPk: recipientPk, recipientSk: recipientSk)

        if key == nil {
            throw MultiAnonymousBoxError.decryptWithSecretBoxFailed
        }

        if let plaintextBytes = Sodium().secretBox.open(
            authenticatedCipherText: ciphertext,
            secretKey: key!.asUnencryptedBytes(),
            nonce: Array(repeating: UInt8(0), count: Sodium().secretBox.NonceBytes)
        ) {
            return try U.fromUnencryptedBytes(bytes: plaintextBytes) as! U
        } else {
            throw MultiAnonymousBoxError.decryptWithSecretBoxFailed
        }
    }

    static func encryptWithSecretBox(
        key: [UInt8],
        nonce: [UInt8],
        message: [UInt8]
    ) -> [UInt8]? {
        let ciphertext = Sodium().secretBox.seal(message: message, secretKey: key, nonce: nonce)
        return ciphertext
    }

    static func findKey<R: Role>(
        wrappedKeys: [UInt8],
        recipientPk: PublicEncryptionKey<R>,
        recipientSk: SecretEncryptionKey<R>
    ) -> [UInt8]? {
        var foundKey: [UInt8]?

        let keys = wrappedKeys
            .chunked(into: wrappedKeySize)

        for wrappedKey in keys {
            do {
                let plaintext: [UInt8] = try AnonymousBox<[UInt8]>.decrypt(
                    myPk: recipientPk,
                    mySk: recipientSk,
                    data: AnonymousBox<[UInt8]>(pkTagAndCiphertext: wrappedKey)
                )
                foundKey = plaintext
                break
            } catch {
                continue
            }
        }
        return foundKey
    }
}
