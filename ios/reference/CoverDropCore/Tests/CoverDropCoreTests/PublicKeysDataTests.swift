@testable import CoverDropCore
import XCTest

final class PublicKeysDataTests: XCTestCase {
    func testPublicDataJsonDecoding() async throws {
        let data = try PublicKeysHelper.readLocalKeysFile()
        XCTAssertTrue(data.keys.count == 1)
    }
}
