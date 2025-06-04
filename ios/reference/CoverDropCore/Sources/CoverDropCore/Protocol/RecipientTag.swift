import CryptoKit
import Foundation

enum RecipientTagError: Error {
    case hashCollisionOccured
}

public struct RecipientTag: Equatable, Codable {
    let tag: [UInt8]

    public init(tag: [UInt8]) {
        self.tag = tag
    }

    static let recipientTagForCoverMessage: [UInt8] = Array(repeating: 0, count: Constants.recipientTagLen)

    public static func hashJournalistIdentifierToRecipientTag(
        journalistIdentifier: String
    ) throws -> RecipientTag {
        // note: the hash operation here does not need any particular security properties (it should
        // just map pseudo randomly into the output domain to avoid collisions)
        var hasher = SHA256()
        hasher.update(data: journalistIdentifier.asBytes())
        let hash = hasher.finalize()

        let hashBytes: [UInt8] = Array(Data(hash))
        let truncatedHash = Array(hashBytes.prefix(Constants.recipientTagLen))

        // note: this is virtually impossible to happen (2^-32); if it happens, the respective
        // journalist can be given a different identifier
        if truncatedHash == RecipientTag.recipientTagForCoverMessage {
            throw RecipientTagError.hashCollisionOccured
        }
        return RecipientTag(tag: truncatedHash)
    }
}
