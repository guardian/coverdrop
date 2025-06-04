@testable import CoverDropCore
import Sodium
import XCTest

final class Base64Test: XCTestCase {
    func testBase64EncodeDecodeWithEmptyString() throws {
        let bytes: [UInt8] = []

        let encoded = bytes.base64Encode()!
        XCTAssertEqual(encoded, "")

        let decoded = encoded.base64Decode()
        XCTAssertEqual(decoded, bytes)
    }

    func testBase64EncodeDecodeWithString() throws {
        let bytes = "hello".asUnencryptedBytes()

        let encoded = bytes.base64Encode()!
        XCTAssertFalse(encoded.isEmpty)

        let decoded = encoded.base64Decode()
        XCTAssertEqual(decoded, bytes)
    }

    func testJsonDecode() throws {
        let data = try DeadDropDataHelper.shared.readLoadDeadDropJson()
        _ = try JSONDecoder().decode(DeadDropData.self, from: data)
    }
}
