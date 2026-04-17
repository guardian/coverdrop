@testable import CoverDropCore
import CryptoKit
import Sodium
import XCTest

struct TestStorageState {
    var storage: Storage
    var blobLength: Int
    var lastModifiedTimestamp: Date?
    var createdTimestamp: Date?
}

final class EncryptedStorageTests: XCTestCase {
    let config: StaticConfig = .devConfig
    let instance = EncryptedStorage.createForTesting()

    /// Before any run, clean all data from storage
    override func setUp() async throws {
        try StorageManager.shared.deleteFile(file: CoverDropFiles.encryptedStorage)
    }

    func testOnFreshInstallUnlockWithoutStorageFails() async throws {
        let passphrase = PasswordGenerator.shared.generate(wordCount: config.passphraseWordCount)

        var caughtError: Error?
        do {
            _ = try await instance.unlockStorageWithPassphrase(passphrase: passphrase)
        } catch {
            caughtError = error
        }
        XCTAssertNotNil(caughtError)
        XCTAssertEqual(caughtError as? EncryptedStorageError, EncryptedStorageError.storageFileMissing)
    }

    func testOnAppStartCreatesStorage() async throws {
        try await instance.onAppStart(config: config)
        let storage = try readTestStorageState()

        guard let storage = storage else { throw "storage was nil" }
        XCTAssertTrue(storage.storage.blobData.count > 0)
    }

    func testEncryptDecryptRoundTrip() async throws {
        try await instance.onAppStart(config: config)
        let passphrase = PasswordGenerator.shared.generate(wordCount: config.passphraseWordCount)

        // Store data in the encrypted storage
        let session1 = try await instance.createOrResetStorageWithPassphrase(passphrase: passphrase)
        let data1 = try UnlockedSecretData.createEmpty()
        try await instance.updateStorageOnDisk(session: session1, state: data1)

        // Read data from the encrypted storage using our previous session
        let data2 = try await instance.loadStorageFromDisk(session: session1)
        XCTAssertEqual(data1, data2)

        // Read data from the encrypted storage using a new session
        let session2 = try await instance.unlockStorageWithPassphrase(passphrase: passphrase)
        let data3 = try await instance.loadStorageFromDisk(session: session2)
        XCTAssertEqual(data1, data3)
    }

    func testDecryptionFailsWithWrongPassphrases() async throws {
        try await instance.onAppStart(config: config)
        let passphrase = PasswordGenerator.shared.generate(wordCount: config.passphraseWordCount)

        // Store data in the encrypted storage
        let session = try await instance.createOrResetStorageWithPassphrase(passphrase: passphrase)
        let data1 = try UnlockedSecretData.createEmpty()
        try await instance.updateStorageOnDisk(session: session, state: data1)

        // Try decrypting with a different passphrase
        let passphrase2 = PasswordGenerator.shared.generate(wordCount: config.passphraseWordCount)
        var caughtError: Error?
        do {
            _ = try await instance.unlockStorageWithPassphrase(passphrase: passphrase2)
        } catch {
            caughtError = error
        }
        XCTAssertNotNil(caughtError)
        XCTAssertEqual(caughtError as? EncryptedStorageError, EncryptedStorageError.decryptionFailed)
    }

    func testOnAppStartUpdatesStorageTimestamps() async throws {
        try await instance.onAppStart(config: config)
        let storage1 = try readTestStorageState()

        // just reading after a while yields the same timestamps
        sleep(1)
        let storage1b = try readTestStorageState()

        guard let storage1 = storage1 else { throw "storage1 was nil" }
        guard let storage1b = storage1b else { throw "storage1b was nil" }
        XCTAssertEqual(storage1.createdTimestamp, storage1b.createdTimestamp)
        XCTAssertEqual(storage1.lastModifiedTimestamp, storage1b.lastModifiedTimestamp)

        // reading after the next onAppStart yields new timestamps
        sleep(1)
        try await instance.onAppStart(config: config)
        let storage2 = try readTestStorageState()

        guard let storage2 = storage2 else { throw "storage was nil" }
        XCTAssertGreaterThan(try XCTUnwrap(storage2.createdTimestamp), try XCTUnwrap(storage1.createdTimestamp))
        XCTAssertGreaterThan(
            try XCTUnwrap(storage2.lastModifiedTimestamp),
            try XCTUnwrap(storage1.lastModifiedTimestamp)
        )
    }

