@testable import CoverDropCore
import Sodium
import XCTest

final class SecretDataRepositoryTests: XCTestCase {
    override func setUp() async throws {
        try StorageManager.shared.deleteFile(file: CoverDropFiles.encryptedStorage)
    }

    override func tearDown() {
        // Remove the time override we set in the expiration test
        TestingBridge.setCurrentTimeOverride(override: nil)
    }

    func testSecretDataRepositoryRoundTrip() async throws {
        let publicDataRepository = PublicDataRepository(StaticConfig.devConfig, urlSession: URLSession.shared)
        let secretDataRepository = SecretDataRepository(publicDataRepository: publicDataRepository)

        let verifiedPublicKeys = PublicKeysHelper.shared.testKeys
        publicDataRepository.injectVerifiedPublicKeysForTesting(verifiedPublicKeys: verifiedPublicKeys)

        guard (try? publicDataRepository.getCoverMessageFactory()) != nil else {
            XCTFail("Unable to make cover message")
            return
        }

        try await secretDataRepository.onAppStart()

        // setup secret data repository with new passphrase
        let passphrase = ValidPassword(password: "external jersey squeeze")
        try await secretDataRepository.createOrReset(passphrase: passphrase)

        let recipient = PublicKeysHelper.shared.testDefaultJournalist!
        let privateSendingQueueSecret = try PrivateSendingQueueSecret.fromSecureRandom()
        let userKeyPair: EncryptionKeypair<User> = try EncryptionKeypair<User>.generateEncryptionKeypair()

        let encryptedMessage = try await UserToCoverNodeMessageData.createMessage(
            message: "hey",
            messageRecipient: recipient,
            verifiedPublicKeys: verifiedPublicKeys,
            userPublicKey: userKeyPair.publicKey
        )

        let hint = HintHmac(hint: PrivateSendingQueueHmac.hmac(
            secretKey: privateSendingQueueSecret.bytes,
            message: encryptedMessage.asBytes()
        ))

        let realMessage = Message.outboundMessage(
            message: OutboundMessageData(
                recipient: recipient,
                messageText: "hey",
                dateQueued: DateFunction.currentTime(),
                hint: hint
            )
        )
        let messages: Set<Message> = [realMessage]

        let newData = UnlockedSecretData(
            messageMailbox: messages,
            userKey: userKeyPair,
            privateSendingQueueSecret: privateSendingQueueSecret
        )

        secretDataRepository.setUnlockedDataForTesting(unlockedData: newData)
        try await secretDataRepository.lock()
        if case .lockedSecretData = secretDataRepository.secretData {
            XCTAssertTrue(true)
        } else {
            XCTFail("secretDataRepository was not locked after locking")
        }

        try await secretDataRepository.unlock(passphrase: passphrase)
        if case let .unlockedSecretData(data) = secretDataRepository.secretData {
            Task {
                await MainActor.run {
                    XCTAssertTrue(data.messageMailbox == messages)
                }
            }
        } else {
            XCTFail("secretDataRepository was not unlocked after unlocking")
        }
    }

