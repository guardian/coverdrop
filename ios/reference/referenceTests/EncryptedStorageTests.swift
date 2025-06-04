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
        XCTAssertEqual(caughtError as! EncryptedStorageError, EncryptedStorageError.storageFileMissing)
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
        XCTAssertEqual(caughtError as! EncryptedStorageError, EncryptedStorageError.decryptionFailed)
    }

    func testOnAppStartUpdatesStorageTimestamps() async throws {
        try await instance.onAppStart(config: config)
        let storage1 = try readTestStorageState()

        // just reading after a while yields the same timestamps
        sleep(1)
        let storage1b = try readTestStorageState()

        guard let storage1 = storage1 else { throw "storage1 was nil" }
        guard let storage1b = storage1b else { throw "storage1b was nil" }
        XCTAssertEqual(storage1.createdTimestamp!, storage1b.createdTimestamp!)
        XCTAssertEqual(storage1.lastModifiedTimestamp!, storage1b.lastModifiedTimestamp!)

        // reading after the next onAppStart yields new timestamps
        sleep(1)
        try await instance.onAppStart(config: config)
        let storage2 = try readTestStorageState()

        guard let storage2 = storage2 else { throw "storage was nil" }
        XCTAssertGreaterThan(storage2.createdTimestamp!, storage1.createdTimestamp!)
        XCTAssertGreaterThan(storage2.lastModifiedTimestamp!, storage1.lastModifiedTimestamp!)
    }

    func testOnAppStartDoesNotChangeStorageContent() async throws {
        try await instance.onAppStart(config: config)
        let storage1 = try readTestStorageState()

        try await instance.onAppStart(config: config)
        let storage2 = try readTestStorageState()

        XCTAssertEqual(storage1!.storage.blobData, storage2!.storage.blobData)
        XCTAssertEqual(storage1!.storage.salt, storage2!.storage.salt)
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
        let size1 = try readTestStorageState()!.blobLength

        // Check after resetting the storage
        let passphrase = PasswordGenerator.shared.generate(wordCount: config.passphraseWordCount)
        let session = try await instance.createOrResetStorageWithPassphrase(passphrase: passphrase)
        let size2 = try readTestStorageState()!.blobLength

        // Check after updating the storage
        try await instance.updateStorageOnDisk(
            session: session,
            state: UnlockedSecretData.createEmpty()
        )
        let size3 = try readTestStorageState()!.blobLength

        XCTAssertEqual(size1, size2)
        XCTAssertEqual(size1, size3)
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
}
