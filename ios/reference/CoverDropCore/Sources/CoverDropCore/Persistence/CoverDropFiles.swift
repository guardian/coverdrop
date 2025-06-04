import Foundation

enum CoverDropFiles: String, CaseIterable {
    /// The persistent storage of the Private Sending Queue. This is a binary file where messages are regularly
    /// dequeued from the front. By default, cover messages are added automatically to the back of the queue.
    /// Hence, it should always have the same size.
    case privateSendingQueueV2 = "privateSendingQueue_v2"

    /// The encrypted storage based on the Sloth plausibly deniable scheme. This file is managed by
    /// the `EncryptedStorage` class. It is padded to a fixed size and should be "touched" on every
    /// launched.
    case encryptedStorage = "coverdrop"

    /// Cache for the latest processed dead-drop ID
    case deadDropId = "deadDropId.json"

    /// Cache for the dead drops endpoint. This JSON contains the most recent cached elements
    /// and is updated regularly depending on its last modified date.
    case deadDropCache = "deadDrops.json"

    /// Cache for the public keys endpoint. This JSON contains the most recent cached elements
    /// and is updated regularly depending on its last modified date.
    case publicKeysCache = "publicKeys.json"

    /// Cache for the status endpoint. This JSON contains the most recent cached elements
    /// and is updated regularly depending on its last modified date.
    case statusCache = "status.json"
}

extension CoverDropFiles {
    /// The file protection mode for this file. The public ones are not protected, i.e. `none`,
    /// while the private ones are `complete`.
    func getFileProtectionMode() -> FileProtectionType {
        switch self {
        case .privateSendingQueueV2:
            return .none
        case .deadDropId, .deadDropCache, .publicKeysCache, .statusCache:
            return .none
        case .encryptedStorage:
            return .complete
        }
    }
}

extension FileProtectionType {
    /// Converts the `FileProtectionType` to the corresponding `Data.WritingOptions`
    func toDataWritingOptions() throws -> Data.WritingOptions {
        switch self {
        case .none:
            return []
        case .complete:
            return [.completeFileProtection]
        default:
            throw StorageManagerError.unsupportedFileProtection
        }
    }
}