    func testOnAppStartDoesNotChangeStorageContent() async throws {
        try await instance.onAppStart(config: config)
        let storage1 = try readTestStorageState()

        try await instance.onAppStart(config: config)
        let storage2 = try readTestStorageState()

        XCTAssertEqual(storage1?.storage.blobData, storage2?.storage.blobData)
        XCTAssertEqual(storage1?.storage.salt, storage2?.storage.salt)
    }

    func testSecondStoreOverwritesContents() async throws {
        try await instance.onAppStart(config: config)
        let passphrase = PasswordGenerator.shared.generate(wordCount: config.passphraseWordCount)

        // Store data in the encrypted storage
        let session1 = try await instance.createOrResetStorageWithPassphrase(passphrase: passphrase)
        let data1 = try UnlockedSecretData.createEmpty()
        try await instance.updateStorageOnDisk(session: session1, state: data1)

        // Overwrite with new data
        let session2 = try await instance.unlockStorageWithPassphrase(passphrase: passphrase)
        let data2 = try UnlockedSecretData.createEmpty()
        try await instance.updateStorageOnDisk(session: session2, state: data2)

        // Read data from the encrypted storage
        let session3 = try await instance.unlockStorageWithPassphrase(passphrase: passphrase)
        let data3 = try await instance.loadStorageFromDisk(session: session3)

        XCTAssertEqual(data2, data3) // matches most-recent written data
        XCTAssertNotEqual(data1, data3) // does not match originally written data
    }

    func testStoredFileAlwaysSameSize() async throws {
        // Check after initial onAppStart call
        try await instance.onAppStart(config: config)
        let size1 = try XCTUnwrap(try readTestStorageState()?.blobLength)

        // Check after resetting the storage
        let passphrase = PasswordGenerator.shared.generate(wordCount: config.passphraseWordCount)
        let session = try await instance.createOrResetStorageWithPassphrase(passphrase: passphrase)
        let size2 = try XCTUnwrap(try readTestStorageState()?.blobLength)

        // Check after updating the storage
        try await instance.updateStorageOnDisk(
            session: session,
            state: UnlockedSecretData.createEmpty()
        )
        let size3 = try XCTUnwrap(try readTestStorageState()?.blobLength)

        XCTAssertEqual(size1, size2)
        XCTAssertEqual(size1, size3)
    }

    func testOverfullMessageBoxesAreTrimmedToMaintainSizeInvariant() async throws {
        let passphrase = PasswordGenerator.shared.generate(wordCount: config.passphraseWordCount)
        let session = try await instance.createOrResetStorageWithPassphrase(passphrase: passphrase)

        let emptyState = try UnlockedSecretData.createEmpty()
        try await instance.updateStorageOnDisk(session: session, state: emptyState)
        let emptyStorage = try XCTUnwrap(try readTestStorageState())

        let stateAtFewMessages = makeMailboxState(from: emptyState, messageCount: 100)
        try await instance.updateStorageOnDisk(session: session, state: stateAtFewMessages)
        let storageAtFewMessages = try XCTUnwrap(try readTestStorageState())

        // Note: the threshold is at around 1220 messages, we choose message counts that are
        // far away so that this test is robust against minor encoding changes
        let stateAtOverfull = makeMailboxState(from: emptyState, messageCount: 1500)
        try await instance.updateStorageOnDisk(session: session, state: stateAtOverfull)
        let storageAtOverfull = try XCTUnwrap(try readTestStorageState())

        // All unencrypted state must not be larger than the padding target
        XCTAssertLessThanOrEqual(emptyState.asUnencryptedBytes().count, EncryptedStorage.storagePaddingToSize)
        XCTAssertLessThanOrEqual(stateAtFewMessages.asUnencryptedBytes().count, EncryptedStorage.storagePaddingToSize)
        XCTAssertLessThanOrEqual(stateAtOverfull.asUnencryptedBytes().count, EncryptedStorage.storagePaddingToSize)

        // And on-disk storage size should always be the same
        XCTAssertEqual(emptyStorage.blobLength, storageAtFewMessages.blobLength)
        XCTAssertEqual(emptyStorage.blobLength, storageAtOverfull.blobLength)
    }

