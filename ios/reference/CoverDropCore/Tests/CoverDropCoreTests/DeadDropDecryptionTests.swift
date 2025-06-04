@testable import CoverDropCore
import Sodium
import XCTest

final class DeadDropDecryptionTests: XCTestCase {
    func testDecryptMessageParsesTextMessage() async throws {
        let initalMessage = "This is a test message"
        let textMessage = try PaddedCompressedString.fromString(text: initalMessage).asUnencryptedBytes()

        let journalistData = PublicKeysHelper.shared.testDefaultJournalist!

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

    func testDecryptMessageParsesHandoverMessage() async throws {
        let journalistId = "static_test_journalist"
        let journalistIdBytes: [UInt8] = Array(journalistId.utf8)

        var handoverMessage = [Constants.flagJ2UMessageTypeHandover]
        handoverMessage.append(contentsOf: journalistIdBytes)

        let padding: [UInt8] = Array(repeating: 0x00, count: Constants.messagePaddingLen - handoverMessage.count)
        handoverMessage.append(contentsOf: padding)

        let journalistKey = PublicKeysHelper.shared.testDefaultJournalist!

        let result = DeadDropMessageParser.parseMessage(
            messageBytes: handoverMessage,
            journalistData: journalistKey,
            deadDropId: 1,
            dateReceived: DateFunction.currentTime()
        )
        if case let .incomingMessage(message: incomingMessage) = result,
           case let .handoverMessage(message: messageData) = incomingMessage {
            XCTAssertEqual(messageData.handoverTo, journalistId)
        } else {
            XCTFail("Failed to parse message")
        }
    }

    func testDecryptMessageFailsOnEmptyMessage() async throws {
        let handoverMessage: [UInt8] = []
        let journalistData = PublicKeysHelper.shared.testDefaultJournalist!

        let result = DeadDropMessageParser.parseMessage(
            messageBytes: handoverMessage,
            journalistData: journalistData,
            deadDropId: 1,
            dateReceived: DateFunction.currentTime()
        )
        XCTAssertNil(result)
    }
}
