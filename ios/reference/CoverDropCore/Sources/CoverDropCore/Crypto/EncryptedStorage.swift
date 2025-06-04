import CryptoKit
import Foundation
import RainbowSloth
import Sodium

/// The information to be stored on disk
public struct Storage: Codable {
    /// Salt used for the `RainbowSloth` password hashing algorithm
    var salt: [UInt8]

    /// The encrypted AES-GCM ciphertext
    var blobData: [UInt8]
}

/// The `EncryptedStorageSession` the derived key so that subsequent operations are faster
public struct EncryptedStorageSession {
    var cachedKey: [UInt8]
    var salt: [UInt8]
}

enum EncryptedStorageError: Error {
    case storageFileMissing
    case storageFileDeserializationFailed
    case encryptionFailed
    case decryptionFailed
}

/// The `EncryptedStorage` encrypts the mailbox content using a key that is derived using the Sloth library.
/// The Sloth library (and its iOS variant `RainbowSloth`) store a secret inside the Secure Enclave to
/// effectively rate-limit the guess rate of passphrases.
public class EncryptedStorage {
    public static let storagePaddingToSize = 512 * 1024 // 512 KiB

    /// The parameter N for RainbowSloth is chosen based on the paper and translates to at least ~1 seconds
    let rainbowSloth = RainbowSloth(withN: 200)

    let rainbowSlothKeyHandle = "coverdop"
    let xchacha20poly1305KeySize = 32
    let file = CoverDropFiles.encryptedStorage

    fileprivate init() {}

    /// To be called on every app start. If no storage exists, a new one is created with an undisclosed passphrase. If
    /// one already exists, its last-modified date is updated.
    /// - Returns: `Storage` object with encrypted `blob`
    /// - Throws: if touching or creating storage fails
    public func onAppStart(config: CoverDropConfig) async throws {
        if StorageManager.shared.doesFileExist(file: file) {
            // If there is an existing storage, update its creation and last-modified timestamps
            try StorageManager.shared.touchFile(file: file)
        } else {
            // Otherwise, there is no storage yet and we create it with a random passphrase
            let passphrase = PasswordGenerator.shared.generate(wordCount: config.passphraseWordCount)
            _ = try await createOrResetStorageWithPassphrase(passphrase: passphrase)
        }
    }

    public func onDidEnterBackground() throws {
        if StorageManager.shared.doesFileExist(file: file) {
            try StorageManager.shared.touchFile(file: file)
        }
    }

    /// This will update the modification date on the on-Disk storage file to the current datetime.
    /// To make sure this is done correctly we set the `modificationDate`and `creationDate` attributes, and then read
    /// the attribute again
    /// - Parameters:
    ///  - fileUrl: the `URL` of the storage file to write to
    /// - Throws: if attribute setting fails
    func touchExistingStorage(fileUrl: URL) throws {
        let date = NSDate()
        try FileManager.default.setAttributes([FileAttributeKey.modificationDate: date], ofItemAtPath: fileUrl.path)
        try FileManager.default.setAttributes([FileAttributeKey.creationDate: date], ofItemAtPath: fileUrl.path)
    }

    /// Creates or resets the storage with a new passphrase. This will irrecoverly remove all existing data.
    /// - Parameters:
    ///   - passphrase: the new passphrase created by the user
    /// - Returns: `EncryptedStorageSession` object
    /// - Throws: if the writing the storage fails
    public func createOrResetStorageWithPassphrase(passphrase: ValidPassword) async throws
        -> EncryptedStorageSession {
        // Generate a new active session with the new passphrase; this resets the SE key
        let (slothStorageState, kUser) = try rainbowSloth.keygen(
            pw: passphrase.password,
            handle: rainbowSlothKeyHandle,
            outputLength: xchacha20poly1305KeySize
        )
        let session = EncryptedStorageSession(cachedKey: [UInt8](kUser), salt: [UInt8](slothStorageState.salt))

        // Create an initial empty state
        let emptyState = try UnlockedSecretData.createEmpty()

        // Store on disk using our newly derived session
        try await updateStorageOnDisk(
            session: session,
            state: emptyState
        )

        return session
    }

