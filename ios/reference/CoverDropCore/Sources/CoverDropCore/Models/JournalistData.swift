import Foundation

public enum JournalistVisibility: Codable {
    case visible
    case hidden
}

public struct JournalistData: Hashable, Codable, Comparable {
    public static func < (lhs: JournalistData, rhs: JournalistData) -> Bool {
        return lhs.recipientId < rhs.recipientId &&
            lhs.displayName < rhs.displayName
    }

    public static func == (lhs: JournalistData, rhs: JournalistData) -> Bool {
        return lhs.recipientId == rhs.recipientId &&
            lhs.displayName == rhs.displayName &&
            lhs.tag == rhs.tag &&
            lhs.isDesk == rhs.isDesk &&
            lhs.visibility == rhs.visibility
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(recipientId)
    }

    public let recipientId: String
    public let displayName: String
    public let isDesk: Bool
    public let recipientDescription: String
    public let tag: RecipientTag
    public let visibility: JournalistVisibility

    public init(
        recipientId: String,
        displayName: String,
        isDesk: Bool,
        recipientDescription: String,
        tag: RecipientTag,
        visibility: JournalistVisibility
    ) {
        self.recipientId = recipientId
        self.displayName = displayName
        self.isDesk = isDesk
        self.recipientDescription = recipientDescription
        self.tag = tag
        self.visibility = visibility
    }
}
