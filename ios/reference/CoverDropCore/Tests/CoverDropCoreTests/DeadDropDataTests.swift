@testable import CoverDropCore
import XCTest

final class DeadDropDataTests: XCTestCase {
    func testDataJsonDecoding() async throws {
        let data = try DeadDropDataHelper.shared.readLocalDataFile()
        XCTAssertTrue(data.deadDrops.count == 1)
    }
}
