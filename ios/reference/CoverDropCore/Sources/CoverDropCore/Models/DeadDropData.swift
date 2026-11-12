import Foundation

public struct DeadDropData: Codable {
    public var deadDrops: [DeadDrop]

    enum CodingKeys: String, CodingKey {
        case deadDrops = "dead_drops"
    }
}

/// This is used to store the most recent succesufully loaded dead drop Id
public struct DeadDropId: Codable, Comparable {
    public static func < (lhs: DeadDropId, rhs: DeadDropId) -> Bool {
        return lhs.id < rhs.id
    }

    public var id: Int
}

public struct DeadDrop: Codable, Equatable, Hashable {
    public var id: Int
    public var createdAt: RFC3339DateTimeString
    public var data: Base64EncodedString
    public var signature: HexEncodedString

    /// Two dead drops are the same if everything the CoverNode signature authenticates is the same: the data and the
    /// whole-second `createdAt`. The API-assigned `id` and the sub-second part of `createdAt` are not signed, so they
    /// are ignored. Otherwise a malicious API could serve one signed dead drop many times under different ids or
    /// timestamps, and every copy would be treated as a distinct dead drop.
    public static func == (lhs: DeadDrop, rhs: DeadDrop) -> Bool {
        lhs.createdAt.epochSeconds == rhs.createdAt.epochSeconds &&
            lhs.data.bytes == rhs.data.bytes &&
            lhs.signature.bytes == rhs.signature.bytes
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(createdAt.epochSeconds)
        hasher.combine(data.bytes)
        hasher.combine(signature.bytes)
    }

    enum CodingKeys: String, CodingKey {
        case id
        case createdAt = "created_at"
        case data
        case signature
    }
}
