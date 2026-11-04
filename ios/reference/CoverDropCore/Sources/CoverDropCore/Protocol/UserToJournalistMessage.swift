import Foundation

enum UserToJournalistMessageError: Error {
    case doesNotMatchU2JEncryptedMessageLength
    case doesNotMatchU2JMessageLength
    case doesNotMatchU2JMessagePayloadLength
}

public struct UserToJournalistMessageData: Equatable, Encryptable {
    let publicKey: PublicEncryptionKey<User>
    let payload: TwoPartyBox<PaddedCompressedString<U2JPaddedLength>>

    public func asUnencryptedBytes() -> [UInt8] {
        let publicKeyBytes = publicKey.toBytes()
        return publicKeyBytes + [UInt8(Constants.authenticatedMessageReservedByteValue)] + payload.asBytes()
    }

    public static func fromUnencryptedBytes(bytes: [UInt8]) throws -> UserToJournalistMessageData {
        if bytes.count != Constants.userToJournalistMessageLen {
            throw UserToJournalistMessageError.doesNotMatchU2JMessageLength
        }

        let publicKey = PublicEncryptionKey<User>(key: Array(bytes.prefix(Constants.x25519PublicKeyLen)))

        // Skip reserved byte as it's unused

        let payloadBytes = Array(bytes
            .suffix(Constants.userToJournalistPayloadLen))
        let payload: TwoPartyBox<PaddedCompressedString<U2JPaddedLength>> =
            TwoPartyBox<PaddedCompressedString<U2JPaddedLength>>
                .fromVecUnchecked(bytes: payloadBytes)

        return UserToJournalistMessageData(
            publicKey: publicKey,
            payload: payload
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
        userKeyPair: EncryptionKeypair<User>,
        message: String
    ) throws -> AnonymousBox<UserToJournalistMessageData> {
        // compress and pad the plaintext message
        let compressedMessage = try PaddedCompressedString<U2JPaddedLength>.fromString(text: message)
        if compressedMessage.totalLength() != Constants.authenticatedMessagePaddingLen {
            throw PaddedCompressedStringError.paddedCompressedStringTooLong
        }
        let twoPartyBox = try TwoPartyBox<UserToJournalistMessageData>.encrypt(
            recipientPk: recipentPublicKeyData.key,
            senderSk: userKeyPair.secretKey,
            data: compressedMessage
        )
        if twoPartyBox.asBytes().count != Constants.userToJournalistPayloadLen {
            throw UserToJournalistMessageError.doesNotMatchU2JMessagePayloadLength
        }
        // append the compressed the user public key
        let publicKeysBytes = PublicEncryptionKey<User>(key: userKeyPair.publicKey.toBytes())

        let userToJournalistData = UserToJournalistMessageData(
            publicKey: publicKeysBytes,
            payload: twoPartyBox
        )

        // create a PublicEncryptionKey instance from the recipient key bytes and
        // encrypt the padded compressed string and public key with Anonymous box
        let encrypted: AnonymousBox<UserToJournalistMessageData> = try AnonymousBox<UserToJournalistMessageData>
            .encrypt(
                recipientPk: recipentPublicKeyData.key,
                data: userToJournalistData
            )

        // prepend recipient_tag to make cover node message
        if encrypted.pkTagAndCiphertext.count != Constants.userToJournalistEncryptedMessageLen {
            throw UserToJournalistMessageError.doesNotMatchU2JEncryptedMessageLength
        }

        return encrypted
    }
}
