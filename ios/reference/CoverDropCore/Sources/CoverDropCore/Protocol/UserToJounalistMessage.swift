import Foundation

public struct UserToJournalistMessageData: Equatable, Encryptable {
    let publicKey: PublicEncryptionKey<User>
    let reservedByte: UInt8
    let paddedCompressedString: PaddedCompressedString

    public func asUnencryptedBytes() -> [UInt8] {
        let publicKeyBytes = publicKey.toBytes()
        let paddedCompressedStringBytes = paddedCompressedString.value
        return publicKeyBytes + [reservedByte] + paddedCompressedStringBytes
    }

    public static func fromUnencryptedBytes(bytes: [UInt8]) throws -> UserToJournalistMessageData {
        let publicKey = PublicEncryptionKey<User>(key: Array(bytes.prefix(Constants.x25519PublicKeyLen)))
        let reservedByte = bytes[Constants.x25519PublicKeyLen]
        let plainTextPaddedCompresssedStringBytes = Array(bytes.suffix(Constants.messagePaddingLen))

        let plainTextPaddedCompresssedString = try PaddedCompressedString
            .fromUncheckedBytes(bytes: plainTextPaddedCompresssedStringBytes)
        return UserToJournalistMessageData(
            publicKey: publicKey,
            reservedByte: reservedByte,
            paddedCompressedString: plainTextPaddedCompresssedString
        )
    }
}

public enum UserToJournalistMessage {
    /// creates the inner part of the covernode message See the [user to journalist message documentation for more
    /// details](https://github.com/guardian/coverdrop/blob/main/docs/protocol_messages.md#user-to-journalist-message)
    /// - Parameters:
    ///
    /// - message: The plain text message composed by the user
    ///   - recipientPublicKey: The recipient journalist or desks Public Key
    ///   - userPublicKey: The current users public key, this is generated and stored in the Apps secret data storage
    ///   - message: The plain text message composed by the user
    /// - Returns: an AnonymousBox of byte array which is the ciphertext for the inner  message.
    static func encryptRealMessageFromUserToJournalist(
        recipentPublicKeyData: JournalistMessagingPublicKey,
        userPublicKey: PublicEncryptionKey<User>,
        message: String
    ) throws -> AnonymousBox<UserToJournalistMessageData> {
        // compress and pad the plaintext message
        let compressedMessage = try PaddedCompressedString.fromString(text: message)
        if compressedMessage.totalLength() != Constants.messagePaddingLen {
            throw PaddedCompressedStringError.paddedCompressedStringTooLong
        }
        // append the compressed the user public key
        let publicKeysBytes = PublicEncryptionKey<User>(key: userPublicKey.toBytes())

        let userToJournalistPlaintext = UserToJournalistMessageData(
            publicKey: publicKeysBytes,
            reservedByte: 0x00,
            paddedCompressedString: compressedMessage
        )

        // create a PublicEncryptionKey instance from the recipient key bytes and
        // encrypt the padded compressed string and public key with Anonymous box
        let encrypted: AnonymousBox<UserToJournalistMessageData> = try AnonymousBox<UserToJournalistMessageData>
            .encrypt(
                recipientPk: recipentPublicKeyData.key,
                data: userToJournalistPlaintext
            )

        // prepend recipient_tag to make cover node message
        if encrypted.pkTagAndCiphertext.count != Constants.userToJournalistEncryptedMessageLen {
            throw EncryptionError.failedToEncrypt
        }

        return encrypted
    }
}