    func testAddingMessagesSavesToDisk() async throws {
        // Start CoverDrop service, private sending queue, and get keys
        let publicDataRepository = PublicDataRepository(StaticConfig.devConfig, urlSession: URLSession.shared)
        let secretDataRepository = SecretDataRepository(publicDataRepository: publicDataRepository)
        let verifiedPublicKeys = PublicKeysHelper.shared.testKeys
        publicDataRepository.injectVerifiedPublicKeysForTesting(verifiedPublicKeys: verifiedPublicKeys)

        guard (try? publicDataRepository.getCoverMessageFactory()) != nil else {
            XCTFail("Unable to make cover message")
            return
        }

        try await secretDataRepository.onAppStart()

        // setup secret data repository with new passphrase
        let passphrase = ValidPassword(password: "external jersey squeeze")
        try await secretDataRepository.createOrReset(passphrase: passphrase)

        // Create a recipient journalist and the user key pair
        let recipient = PublicKeysHelper.shared.testDefaultJournalist!
        let privateSendingQueueSecret = try PrivateSendingQueueSecret.fromSecureRandom()
        let userKeyPair: EncryptionKeypair<User> = try EncryptionKeypair<User>.generateEncryptionKeypair()

        // Create an encrypted message to the recipient journalist
        let encryptedMessage = try await UserToCoverNodeMessageData.createMessage(
            message: "hey",
            messageRecipient: recipient,
            verifiedPublicKeys: verifiedPublicKeys,
            userPublicKey: userKeyPair.publicKey
        )
        let hint = HintHmac(hint: PrivateSendingQueueHmac.hmac(
            secretKey: privateSendingQueueSecret.bytes,
            message: encryptedMessage.asBytes()
        ))
        let realMessage = Message.outboundMessage(
            message: OutboundMessageData(
                recipient: recipient,
                messageText: "hey",
                dateQueued: DateFunction.currentTime(),
                hint: hint
            )
        )

        // Add the new message to the current a new version of unlocked secret data
        let messages: Set<Message> = [realMessage]
        let newData = UnlockedSecretData(
            messageMailbox: messages,
            userKey: userKeyPair,
            privateSendingQueueSecret: privateSendingQueueSecret
        )

        // Lock the secretDataRespository and check the enum state is updated to locked
        secretDataRepository.setUnlockedDataForTesting(unlockedData: newData)
        try await secretDataRepository.lock()
        if case .lockedSecretData = secretDataRepository.secretData {
            XCTAssertTrue(true)
        } else {
            XCTFail("secretDataRepository was not locked after locking")
        }

        // unlock the secretDataRespository and check the enum state is updated to unlocked

        _ = try await secretDataRepository.unlock(passphrase: passphrase)
        guard case .unlockedSecretData = secretDataRepository.secretData else {
            XCTFail("secretDataRepository was not unlocked after unlocking")
            return
        }

        // create another message to the recipient journalist
        _ = try await UserToCoverNodeMessageData.createMessage(
            message: "another message",
            messageRecipient: recipient,
            verifiedPublicKeys: verifiedPublicKeys,
            userPublicKey: userKeyPair.publicKey
        )
        let hint2 = HintHmac(hint: PrivateSendingQueueHmac.hmac(
            secretKey: privateSendingQueueSecret.bytes,
            message: encryptedMessage.asBytes()
        ))
        let newMessage = Message.outboundMessage(
            message: OutboundMessageData(
                recipient: recipient,
                messageText: "another message",
                dateQueued: DateFunction.currentTime(),
                hint: hint2
            )
        )

        // Add the new message to the current mailbox (which we expect to save everything to disk)
        try await secretDataRepository.addMessage(message: newMessage)
        if case let .unlockedSecretData(data) = secretDataRepository.secretData {
            XCTAssertTrue(data.messageMailbox.contains(newMessage))
        } else {
            XCTFail("secretDataRepository was not unlocked after unlocking")
        }

        // Unlock the data from disk again, we expect to see the new message we just added in the contents of the
        // mailbox from disk
        _ = try await secretDataRepository.unlock(passphrase: passphrase)
        if case let .unlockedSecretData(data) = secretDataRepository.secretData {
            Task {
                await MainActor.run {
                    XCTAssertTrue(data.messageMailbox.contains(newMessage))
                }
            }
        } else {
            XCTFail("secretDataRepository was not unlocked after unlocking")
        }
    }

