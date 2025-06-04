import Foundation

enum CoverDropServiceHelperError: Error {
    case cannotGetTestJournalist
    case bothEmptyAndNonEmptyTestStorageRequested
}

public enum CoverDropServiceHelper {
    public static func awaitCoverDropService() async throws -> CoverDropLibrary {
        while true {
            if case let .initialized(lib: lib) = CoverDropService.shared.state {
                return lib
            }
        }
    }

    public static func handleTestingFlags(config: CoverDropConfig, verifiedKeys: VerifiedPublicKeys) async throws {
        if TestingBridge.isEnabled(.removeBackgroundSendStateOnStart) {
            BackgroundMessageSendState.clearAllState()
        }

        // checking for accidentally conflicting testing flags
        if TestingBridge.isEnabled(.startWithEmptyStorage), TestingBridge.isEnabled(.startWithNonEmptyStorage) {
            throw CoverDropServiceHelperError.bothEmptyAndNonEmptyTestStorageRequested
        }

        if TestingBridge.isEnabled(.startWithEmptyStorage) {
            try await addTestStorage(includeMessages: false, config: config, verifiedKeys: verifiedKeys)
        } else if TestingBridge.isEnabled(.startWithNonEmptyStorage) {
            try await addTestStorage(includeMessages: true, config: config, verifiedKeys: verifiedKeys)
        }
    }

    public static func addTestMessagesToLib(
        lib: CoverDropLibrary
    ) async throws -> UnlockedSecretData {
        guard let testDefaultJournalist = PublicKeysHelper.shared.testDefaultJournalist else {
            throw CoverDropServiceHelperError.cannotGetTestJournalist
        }

        // Set our test user keys
        let userSecretMessageKey = try PublicKeysHelper.shared.getTestUserMessageSecretKey()
        let userPublicMessageKey = try PublicKeysHelper.shared.getTestUserMessagePublicKey()
        let userKeyPair = EncryptionKeypair(publicKey: userPublicMessageKey, secretKey: userSecretMessageKey)
        let privateSendingQueueSecret = try PrivateSendingQueueSecret.fromSecureRandom()
        let encryptedMessage = try await UserToCoverNodeMessageData.createMessage(
            message: "Hey this is pending",
            messageRecipient: testDefaultJournalist,
            verifiedPublicKeys: lib.publicDataRepository.getVerifiedKeys(),
            userPublicKey: userKeyPair.publicKey
        )

        let hint = HintHmac(hint: PrivateSendingQueueHmac.hmac(
            secretKey: privateSendingQueueSecret.bytes,
            message: encryptedMessage.asBytes()
        ))

        var messages: Set<Message> = []

        let outboundMessage = OutboundMessageData(
            recipient: testDefaultJournalist,
            messageText: "Hey this is pending",
            dateQueued: DateFunction.currentTime(),
            hint: hint
        )

        messages = [
            .outboundMessage(message: outboundMessage),
            .incomingMessage(message: .textMessage(message: IncomingMessageData(
                sender: testDefaultJournalist,
                messageText: "Hey this has expired",
                dateReceived: Date(timeIntervalSinceNow: -TimeInterval(60 * 60 * 24 * 15))
            ))),
            .incomingMessage(message: .textMessage(message: IncomingMessageData(
                sender: testDefaultJournalist,
                messageText: "Hey this has expiry warning",
                dateReceived: Date(timeIntervalSinceNow: -TimeInterval(60 * 60 * 24 * 13))
            ))),
            .incomingMessage(message: .textMessage(message: IncomingMessageData(
                sender: testDefaultJournalist,
                messageText: "Hey this was sent today",
                dateReceived: DateFunction.currentTime()
            )))
        ]

        let data = UnlockedSecretData(
            messageMailbox: messages,
            userKey: userKeyPair,
            privateSendingQueueSecret: privateSendingQueueSecret
        )
        return data
    }

    private static func addTestStorage(
        includeMessages _: Bool,
        config _: CoverDropConfig,
        verifiedKeys: VerifiedPublicKeys
    ) async throws {
        guard let testDefaultJournalist = PublicKeysHelper.shared.testDefaultJournalist else {
            throw CoverDropServiceHelperError.cannotGetTestJournalist
        }

        let passphrase = ValidPassword(password: "external jersey squeeze")

        let encryptedStorage = EncryptedStorage.createForTesting()
        let session = try await encryptedStorage.createOrResetStorageWithPassphrase(passphrase: passphrase)

        // Set our test user keys
        let userSecretMessageKey = try PublicKeysHelper.shared.getTestUserMessageSecretKey()
        let userPublicMessageKey = try PublicKeysHelper.shared.getTestUserMessagePublicKey()
        let userKeyPair = EncryptionKeypair(publicKey: userPublicMessageKey, secretKey: userSecretMessageKey)
        let privateSendingQueueSecret = try PrivateSendingQueueSecret.fromSecureRandom()

        let encryptedMessage = try await UserToCoverNodeMessageData.createMessage(
            message: "Hey this is pending",
            messageRecipient: testDefaultJournalist,
            verifiedPublicKeys: verifiedKeys,
            userPublicKey: userKeyPair.publicKey
        )

        let hint = try await PrivateSendingQueueRepository.shared.enqueue(
            secret: privateSendingQueueSecret,
            message: MultiAnonymousBox(bytes: encryptedMessage.bytes)
        )

        var messages: Set<Message> = []

        let outboundMessage = OutboundMessageData(
            recipient: testDefaultJournalist,
            messageText: "Hey this is pending",
            dateQueued: DateFunction.currentTime(),
            hint: hint
        )

        let outboundMessage2 = OutboundMessageData(
            recipient: testDefaultJournalist,
            messageText: "Hey this is sent",
            dateQueued: DateFunction.currentTime(),
            hint: HintHmac(hint: [0, 0, 0, 0])
        )

        messages = [
            .outboundMessage(message: outboundMessage),
            .outboundMessage(message: outboundMessage2),
            .incomingMessage(message: .textMessage(message: IncomingMessageData(
                sender: testDefaultJournalist,
                messageText: "Hey this has expired",
                dateReceived: Date(timeIntervalSinceNow: -TimeInterval(60 * 60 * 24 * 15))
            ))),
            .incomingMessage(message: .textMessage(message: IncomingMessageData(
                sender: testDefaultJournalist,
                messageText: "Hey this has expiry warning",
                dateReceived: Date(timeIntervalSinceNow: -TimeInterval(60 * 60 * 24 * 13))
            )))
        ]

        let data = UnlockedSecretData(
            messageMailbox: messages,
            userKey: userKeyPair,
            privateSendingQueueSecret: privateSendingQueueSecret
        )
        try await encryptedStorage.updateStorageOnDisk(
            session: session,
            state: data
        )
    }
}
