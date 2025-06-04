import Foundation

// swiftlint:disable identifier_name
enum UserToCoverNodeMessageError: Error {
    case doesNotMatchUserToCoverNodeEncryptedMessageLength
}

public struct UserToCoverNodeMessageData: Equatable, Encryptable {
    let userToJournalistMessage: AnonymousBox<UserToJournalistMessageData>
    let recipientTag: RecipientTag

    public func asUnencryptedBytes() -> [UInt8] {
        var buffer = recipientTag.tag
        buffer.append(contentsOf: userToJournalistMessage.asBytes())
        return buffer
    }

    public static func fromUnencryptedBytes(bytes: [UInt8]) throws -> UserToCoverNodeMessageData {
        let recipientTag = RecipientTag(tag: Array(bytes.prefix(Constants.recipientTagLen)))
        let userToJournalistMessage = AnonymousBox<UserToJournalistMessageData>
            .fromVecUnchecked(bytes: Array(bytes.suffix(Constants.userToJournalistEncryptedMessageLen)))
        return UserToCoverNodeMessageData(userToJournalistMessage: userToJournalistMessage, recipientTag: recipientTag)
    }

    public static func == (lhs: UserToCoverNodeMessageData, rhs: UserToCoverNodeMessageData) -> Bool {
        return lhs.recipientTag == rhs.recipientTag &&
            lhs.userToJournalistMessage.asBytes() == rhs.userToJournalistMessage.asBytes()
    }

    public static func createMessage(
        message: String,
        messageRecipient: JournalistData,
        verifiedPublicKeys: VerifiedPublicKeys,
        userPublicKey: UserPublicKey
    ) async throws -> MultiAnonymousBox<UserToCoverNodeMessageData> {
        if let messageKey = verifiedPublicKeys.getLatestMessagingKey(journalistId: messageRecipient.recipientId) {
            return try UserToCoverNodeMessage.createMessage(
                message: message,
                recipientPublicKey: messageKey,
                verifiedPublicKeys: verifiedPublicKeys,
                userPublicKey: userPublicKey,
                tag: messageRecipient.tag
            )
        } else {
            throw KeysError.cannotFindFileError
        }
    }
}

public enum UserToCoverNodeMessage {
    ///  This function creates the whole user to journalist message.
    ///  We first encrypt the inner message, then pass that payload into the encryption for the outer message
    /// See the [message protocol documentation for more
    /// details](https://github.com/guardian/coverdrop/blob/main/docs/protocol_messages.md#user-to-journalist-message)
    /// - Parameters:
    ///   - message: The plain text message composed by the user
    ///   - recipientPublicKey: The recipient journalist or desks Public Key
    ///   - covernodeMessagePublicKey: The public key for covernode
    ///   - userPublicKey: The current users public key, this is generated and stored in the Apps secret data storage
    /// - Returns: an AnonymousBox of byte array which is the ciphertext for the outer covernode message. This can be
    /// sent as is to the covernode service.
    public static func createMessage(
        message: String,
        recipientPublicKey: JournalistMessagingPublicKey,
        verifiedPublicKeys: VerifiedPublicKeys,
        userPublicKey: PublicEncryptionKey<User>,
        tag: RecipientTag
    ) throws -> MultiAnonymousBox<UserToCoverNodeMessageData> {
        let innerMessage: AnonymousBox<UserToJournalistMessageData> = try UserToJournalistMessage
            .encryptRealMessageFromUserToJournalist(
                recipentPublicKeyData: recipientPublicKey,
                userPublicKey: userPublicKey,
                message: message
            )
        let keys = verifiedPublicKeys.mostRecentCoverNodeMessagingKeysFromAllHierarchies()
        let message = try UserToCoverNodeMessage.encryptRealMessageFromUserToCovernode(
            verifiedPublicKeys: keys,
            userToJournalistMessage: innerMessage,
            recipientIdenitfierTag: tag
        )
        return message
    }

    /// The outer covernode message, See the [user to covernode message documentation for more
    /// details](https://github.com/guardian/coverdrop/blob/main/docs/protocol_messages.md#user-to-covernode-message)
    /// - Parameters:
    ///   - covernodeMessagePublicKey: The covernode message public key
    ///   - userToJournalistMessageWithTag: The user to journalist message encrypted as an `AnonymousBox`
    ///   - recipientIdenitfierTag:The first 4 bytes of a  SHA256 hash of the journalist Identifier (generally their
    /// name)
    /// - Returns: an AnonymousBox of byte array which is the ciphertext for the outer  message.
    static func encryptRealMessageFromUserToCovernode(
        verifiedPublicKeys: [CoverNodeIdentity: CoverNodeMessagingPublicKey],
        userToJournalistMessage: AnonymousBox<UserToJournalistMessageData>,
        recipientIdenitfierTag: RecipientTag
    ) throws -> MultiAnonymousBox<UserToCoverNodeMessageData> {
        // append the covernode user to journalist message to the recipient Identifier Tag
        let serializedInnerCovernodeMessage = UserToCoverNodeMessageData(
            userToJournalistMessage: userToJournalistMessage,
            recipientTag: recipientIdenitfierTag
        )

        if serializedInnerCovernodeMessage.asUnencryptedBytes().count != (Constants.userToCovernodeMessageLen) {
            throw EncryptionError.failedToEncrypt
        }

        let coverNodeKeys = selectCoverNodeKeys(coverNodeKeys: verifiedPublicKeys)

        // encrypt the serializedInnerCovernodeMessage with the covernodeMessagePublicKey
        let encrypted: MultiAnonymousBox<UserToCoverNodeMessageData> = try MultiAnonymousBox<UserToCoverNodeMessageData>
            .encrypt(
                recipientPks: coverNodeKeys,
                data: serializedInnerCovernodeMessage
            )
        if encrypted.asBytes().count != (Constants.userToCovernodeEncryptedMessageLen) {
            throw UserToCoverNodeMessageError.doesNotMatchUserToCoverNodeEncryptedMessageLength
        }

        return encrypted
    }

    /**
     * Selects the covernode encryption keys (exactly [COVERNODE_WRAPPING_KEY_COUNT] many) to
     * for encrypting the outer message. If the given keys are more than the output list size,
     * then the first ones are chosen. Otherwise, the first ones are repeated.
     */
    public static func selectCoverNodeKeys(coverNodeKeys: [CoverNodeIdentity: CoverNodeMessagingPublicKey])
        -> [PublicEncryptionKey<CoverNodeMessaging>] {
        if coverNodeKeys.isEmpty {
            return []
        }
        let coverNodeKeysData = coverNodeKeys.values.map { $0.key }
        return Array(1 ... Constants.covernodeWrappingKeyCount)
            .compactMap { coverNodeKeysData[$0 % coverNodeKeysData.count] }
    }
}

// swiftlint:enable identifier_name
