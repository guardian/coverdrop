@testable import CoverDropCore
import XCTest

/// Message identity must use only what the dead-drop signature covers: the message and its whole-second
/// timestamp. Mirrors Android's `StoredMessage` equality.
final class MessageDataTests: XCTestCase {
    private func incomingMessage(
        sender: JournalistData,
        text: String = "This is a test message",
        dateReceived: Date
    ) -> Message {
        return .incomingMessage(message: .textMessage(message: IncomingMessageData(
            sender: sender,
            messageText: text,
            dateReceived: dateReceived
        )))
    }

    /// The sub-second component is unsigned, so a malicious API can vary it and the dead drop still
    /// verifies. Also covers messages stored by earlier app versions.
    func testIncomingMessages_whenDifferingOnlyInSubSecondPrecision_thenDeduplicated() throws {
        let sender = try XCTUnwrap(PublicKeysHelper.shared.testDefaultJournalist)
        let whole = Date(timeIntervalSince1970: 1_696_446_404)
        let fractional = Date(timeIntervalSince1970: 1_696_446_404.873736)

        let stored = incomingMessage(sender: sender, dateReceived: fractional)
        let redecrypted = incomingMessage(sender: sender, dateReceived: whole)

        XCTAssertEqual(stored, redecrypted)
        XCTAssertEqual(Set([stored, redecrypted]).count, 1)
    }

    func testIncomingMessages_whenTimestampsDifferInWholeSeconds_thenBothKept() throws {
        let sender = try XCTUnwrap(PublicKeysHelper.shared.testDefaultJournalist)

        let first = incomingMessage(sender: sender, dateReceived: Date(timeIntervalSince1970: 1_696_446_404))
        let second = incomingMessage(sender: sender, dateReceived: Date(timeIntervalSince1970: 1_696_446_405))

        XCTAssertNotEqual(first, second)
        XCTAssertEqual(Set([first, second]).count, 2)
    }

    func testIncomingMessages_whenTextDiffers_thenBothKept() throws {
        let sender = try XCTUnwrap(PublicKeysHelper.shared.testDefaultJournalist)
        let dateReceived = Date(timeIntervalSince1970: 1_696_446_404)

        let first = incomingMessage(sender: sender, text: "first", dateReceived: dateReceived)
        let second = incomingMessage(sender: sender, text: "second", dateReceived: dateReceived)

        XCTAssertNotEqual(first, second)
        XCTAssertEqual(Set([first, second]).count, 2)
    }
}
