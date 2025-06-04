import Foundation

public enum DateFormats {
    /// We can recieve dates in different string formats
    /// ValidateDate supports:
    ///  iso3339 dates with microseconds precision such as "2023-04-24T16:04:59.389866670Z"
    ///  is03339 dates with second only precision such as "2023-04-08T12:00:00Z"
    ///
    public static func validateDate(date: String) -> Date? {
        let dateFormatter = isoDateFormatter()
        let fractionalDate = dateFormatter.date(from: date)
        let nonFractionalDate = iso8601DateFormatter().date(from: date)
        if fractionalDate != nil {
            return fractionalDate
        } else if nonFractionalDate != nil {
            return nonFractionalDate
        } else {
            return nil
        }
    }

    public static func iso8601DateFormatter() -> ISO8601DateFormatter {
        let dateFormatter = ISO8601DateFormatter()
        dateFormatter.formatOptions = [
            .withInternetDateTime
        ]
        return dateFormatter
    }

    public static func isoDateFormatter() -> DateFormatter {
        let dateFormatter = DateFormatter()
        dateFormatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSSSSSZ"
        return dateFormatter
    }
}
