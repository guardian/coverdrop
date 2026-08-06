@testable import CoverDropCore
import Sodium
import XCTest

final class DeadDropDecryptionTests: XCTestCase {
    func testDecryptMessageParsesTextMessage() throws {
        let initalMessage = "This is a test message"
        let textMessage = try PaddedCompressedString.fromString(text: initalMessage).asUnencryptedBytes()

        let journalistData = try XCTUnwrap(PublicKeysHelper.shared.testDefaultJournalist)

        let result = DeadDropMessageParser.parseMessage(
            messageBytes: textMessage,
            journalistData: journalistData,
            deadDropId: 1,
            dateReceived: DateFunction.currentTime()
        )
        if case let .incomingMessage(message: incomingMessage) = result,
           case let .textMessage(message: messageText) = incomingMessage {
            XCTAssertEqual(messageText.messageText, initalMessage)
        } else {
            XCTFail("Failed to parse message")
        }
    }

    func testDecryptMessageIgnoresDeprecatedHandoverFlag() throws {
        // 0x01 is the deprecated handover type flag; it must not trigger any logic
        var handoverMessage: [UInt8] = [0x01]

        let padding: [UInt8] = Array(repeating: 0x00, count: Constants.messagePaddingLen - handoverMessage.count)
        handoverMessage.append(contentsOf: padding)

        let journalistKey = try XCTUnwrap(PublicKeysHelper.shared.testDefaultJournalist)

        let result = DeadDropMessageParser.parseMessage(
            messageBytes: handoverMessage,
            journalistData: journalistKey,
            deadDropId: 1,
            dateReceived: DateFunction.currentTime()
        )
        XCTAssertNil(result)
    }

    func testDecryptMessageFailsOnEmptyMessage() throws {
        let handoverMessage: [UInt8] = []
        let journalistData = try XCTUnwrap(PublicKeysHelper.shared.testDefaultJournalist)

        let result = DeadDropMessageParser.parseMessage(
            messageBytes: handoverMessage,
            journalistData: journalistData,
            deadDropId: 1,
            dateReceived: DateFunction.currentTime()
        )
        XCTAssertNil(result)
    }
}
