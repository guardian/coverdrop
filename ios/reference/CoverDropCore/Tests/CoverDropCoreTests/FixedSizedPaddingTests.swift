@testable import CoverDropCore
import XCTest

class FixedSizedPaddingTests: XCTestCase {
    // MARK: - Core Functionality Tests

    func testRoundTripWithPadding() throws {
        // Tests the complete lifecycle: create -> pad -> restore
        let originalData: [UInt8] = [0x01, 0x02, 0xAA, 0xBB]
        let targetSize = 20

        let padding1 = try FixedSizedPadding(targetSize: targetSize, bytes: originalData)
        let padded = padding1.paddedBytes()

        // Verify padded size
        XCTAssertEqual(padded.count, targetSize)

        // Restore and verify
        let padding2 = try FixedSizedPadding.fromPaddedBytes(padded, targetSize: targetSize)
        XCTAssertEqual(padding2.getBytes(), originalData)
        XCTAssertEqual(padding2.getTargetSize(), targetSize)
    }

    func testEmptyDataRoundTrip() throws {
        // Edge case: zero-length data
        let originalData: [UInt8] = []
        let targetSize = 10

        let padding1 = try FixedSizedPadding(targetSize: targetSize, bytes: originalData)
        let padded = padding1.paddedBytes()

        // Header should indicate zero length
        XCTAssertEqual(Array(padded[0 ... 3]), [0x00, 0x00, 0x00, 0x00])

        let padding2 = try FixedSizedPadding.fromPaddedBytes(padded, targetSize: targetSize)
        XCTAssertEqual(padding2.getBytes().count, 0)
    }

    // MARK: - Error Handling Tests

    func testBytesTooLargeForTargetSize() {
        // Error case: data exceeds available space (target=8 minus header=4 mean data=4)
        let data: [UInt8] = [0x01, 0x02, 0x03, 0x04, 0x05]

        XCTAssertThrowsError(try FixedSizedPadding(targetSize: 8, bytes: data)) { error in
            guard case let FixedSizedPaddingError.invalidSize(expected, actual) = error else {
                XCTFail("Wrong error type")
                return
            }
            XCTAssertEqual(expected, 4)
            XCTAssertEqual(actual, 5)
        }
    }

    func testTargetSizeMismatch() throws {
        // Error case: padded data doesn't match expected target size
        let originalData: [UInt8] = [0x01, 0x02, 0x03, 0x04]
        let padding1 = try FixedSizedPadding(targetSize: 10, bytes: originalData)
        let padded = padding1.paddedBytes()

        XCTAssertThrowsError(try FixedSizedPadding.fromPaddedBytes(padded, targetSize: 12)) { error in
            guard case let FixedSizedPaddingError.sizeMismatch(expected, actual) = error else {
                XCTFail("Wrong error type")
                return
            }
            XCTAssertEqual(expected, 12)
            XCTAssertEqual(actual, 10)
        }
    }
}
