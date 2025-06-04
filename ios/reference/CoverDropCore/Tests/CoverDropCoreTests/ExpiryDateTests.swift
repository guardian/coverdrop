@testable import CoverDropCore
import Sodium
import XCTest

// This tests the 3 expiry statuses for messages
final class ExpiryDateTests: XCTestCase {
    func testExpiry() async throws {
        let now = DateFunction.currentTime()

        let messageSend1DayAgo = try now.minusSeconds(1 * 24 * 3600)
        let expiredStatus1 = try getExpiryState(messageDate: messageSend1DayAgo)
        XCTAssertEqual(expiredStatus1, .fresh)

        let messageSend13DaysAgo = try now.minusSeconds(13 * 24 * 3600)
        let expiredStatus2 = try getExpiryState(messageDate: messageSend13DaysAgo)
        XCTAssertEqual(expiredStatus2, .soonToBeExpired(expiryCountdownString: "23h"))

        let messageSend15DaysAgo = try now.minusSeconds(15 * 24 * 3600)
        let expiredStatus3 = try getExpiryState(messageDate: messageSend15DaysAgo)
        XCTAssertEqual(expiredStatus3, .expired)
    }
}
