import Foundation

public enum DateFormats {
    /// We can receive dates in different string formats
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

    /// Formats dates as RFC 3339 strings in the microsecond format.
    /// Note: This function uses the device's time zone and emits an offset e.g. `+0100`.
    /// Note: although the format is microsecond precision (6 digits) the last three digits are always zero.
    /// TODO we should look at migrating timestamps stored in key hierarchy and dead drop data to microsecond
    /// precision to align with the API and Android.
    public static func isoDateFormatter() -> DateFormatter {
        let dateFormatter = DateFormatter()
        // We use the en_US_POSIX locale to ensure that the date format
        // is interpreted correctly regardless of the user's locale settings.
        dateFormatter.locale = Locale(identifier: "en_US_POSIX")
        dateFormatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSSSSSZ"
        return dateFormatter
    }

    /// Formats dates as UTC RFC 3339 strings in the microsecond format the API uses, e.g.
    /// "2023-04-24T16:04:59.389000Z", so that cursors are comparable to the API's UTC `created_at` dates.
    /// `DateFormatter` only resolves milliseconds, so the last three digits are always zero, which matches how
    /// dead drop timestamps are persisted on the device.
    /// The time zone is fixed to UTC and the suffix is a literal `Z`.
    public static func rfc3339MicrosecondFormatter() -> DateFormatter {
        let dateFormatter = DateFormatter()
        dateFormatter.locale = Locale(identifier: "en_US_POSIX")
        dateFormatter.timeZone = TimeZone(secondsFromGMT: 0)
        dateFormatter.dateFormat = "yyyy-MM-dd'T'HH:mm:ss.SSSSSS'Z'"
        return dateFormatter
    }
}

public extension Date {
    /// RFC 3339 string in the microsecond format, e.g. "2023-04-24T16:04:59.389000Z"
    var rfc3339MicrosecondString: String {
        DateFormats.rfc3339MicrosecondFormatter().string(from: self)
    }
}

public extension RFC3339DateTimeString {
    /// RFC 3339 string in the microsecond format, e.g. "2023-04-24T16:04:59.389000Z"
    var rfc3339MicrosecondString: String {
        date.rfc3339MicrosecondString
    }
}
