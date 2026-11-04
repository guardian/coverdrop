@testable import CoverDropCore
import XCTest

final class PaddiedCompressedStringTests: XCTestCase {
    func testSuccessfullRoundtrip() throws {
        // let expected = "hello world"
        let expected = "hello world"

        let pcs = try PaddedCompressedString<U2JPaddedLength>.fromString(text: expected)

        XCTAssertTrue(
            U2JPaddedLength.bytes == pcs.value.count
        )
        XCTAssertTrue(
            pcs.value.count == Constants.authenticatedMessagePaddingLen
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
            let pcs = try PaddedCompressedString<U2JPaddedLength>.fromString(text: message)

            XCTAssertTrue(
                U2JPaddedLength.bytes == pcs.value.count
            )
            XCTAssertTrue(
                pcs.value.count == Constants.authenticatedMessagePaddingLen
            )
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

        XCTAssertThrowsError(try PaddedCompressedString<U2JPaddedLength>.fromString(text: message)) { error in
            XCTAssertEqual(error as! PaddedCompressedStringError, PaddedCompressedStringError.compressedStringTooLong)
        }
    }

    func testIfDecompressionRatioTooHighThenError() throws {
        let message = String(repeating: "a", count: 10000)
        let pcs = try PaddedCompressedString<U2JPaddedLength>.fromString(text: message)

        XCTAssertThrowsError(try pcs.toString()) { error in
            XCTAssertEqual(error as! PaddedCompressedStringError, PaddedCompressedStringError.decompressionRatioTooHigh)
        }
    }

    func nondeterministicTestPaddingIsNonZero() throws {
        let pcs = try PaddedCompressedString<U2JPaddedLength>.fromString(text: "")

        let suffix = pcs.asUnencryptedBytes().suffix(Constants.messagePaddingLen - 100)
        XCTAssertGreaterThanOrEqual(suffix.count, 100)
        XCTAssertLessThan(suffix.filter { $0 == 0x00 }.count, 10)
    }

    func testWillErrorIfBytesAreWrongLengthU2J() throws {
        let j2uPcsBytes = try PaddedCompressedString<J2UPaddedLength>.fromString(text: "").asUnencryptedBytes()

        XCTAssertThrowsError(try PaddedCompressedString<U2JPaddedLength>
            .fromUncheckedBytes(bytes: j2uPcsBytes)) { error in
                XCTAssertEqual(error as! PaddedCompressedStringError, PaddedCompressedStringError.incorrectByteLength)
            }
    }

    func testWillErrorIfBytesAreWrongLengthJ2U() throws {
        let u2jPcsBytes = try PaddedCompressedString<U2JPaddedLength>.fromString(text: "").asUnencryptedBytes()

        XCTAssertThrowsError(try PaddedCompressedString<J2UPaddedLength>
            .fromUncheckedBytes(bytes: u2jPcsBytes)) { error in
                XCTAssertEqual(error as! PaddedCompressedStringError, PaddedCompressedStringError.incorrectByteLength)
            }
    }

    func testWillChainedRoundTripU2J() throws {
        let message = "Hello!"

        let u2jPcsBytes = try PaddedCompressedString<U2JPaddedLength>.fromString(text: message).asUnencryptedBytes()

        let u2jRoundTripPcs = try PaddedCompressedString<U2JPaddedLength>.fromUncheckedBytes(bytes: u2jPcsBytes)

        XCTAssertEqual(message, try u2jRoundTripPcs.toString())
        XCTAssertEqual(u2jRoundTripPcs.asUnencryptedBytes(), u2jPcsBytes)
        XCTAssertEqual(u2jRoundTripPcs.asUnencryptedBytes().count, Constants.authenticatedMessagePaddingLen)
    }

    func testWillChainedRoundTripJ2U() throws {
        let message = "Hi!"

        let j2uPcsBytes = try PaddedCompressedString<J2UPaddedLength>.fromString(text: message).asUnencryptedBytes()

        let j2uRoundTripPcs = try PaddedCompressedString<J2UPaddedLength>.fromUncheckedBytes(bytes: j2uPcsBytes)

        XCTAssertEqual(message, try j2uRoundTripPcs.toString())
        XCTAssertEqual(j2uRoundTripPcs.asUnencryptedBytes(), j2uPcsBytes)
        XCTAssertEqual(j2uRoundTripPcs.asUnencryptedBytes().count, Constants.messagePaddingLen)
    }
}
