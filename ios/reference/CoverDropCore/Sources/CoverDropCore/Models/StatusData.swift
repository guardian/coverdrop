import Foundation

public struct StatusData: Codable, Equatable {
    public init(status: StatusType, description: String, timestamp: RFC3339DateTimeString, isAvailable: Bool) {
        self.status = status
        self.description = description
        self.timestamp = timestamp
        self.isAvailable = isAvailable
    }

    public var status: StatusType
    public var description: String
    public var timestamp: RFC3339DateTimeString
    public var isAvailable: Bool

    enum CodingKeys: String, CodingKey {
        case isAvailable = "is_available"
        case status
        case description
        case timestamp
    }
}

public enum StatusType: String, Codable {
    case noInformation = "NO_INFORMATION"
    case available = "AVAILABLE"
    case unavailable = "UNAVAILABLE"
    case degradedPerformace = "DEGRADED_PERFORMANCE"
    case scheduledMaintenance = "SCHEDULED_MAINTENANCE"
}
