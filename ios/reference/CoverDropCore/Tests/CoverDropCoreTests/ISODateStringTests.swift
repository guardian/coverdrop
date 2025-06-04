@testable import CoverDropCore
import Sodium
import XCTest

final class ISODateStringTest: XCTestCase {
    func testJsonDecode() throws {
        let data = try DeadDropDataHelper.shared.readLoadDeadDropJson()
        _ = try JSONDecoder().decode(DeadDropData.self, from: data)
    }
}
