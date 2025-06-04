import Foundation

public struct DeadDropData: Codable {
    public var deadDrops: [DeadDrop]

    enum CodingKeys: String, CodingKey {
        case deadDrops = "dead_drops"
    }
}

// This is used to store the most recent succesufully loaded dead drop Id
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
    public var cert: HexEncodedString?
    public var signature: HexEncodedString?

    public func hash(into hasher: inout Hasher) {
        hasher.combine(id)
        hasher.combine(createdAt.date)
        hasher.combine(data.bytes)
        hasher.combine(cert?.bytes ?? [])
        return hasher.combine(signature?.bytes ?? [])
    }

    enum CodingKeys: String, CodingKey {
        case id
        case createdAt = "created_at"
        case data
        case cert
        case signature
    }
}