    func testRemovingOldestMessage() async throws {
        let state = try UnlockedSecretData.createEmpty()

        // Test data
        let now = Date()
        let journalist = JournalistData(
            recipientId: "test-journalist",
            displayName: "Journalist Test",
            isDesk: false,
            recipientDescription: "test sender",
            tag: RecipientTag(tag: [1, 2, 3, 4]),
            visibility: .visible
        )

        // Add an initial message from the user
        try await state.addMessage(
            message: .outboundMessage(
                message: OutboundMessageData(
                    recipient: journalist,
                    messageText: "Hello",
                    dateQueued: now.minusSeconds(3600),
                    hint: HintHmac(hint: [0, 0, 0, 0])
                )
            )
        )

        // Followed by a reply of the journalist
        await state.addMessage(message:
            .incomingMessage(message: .textMessage(message: IncomingMessageData(
                sender: journalist,
                messageText: "Hi there",
                dateReceived: now
            ))))

        // Initially, we can retrieve both
        XCTAssertEqual(state.messageMailbox.count, 2)

        // After removing the oldest, we only have the journalist reply
        let removalSucccess1 = await state.removeOldestMessage()
        XCTAssertTrue(removalSucccess1)
        XCTAssertEqual(state.messageMailbox.count, 1)
        XCTAssert(state.messageMailbox.first?.getDate() == now)

        // After another removal, it is empty
        let removalSucccess2 = await state.removeOldestMessage()
        XCTAssertTrue(removalSucccess2)
        XCTAssertTrue(state.messageMailbox.isEmpty)

        // Subsequent removal attempts fail
        let removalSucccess3 = await state.removeOldestMessage()
        XCTAssertFalse(removalSucccess3)
    }

    /// Reads the current on-disk storage state for inspection by the individual tests.
    ///  If the file is missing, `nil` is returned
    private func readTestStorageState() throws -> TestStorageState? {
        let fileURL = try StorageManager.shared.getFullUrl(file: CoverDropFiles.encryptedStorage)

        guard let readData: Data = try? Data(contentsOf: fileURL) else { return nil }

        let storage: Storage = try JSONDecoder().decode(Storage.self, from: readData)
        let attributes: [FileAttributeKey: Any] = try FileManager.default.attributesOfItem(atPath: fileURL.path)
        return TestStorageState(
            storage: storage,
            blobLength: storage.blobData.count,
            lastModifiedTimestamp: attributes[FileAttributeKey.modificationDate] as? Date,
            createdTimestamp: attributes[FileAttributeKey.creationDate] as? Date
        )
    }

    private func makeMailboxState(
        from seedState: UnlockedSecretData,
        messageCount: Int
    ) -> UnlockedSecretData {
        let sender = JournalistData(
            recipientId: "test-journalist",
            displayName: "Journalist Test",
            isDesk: false,
            recipientDescription: "test sender",
            tag: RecipientTag(tag: [1, 2, 3, 4]),
            visibility: .visible
        )

        let seed = Bytes(repeating: 0x00, count: 32)
        let messages = Set((0 ..< messageCount).map { counter in
            // Generate pseudorandom hex string of length 128 with around ~64 byte of entropy
            let payload = Sodium().keyDerivation.derive(
                secretKey: seed,
                index: UInt64(counter),
                length: 128 / 2,
                context: "payload"
            )!.hexStr!
            return Message.incomingMessage(
                message: .textMessage(
                    message: IncomingMessageData(
                        sender: sender,
                        messageText: payload,
                        dateReceived: Date(timeIntervalSince1970: Double(counter)),
                        deadDropId: counter
                    )
                )
            )
        })

        return UnlockedSecretData(
            messageMailbox: messages,
            userKey: seedState.userKey,
            privateSendingQueueSecret: seedState.privateSendingQueueSecret
        )
    }
}
