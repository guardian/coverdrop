import Foundation

enum MessageHelperError: Error {
    case unableToCreateMessage
}

/// This helper is used to generate a mock user message inbox for the purpose of previewing the UI in xcode
/// It is located here because our tests are defined across multiple packages, and CoverDropCore is a common dependency
/// of them all
@MainActor public enum MessageHelper {
    public static func addMessagesToInbox(publicDataRepository: any PublicDataRepositoryProtocol) async throws
        -> SecretData {
        let twoDaysAgo = -TimeInterval(60 * 60 * 24 * 2)
        let twelveDaysAgo = -TimeInterval(60 * 60 * 24 * 12)
        let thirteenDaysAgo = -TimeInterval(60 * 60 * 24 * 13)

        let recipient = PublicKeysHelper.shared.testDefaultJournalist

        let otherRecipient = PublicKeysHelper.shared.getTestDesk
        // setup the message mailbox to be empty
        var messages: Set<Message> = []

        let userKeyPair: EncryptionKeypair<User> = try EncryptionKeypair<User>.generateEncryptionKeypair()
        let privateSendingQueueSecret = try PrivateSendingQueueSecret.fromSecureRandom()

        guard let recipientUnwrapped = recipient,
              let otherRecipientUnwrapped = otherRecipient else { throw MessageHelperError.unableToCreateMessage }

        let encryptedMessage = try await UserToCoverNodeMessageData.createMessage(
            message: "hey \(recipientUnwrapped.displayName)",
            messageRecipient: recipientUnwrapped,
            verifiedPublicKeys: publicDataRepository.getVerifiedKeys(),
            userPublicKey: userKeyPair.publicKey
        )

        let hint = HintHmac(hint: PrivateSendingQueueHmac.hmac(
            secretKey: privateSendingQueueSecret.bytes,
            message: encryptedMessage.asBytes()
        ))
        let outboundMessage = OutboundMessageData(
            recipient: recipientUnwrapped,
            messageText: "hey \(recipientUnwrapped.displayName)",
            dateQueued: Date(timeIntervalSinceNow: twoDaysAgo),
            hint: hint
        )

        let nonExpiredMessage = Message.outboundMessage(message: outboundMessage)

        let realOutboundMessage = OutboundMessageData(
            recipient: recipientUnwrapped,
            messageText: "hey outbound \(recipientUnwrapped.displayName)",
            dateQueued: Date(timeIntervalSinceNow: twelveDaysAgo),
            hint: hint
        )

        let realMessage = Message.outboundMessage(message: realOutboundMessage)

        let realReplyMessage = Message.incomingMessage(message: .textMessage(message: IncomingMessageData(
            sender: recipientUnwrapped,
            messageText: "hey user, from: \(recipientUnwrapped.displayName)",
            dateReceived: DateFunction.currentTime()
        )))

        let encryptedMessage2 = try await UserToCoverNodeMessageData.createMessage(
            message: "hey \(recipientUnwrapped.displayName)",
            messageRecipient: recipientUnwrapped,
            verifiedPublicKeys: publicDataRepository.getVerifiedKeys(),
            userPublicKey: userKeyPair.publicKey
        )

        let hint2 = HintHmac(hint: PrivateSendingQueueHmac.hmac(
            secretKey: privateSendingQueueSecret.bytes,
            message: encryptedMessage2.asBytes()
        ))

        let inactiveMessageInner = OutboundMessageData(
            recipient: otherRecipientUnwrapped,
            messageText: "hey \(otherRecipientUnwrapped.displayName)",
            dateQueued: Date(timeIntervalSinceNow: thirteenDaysAgo),
            hint: hint2
        )

        let inactiveMessage1 = Message.outboundMessage(message: inactiveMessageInner)

        let inactiveMessage2 = Message.incomingMessage(
            message: .textMessage(
                message: IncomingMessageData(
                    sender: otherRecipientUnwrapped,
                    messageText: "hey user from \(otherRecipientUnwrapped.displayName)",
                    dateReceived: Date(timeIntervalSinceNow: thirteenDaysAgo)
                )
            )
        )
        // add a message to the inbox
        messages.insert(realMessage)
        messages.insert(nonExpiredMessage)
        messages.insert(realReplyMessage)
        messages.insert(inactiveMessage1)
        messages.insert(inactiveMessage2)

        return .unlockedSecretData(unlockedData: UnlockedSecretData(
            messageMailbox: messages,
            userKey: userKeyPair,
            privateSendingQueueSecret: privateSendingQueueSecret
        ))
    }

    public static func loadMessagesFromDeadDrop() async throws -> SecretData {
        let maybeJournalistData = PublicKeysHelper.shared.testDefaultJournalist
        let verifiedPublicKeys = PublicKeysHelper.shared.testKeys

        let userMessageSecretKey = try PublicKeysHelper.shared.getTestUserMessageSecretKey()
        let userMessagePublicKey = try PublicKeysHelper.shared.getTestUserMessagePublicKey()
        let userKeyPair: EncryptionKeypair<User> = EncryptionKeypair(
            publicKey: userMessagePublicKey,
            secretKey: userMessageSecretKey
        )
        let privateSendingQueueSecret = try PrivateSendingQueueSecret.fromSecureRandom()

        let deadDropData = try DeadDropDataHelper.shared.readLocalDataFile()
        let verifiedDeadDrops = VerifiedDeadDrops.fromAllDeadDropData(
            deadDrops: deadDropData,
            verifiedKeys: verifiedPublicKeys
        )

        var userMessages: Set<Message> = []

        if let journalistData = maybeJournalistData {
            await userMessages.formUnion(
                DecryptedDeadDrops.decryptWithUserKey(
                    userSecretKey: userMessageSecretKey,
                    journalistData: journalistData,
                    verifiedDeadDropData: verifiedDeadDrops,
                    verifiedPublicKeys: verifiedPublicKeys
                )
            )
        }

        return .unlockedSecretData(unlockedData: UnlockedSecretData(
            messageMailbox: userMessages,
            userKey: userKeyPair,
            privateSendingQueueSecret: privateSendingQueueSecret
        ))
    }
}
