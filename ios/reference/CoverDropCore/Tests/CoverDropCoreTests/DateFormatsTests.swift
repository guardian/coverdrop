@testable import CoverDropCore
import XCTest

final class DateFormatsTests: XCTestCase {
    func testMicroDateDecoding() async throws {
        let microSecondsDate = "2023-04-24T16:04:59.389866670Z"
        XCTAssertNotNil(DateFormats.validateDate(date: microSecondsDate))
    }

    func testSecondsDateDecoding() async throws {
        let secondsOnlyDate = "2023-04-08T12:00:00Z"
        XCTAssertNotNil(DateFormats.validateDate(date: secondsOnlyDate))
    }
}
