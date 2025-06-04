@testable import CoverDropCore
import XCTest

final class HumanKeyDigestTests: XCTestCase {
    func testSuccessfullVerification() throws {
        let orgKey = "c941a9beed1c8c945c27b150b5aa725a6366f71900a5e93607ba93254fe8d585".hexStringToBytes()!

        let actual = getHumanReadableDigest(key: orgKey)
        let expected = "jdiH4c 9DO9cT kefiCh OXoQ"
        XCTAssertEqual(actual, expected)
    }
}