    func testMessagesAreBeingExpiredOnSave() async throws {
        // We do some manual time-travel, so let's lock us in with a stable base, doc!
        let baseTime = try PublicKeysHelper.readLocalGeneratedAtFile()!
        let expiryTime = try baseTime.plusSeconds(Constants.messageValidForDurationInSeconds)
        TestingBridge.setCurrentTimeOverride(override: baseTime)

        // Start CoverDrop service, private sending queue, and get keys
        let publicDataRepository = PublicDataRepository(StaticConfig.devConfig, urlSession: URLSession.shared)
        let secretDataRepository = SecretDataRepository(publicDataRepository: publicDataRepository)
        let verifiedPublicKeys = PublicKeysHelper.shared.testKeys
        publicDataRepository.injectVerifiedPublicKeysForTesting(verifiedPublicKeys: verifiedPublicKeys)

        guard (try? publicDataRepository.getCoverMessageFactory()) != nil else {
            XCTFail("Unable to make cover message")
            return
        }

        try await secretDataRepository.onAppStart()

        // setup secret data repository with new passphrase
        let passphrase = ValidPassword(password: "external jersey squeeze")
        try await secretDataRepository.createOrReset(passphrase: passphrase)

        // Create a recipient journalist and the user key pair
        let recipient = PublicKeysHelper.shared.testDefaultJournalist!
        let privateSendingQueueSecret = try PrivateSendingQueueSecret.fromSecureRandom()
        let userKeyPair: EncryptionKeypair<User> = try EncryptionKeypair<User>.generateEncryptionKeypair()

        // Create an encrypted message to the recipient journalist
        let encryptedMessage = try await UserToCoverNodeMessageData.createMessage(
            message: "hey",
            messageRecipient: recipient,
            verifiedPublicKeys: verifiedPublicKeys,
            userPublicKey: userKeyPair.publicKey
        )
        let hint = HintHmac(hint: PrivateSendingQueueHmac.hmac(
            secretKey: privateSendingQueueSecret.bytes,
            message: encryptedMessage.asBytes()
        ))
        let realMessage = Message.outboundMessage(
            message: OutboundMessageData(
                recipient: recipient,
                messageText: "hey",
                dateQueued: baseTime,
                hint: hint
            )
        )

        // Add the new message to the current a new version of unlocked secret data
        let messages: Set<Message> = [realMessage]
        let newData = UnlockedSecretData(
            messageMailbox: messages,
            userKey: userKeyPair,
            privateSendingQueueSecret: privateSendingQueueSecret
        )

        // Lock the secretDataRespository and check the enum state is updated to locked
        secretDataRepository.setUnlockedDataForTesting(unlockedData: newData)
        try await secretDataRepository.lock()
        if case .lockedSecretData = secretDataRepository.secretData {
            XCTAssertTrue(true)
        } else {
            XCTFail("secretDataRepository was not locked after locking")
        }

        // BEFORE
        //
        // Move forward to the expiry time minus one second (the messages should survive locking
        // and subsequent unlocking)
        try TestingBridge.setCurrentTimeOverride(override: expiryTime.minusSeconds(1))

        _ = try await secretDataRepository.unlock(passphrase: passphrase)
        guard case let .unlockedSecretData(unlockedData: unlockedData) = secretDataRepository.secretData else {
            XCTFail("secretDataRepository was not unlocked after unlocking")
            return
        }
        XCTAssertTrue(unlockedData.messageMailbox.count == 1)

        _ = try await secretDataRepository.lock()
        _ = try await secretDataRepository.unlock(passphrase: passphrase)
        guard case let .unlockedSecretData(unlockedData: unlockedData) = secretDataRepository.secretData else {
            XCTFail("secretDataRepository was not unlocked after unlocking")
            return
        }
        XCTAssertTrue(unlockedData.messageMailbox.count == 1)

        // AFTER
        //
        // Move forward to the expiry time plus one second (the messages should appear once, but
        // then deleted on save so that they do not appear on te next unlock)
        try TestingBridge.setCurrentTimeOverride(override: expiryTime.plusSeconds(1))

        _ = try await secretDataRepository.unlock(passphrase: passphrase)
        guard case let .unlockedSecretData(unlockedData: unlockedData) = secretDataRepository.secretData else {
            XCTFail("secretDataRepository was not unlocked after unlocking")
            return
        }
        XCTAssertTrue(unlockedData.messageMailbox.count == 1)

        _ = try await secretDataRepository.lock()
        _ = try await secretDataRepository.unlock(passphrase: passphrase)
        guard case let .unlockedSecretData(unlockedData: unlockedData) = secretDataRepository.secretData else {
            XCTFail("secretDataRepository was not unlocked after unlocking")
            return
        }
        XCTAssertTrue(unlockedData.messageMailbox.count == 0)
    }

