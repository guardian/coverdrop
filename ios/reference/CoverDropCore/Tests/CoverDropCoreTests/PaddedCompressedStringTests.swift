@testable import CoverDropCore
import XCTest

final class PaddiedCompressedStringTests: XCTestCase {
    func testSuccessfullRoundtrip() throws {
        // let expected = "hello world"
        let expected = "hello world"

        let pcs = try PaddedCompressedString.fromString(text: expected)

        XCTAssertTrue(
            Constants.messagePaddingLen == pcs.value.count
        )

        let actual: String = try pcs.toString()

        XCTAssertEqual(expected, actual)
    }

    func testIsAlwaysTheSameSize() throws {
        let messages = [
            "a",
            "this is a small message",
            "this is a longer message with a few extra words"
        ]

        for message in messages {
            let pcs = try PaddedCompressedString.fromString(text: message)

            XCTAssertTrue(pcs.value.count == Constants.messagePaddingLen)
        }
    }

    func testWillErrorIfStringIsTooLong() {
        let message = """
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Integer dolor
            nulla, ornare et tristique imperdiet, dictum sit amet velit. Curabitur pharetra erat sed
            neque interdum, non mattis tortor auctor. Curabitur eu ipsum ac neque semper eleifend.
            Orci varius natoque penatibus et magnis dis parturient montes, nascetur ridiculus mus.
            Integer erat mi, ultrices nec arcu ut, sagittis sollicitudin est. In hac habitasse
            platea dictumst. Sed in efficitur elit. Curabitur nec commodo elit. Aliquam tincidunt
            rutrum nisl ut facilisis. Aenean ornare ut mauris eget lacinia. Mauris a felis quis orci
            auctor varius sit amet eget est. Curabitur a urna sit amet diam sagittis aliquet eget eu
            sapien. Curabitur a pharetra purus.
            Nulla facilisi. Suspendisse potenti. Morbi mollis aliquet sapien sed faucibus. Donec
            aliquam nibh nibh, ac faucibus felis aliquam at. Pellentesque egestas enim sem, eu
            tempor urna posuere eget. Cras fermentum commodo neque ac gravida.
        """

        XCTAssertThrowsError(try PaddedCompressedString.fromString(text: message)) { error in
            XCTAssertEqual(error as! PaddedCompressedStringError, PaddedCompressedStringError.compressedStringTooLong)
        }
    }

    func testIfDecompressionRatioTooHighThenError() throws {
        let message = String(repeating: "a", count: 10000)
        let pcs = try PaddedCompressedString.fromString(text: message)

        XCTAssertThrowsError(try pcs.toString()) { error in
            XCTAssertEqual(error as! PaddedCompressedStringError, PaddedCompressedStringError.decompressionRatioTooHigh)
        }
    }

    func nondeterministicTestPaddingIsNonZero() throws {
        let pcs = try PaddedCompressedString.fromString(text: "")

        let suffix = pcs.asUnencryptedBytes().suffix(Constants.messagePaddingLen - 100)
        XCTAssertGreaterThanOrEqual(suffix.count, 100)
        XCTAssertLessThan(suffix.filter { $0 == 0x00 }.count, 10)
    }
}
