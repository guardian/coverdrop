@testable import CoverDropCore
import XCTest

final class DateFormatsTests: XCTestCase {
    func testMicroDateDecoding() {
        let microSecondsDate = "2023-04-24T16:04:59.389866670Z"
        XCTAssertNotNil(DateFormats.validateDate(date: microSecondsDate))
    }

    func testSecondsDateDecoding() {
        let secondsOnlyDate = "2023-04-08T12:00:00Z"
        XCTAssertNotNil(DateFormats.validateDate(date: secondsOnlyDate))
    }

    func testMicrosecondStringPadsMillisecondPrecisionDeadDropTimestamps() throws {
        // Dead drop timestamps are persisted with only the first three fractional digits set
        let date = try XCTUnwrap(DateFormats.validateDate(date: "2023-04-24T16:04:59.389000Z"))
        XCTAssertEqual(date.rfc3339MicrosecondString, "2023-04-24T16:04:59.389000Z")
    }

    func testMicrosecondStringRoundTripsWithoutLosingAMillisecond() throws {
        // Dates parsed from strings are not exactly representable as a `Double`
        for millisecond in 0 ..< 1000 {
            let string = String(format: "2023-04-24T16:04:59.%03d000Z", millisecond)
            let date = try XCTUnwrap(DateFormats.validateDate(date: string))
            XCTAssertEqual(date.rfc3339MicrosecondString, string)
        }
    }

    func testMicrosecondStringForEpoch() {
        XCTAssertEqual(Date(timeIntervalSince1970: 0).rfc3339MicrosecondString, "1970-01-01T00:00:00.000000Z")
    }

    func testMicrosecondStringIsUtcRegardlessOfDeviceTimeZone() throws {
        let originalTimeZone = NSTimeZone.default
        defer { NSTimeZone.default = originalTimeZone }

        let date = Date(timeIntervalSince1970: 1_682_352_299.389) // 2023-04-24T16:04:59.389Z
        for identifier in ["Europe/London", "America/New_York", "Asia/Kolkata", "Pacific/Auckland"] {
            NSTimeZone.default = try XCTUnwrap(TimeZone(identifier: identifier))
            XCTAssertEqual(date.rfc3339MicrosecondString, "2023-04-24T16:04:59.389000Z", identifier)
        }
    }

    func testMicrosecondStringOfRFC3339DateTimeString() {
        let value = RFC3339DateTimeString(date: Date(timeIntervalSince1970: 1.5))
        XCTAssertEqual(value.rfc3339MicrosecondString, "1970-01-01T00:00:01.500000Z")
    }
}
