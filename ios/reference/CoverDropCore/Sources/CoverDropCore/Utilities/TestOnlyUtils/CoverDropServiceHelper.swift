import Foundation

enum CoverDropServiceHelperError: Error {
    case cannotGetTestJournalist
    case bothEmptyAndNonEmptyTestStorageRequested
    case unableToSaveCacheOnStartup
}

public enum CoverDropServiceHelper {
    public static func awaitCoverDropService() async throws -> CoverDropLibrary {
        while true {
            if case let .initialized(lib: lib) = CoverDropService.shared.state {
                return lib
            }
        }
    }

    public static func handleTestingFlags(
        config: CoverDropConfig,
        publicDataRepository: PublicDataRepository
    ) async throws {
        // By default we want to set the current time to the current keys published time
        TestingBridge
            .setCurrentTimeOverride(
                override: currentTimeForKeyVerification()
            )

        if TestingBridge.isEnabled(.removeBackgroundSendStateOnStart) {
            BackgroundMessageSendState.clearAllState()
        }

        if TestingBridge.isEnabled(.mockedDataExpiredMessagesScenario) {
            let keysDate = currentTimeForKeyVerification()
            let futureDateToMakeKeysExpired = Date(timeInterval: TimeInterval(60 * 60 * 24 * 13), since: keysDate)
            TestingBridge
                .setCurrentTimeOverride(
                    override: futureDateToMakeKeysExpired
                )
        }

        // checking for accidentally conflicting testing flags
        if TestingBridge.isEnabled(.startWithEmptyStorage), TestingBridge.isEnabled(.startWithNonEmptyStorage) {
            throw CoverDropServiceHelperError.bothEmptyAndNonEmptyTestStorageRequested
        }

        if TestingBridge.isEnabled(.startWithEmptyStorage) {
            try await addTestStorage(
                includeMessages: false,
                config: config,
                verifiedKeys: PublicKeysHelper.shared.testKeys,
                publicDataRepository: publicDataRepository
            )
        } else if TestingBridge.isEnabled(.startWithNonEmptyStorage) {
            try await addTestStorage(
                includeMessages: true,
                config: config,
                verifiedKeys: PublicKeysHelper.shared.testKeys,
                publicDataRepository: publicDataRepository
            )
        }

        if TestingBridge.isEnabled(.startWithCachedPublicKeys) {
            let publicKeysData = try PublicKeysHelper.readLocalKeysFile()
            let localPublicKeysCacheFile = LocalCacheFileRepository<PublicKeysData>(file: .publicKeysCache)

            guard (try? await localPublicKeysCacheFile.save(
                data: publicKeysData
            )) != nil else {
                throw CoverDropServiceHelperError.unableToSaveCacheOnStartup
            }
        }
    }

    public static func addTestMessagesForPreview(
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
                dateReceived: DateFunction
                    .currentTime()
                    .addingTimeInterval(-TimeInterval(60 * 60 * 24 * 15))
            ))),
            .incomingMessage(message: .textMessage(message: IncomingMessageData(
                sender: testDefaultJournalist,
                messageText: "Hey this has expiry warning",
                dateReceived: DateFunction
                    .currentTime()
                    .addingTimeInterval(-TimeInterval(60 * 60 * 24 * 13))
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
        verifiedKeys: VerifiedPublicKeys,
        publicDataRepository: PublicDataRepository
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

        //  get the verified keys
        guard (try? await publicDataRepository.loadAndVerifyPublicKeys()) != nil else {
            throw CoverDropServicesError.verifiedPublicKeysNotAvailable
        }

        guard let coverMessageFactory = try? publicDataRepository.getCoverMessageFactory() else {
            throw CoverDropServicesError.failedToGenerateCoverMessage
        }
        // create the private sending queue on disk if it does not exist
        _ = try await PrivateSendingQueueRepository.shared.loadOrInitialiseQueue(coverMessageFactory)

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
                dateReceived: Date(timeInterval: -TimeInterval(60 * 60 * 24 * 15), since: DateFunction.currentTime())
            ))),
            .incomingMessage(message: .textMessage(message: IncomingMessageData(
                sender: testDefaultJournalist,
                messageText: "Hey this has expiry warning",
                dateReceived: Date(timeInterval: -TimeInterval(60 * 60 * 24 * 13), since: DateFunction.currentTime())
            )))
        ]

        if TestingBridge.isEnabled(.mockedDataMultipleJournalists) {
            if let additionalJournalist = PublicKeysHelper.shared.testAdditionalJournalist {
                messages.insert(.incomingMessage(message: .textMessage(message: IncomingMessageData(
                    sender: additionalJournalist,
                    messageText: "Hey this was sent today from additional journalist",
                    dateReceived: DateFunction.currentTime()
                ))))
            }
        }

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

    public static func currentTimeForKeyVerification() -> Date {
        var date = Date()
        #if DEBUG
            do {
                if let generatedAtDate = try PublicKeysHelper.readLocalGeneratedAtFile() {
                    date = generatedAtDate
                }
            } catch { Debug.println("Failed to get local keys generated file") }
        #endif
        return date
    }
}
