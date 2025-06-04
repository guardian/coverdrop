import Foundation

///  An ISODateString is a iso3339 date string with microseconds precision such as "2023-04-24T16:04:59.389866670Z"
///  or with second only precision such as "2023-04-08T12:00:00Z"
public struct RFC3339DateTimeString: Codable, Equatable, Comparable {
    public static func < (lhs: RFC3339DateTimeString, rhs: RFC3339DateTimeString) -> Bool {
        lhs.date < rhs.date
    }

    public init(date: Date) {
        self.date = date
    }

    public var date: Date

    public var epochSeconds: Int64 {
        Int64(date.timeIntervalSince1970)
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let string = try container.decode(String.self)
        guard let dateString = DateFormats.validateDate(date: string) else {
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "ISODateString does not contain a valid date string"
            )
        }
        date = dateString
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()
        let dateFormat = DateFormats.isoDateFormatter()
        let dateString = dateFormat.string(from: date)
        try container.encode(dateString)
    }
}