    /// Writes the new state to the storage using the given `EncryptedStorageSession`.
    ///  - Parameters:
    ///   - session: an `EncryptedStorageSession` previously derived via`createOrResetStorageWithPassphrase` or
    /// `unlockStorageWithPassphrase`
    ///   - state: a `UnlockedSecretData` with the new state we want to update storage with, Any existing data will be
    /// overwritten.
    ///  - Throws: if password derivation, key loading, encryption, json encoding or file writing fail
    public func updateStorageOnDisk(
        session: EncryptedStorageSession,
        state: UnlockedSecretData
    ) async throws {
        // Pad the new state to a fixed size
        var statePadded: [UInt8] = state.asUnencryptedBytes()
        Sodium().utils.pad(bytes: &statePadded, blockSize: EncryptedStorage.storagePaddingToSize)

        // Encrypt using an AEAD algorithm.
        // This sets an IV/nonce internally making it CPA and CCA secure.
        // The nonce is included in the returned ciphertext.
        guard let ciphertext: Bytes = Sodium().aead.xchacha20poly1305ietf.encrypt(
            message: statePadded,
            secretKey: session.cachedKey
        ) else { throw EncryptionError.failedToEncrypt }

        // create the `Storage` object that encodes all our information that we need to persist on disk
        let storage = Storage(salt: session.salt, blobData: ciphertext)

        let jsonEncoder = JSONEncoder()
        jsonEncoder.outputFormatting = .sortedKeys
        let jsonData = try jsonEncoder.encode(storage)
        try StorageManager.shared.writeFile(
            file: file,
            data: Array(jsonData)
        )
    }

    /// Derives a session with the provided passphrase that allows reading and writing to the storage.
    /// - Parameters:
    ///   - passphrase: the new passphrase created by the user
    /// - Returns: `EncryptedStorageSession` object
    /// - Throws: if the unlocking fails; this can be due to a wrong passphrase or a tampered file
    public func unlockStorageWithPassphrase(passphrase: ValidPassword) async throws -> EncryptedStorageSession {
        // retrieve our `Storage` information from disk
        if !StorageManager.shared.doesFileExist(file: file) {
            throw EncryptedStorageError.storageFileMissing
        }

        let readData = try StorageManager.shared.readFile(file: file)

        guard let storage: Storage = try? JSONDecoder().decode(Storage.self, from: Data(readData)) else {
            throw EncryptedStorageError.storageFileDeserializationFailed
        }

        // rederive the encryption key `k` using RainbowSloth
        let slothPersistedState = RainbowSlothStorageState(handle: rainbowSlothKeyHandle, salt: storage.salt)
        let kUser = try rainbowSloth.derive(
            storageState: slothPersistedState,
            pw: passphrase.password,
            outputLength: xchacha20poly1305KeySize
        )

        let session = EncryptedStorageSession(cachedKey: [UInt8](kUser), salt: storage.salt)

        // Try decrypting... this will fail both when the passphrase is wrong or the file has been tampered with.
        if Sodium().aead.xchacha20poly1305ietf.decrypt(
            nonceAndAuthenticatedCipherText: storage.blobData,
            secretKey: session.cachedKey
        ) == nil {
            throw EncryptedStorageError.decryptionFailed
        }

        return session
    }

    /// Reads the storage from disk. Where applicable the secure element is used.
    /// - Parameters:
    ///   - session: an `EncryptedStorageSession` previously derived via`createOrResetStorageWithPassphrase` or
    /// `unlockStorageWithPassphrase`
    /// - Returns: `UnlockedSecretData` object
    /// - Throws: If the storage cannot be decrypted; this can be due to a wrong passphrase or a tamered file
    public func loadStorageFromDisk(session: EncryptedStorageSession) async throws -> UnlockedSecretData {
        // retrieve our `Storage` information from disk
        let readData = try StorageManager.shared.readFile(file: file)
        let storage: Storage = try JSONDecoder().decode(Storage.self, from: Data(readData))

        // Try decrypting... this will fail both when the passphrase is wrong or the file has been tampered with.
        guard var plaintext: Bytes = Sodium().aead.xchacha20poly1305ietf.decrypt(
            nonceAndAuthenticatedCipherText: storage.blobData,
            secretKey: [UInt8](session.cachedKey)
        ) else { throw EncryptedStorageError.decryptionFailed }

        // Unpad and decode
        Sodium().utils.unpad(bytes: &plaintext, blockSize: EncryptedStorage.storagePaddingToSize)
        return try UnlockedSecretData.fromUnencryptedBytes(bytes: plaintext)
    }

    /// Named initializer to highlight that this should only be created from the secret data repository.
    /// Creating a parallel `EncryptedStorage` instance can lead to race conditions and data loss.
    static func createForSecretDataRepository() -> EncryptedStorage {
        return EncryptedStorage()
    }

    /// Only for testing purposes. This should not never be called in production code.
    static func createForTesting() -> EncryptedStorage {
        return EncryptedStorage()
    }
}