    func testCallbacks() async throws {
        let publicDataRepository = PublicDataRepository(StaticConfig.devConfig, urlSession: URLSession.shared)
        let secretDataRepository = SecretDataRepository(publicDataRepository: publicDataRepository)
        let verifiedPublicKeys = PublicKeysHelper.shared.testKeys
        publicDataRepository.injectVerifiedPublicKeysForTesting(verifiedPublicKeys: verifiedPublicKeys)

        guard (try? publicDataRepository.getCoverMessageFactory()) != nil else {
            XCTFail("Unable to make cover message")
            return
        }

        let encryptedStorageFilePath = try StorageManager.shared.getFullUrl(file: CoverDropFiles.encryptedStorage).path

        let millisecondsSinceLastModified = {
            let fileAttributes = try FileManager.default.attributesOfItem(atPath: encryptedStorageFilePath)
            let lastModified = fileAttributes[.modificationDate] as? Date
            let currentTime = NSDate()
            return currentTime.timeIntervalSince(lastModified!) * 1000
        }

        // Initial onBackgroundCall without any encrypted storage should not create a file
        try await secretDataRepository.onDidEnterBackground()
        XCTAssertFalse(FileManager.default.fileExists(atPath: encryptedStorageFilePath))

        // Initial onAppStartCall should create a file with a recent timestamp
        try await secretDataRepository.onAppStart()
        XCTAssertTrue(FileManager.default.fileExists(atPath: encryptedStorageFilePath))
        XCTAssertLessThan(try millisecondsSinceLastModified(), 500)

        // After waiting for 1 second, the timestamp should be older
        sleep(1)
        XCTAssertGreaterThan(try millisecondsSinceLastModified(), 1000)

        // Calling onAppStart again should update the timestamp
        try await secretDataRepository.onAppStart()
        XCTAssertLessThan(try millisecondsSinceLastModified(), 500)

        // Waiting and then calling onDidEnterBackground should update the timestamp as well
        sleep(1)
        XCTAssertGreaterThan(try millisecondsSinceLastModified(), 1000)
        try await secretDataRepository.onDidEnterBackground()
        XCTAssertLessThan(try millisecondsSinceLastModified(), 500)
    }

    func testDeleteVault() async throws {
        let publicDataRepository = PublicDataRepository(StaticConfig.devConfig, urlSession: URLSession.shared)
        let secretDataRepository = SecretDataRepository(publicDataRepository: publicDataRepository)
        let verifiedPublicKeys = PublicKeysHelper.shared.testKeys
        publicDataRepository.injectVerifiedPublicKeysForTesting(verifiedPublicKeys: verifiedPublicKeys)

        guard (try? publicDataRepository.getCoverMessageFactory()) != nil else {
            XCTFail("Unable to make cover message")
            return
        }

        let encryptedStorageFilePath = try StorageManager.shared.getFullUrl(file: CoverDropFiles.encryptedStorage).path

        try await secretDataRepository.onAppStart()

        // setup secret data repository with new passphrase
        let passphrase = ValidPassword(password: "external jersey squeeze")
        try await secretDataRepository.createOrReset(passphrase: passphrase)

        // Check that the file exists after creating or resetting the vault
        XCTAssertTrue(FileManager.default.fileExists(atPath: encryptedStorageFilePath))

        // Check that we can unlock the vault with our passphrase
        try await secretDataRepository.unlock(passphrase: passphrase)

        // Delete the vault
        try await secretDataRepository.deleteVault()
        XCTAssertTrue(FileManager.default.fileExists(atPath: encryptedStorageFilePath))

        // Unlocking the vault should now fail
        do {
            try await secretDataRepository.unlock(passphrase: passphrase)
            XCTFail("Unlocking the vault should have failed")
        } catch {
            // Expected error
            XCTAssertTrue(true)
        }
    }
}
