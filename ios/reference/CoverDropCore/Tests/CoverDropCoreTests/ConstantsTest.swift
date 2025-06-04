@testable import CoverDropCore
import Sodium
import XCTest

final class ConstantsTest: XCTestCase {
    func testConstantsPresent() throws {
        XCTAssertGreaterThan(Constants.userToCovernodeEncryptedMessageLen, Constants.messagePaddingLen)
    }
}
